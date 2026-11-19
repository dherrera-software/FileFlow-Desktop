//! Presentaciones. PPTX y ODP se leen con Rust puro para extraer el texto de
//! cada diapositiva. Las rutas que requieren renderizar diapositivas a imagen,
//! convertir PPT binario o cambiar de contenedor entre PPTX/ODP delegan en
//! LibreOffice, y sólo se exponen si `soffice` está instalado.

use std::fs;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::convert::documents::office;
use crate::convert::documents::pdf;
use crate::convert::documents::text;
use crate::convert::documents::{Block, blocks_to_html_body, blocks_to_markdown, blocks_to_text};
use crate::convert::external;

pub fn convert(bytes: &[u8], source: &str, target: &str, title: &str) -> Result<Vec<u8>, String> {
    if source == "PPT" {
        return convert_legacy(bytes, target, title);
    }
    if matches!((source, target), ("PPTX", "ODP") | ("ODP", "PPTX")) {
        return soffice_convert(bytes, source, target);
    }
    if matches!(target, "JPG" | "PNG") {
        return soffice_convert(bytes, source, target);
    }
    let slides = match source {
        "PPTX" => read_pptx(bytes)?,
        "ODP" => read_odp(bytes)?,
        _ => return Err(format!("La conversión desde {source} no está disponible.")),
    };
    let paragraphs = slides
        .iter()
        .flat_map(|slide| slide.iter())
        .map(|line| Block::paragraph(line.clone()))
        .collect::<Vec<_>>();
    if paragraphs.is_empty() {
        return Err(
            "La presentación no contiene texto editable. FileFlow no aplica OCR a diapositivas."
                .to_string(),
        );
    }
    match target {
        "TXT" => Ok(blocks_to_text(&paragraphs).into_bytes()),
        "MD" => Ok(blocks_to_markdown(&paragraphs).into_bytes()),
        "HTML" => {
            Ok(text::html_document(title, &blocks_to_html_body(&paragraphs), true).into_bytes())
        }
        "DOCX" => office::write_docx(&paragraphs, title),
        "ODT" => office::write_odt(&paragraphs, title),
        "RTF" => Ok(office::write_rtf(&paragraphs, title)),
        "EPUB" => office::write_epub(&paragraphs, title),
        "PDF" => pdf::text_to_pdf(&blocks_to_text(&paragraphs)),
        "ODP" => write_odp(&slides, title),
        _ => Err(format!(
            "La conversión de {source} a {target} no está disponible."
        )),
    }
}

pub fn route_available(source: &str, target: &str) -> bool {
    match source {
        "PPT" => external::soffice().is_some(),
        "PPTX" | "ODP" => match target {
            "JPG" | "PNG" => external::soffice().is_some(),
            "PPTX" | "ODP" => !source.eq_ignore_ascii_case(target) && external::soffice().is_some(),
            _ => true,
        },
        _ => false,
    }
}

fn convert_legacy(bytes: &[u8], target: &str, title: &str) -> Result<Vec<u8>, String> {
    if external::soffice().is_none() {
        return Err(
            "Convertir PPT requiere LibreOffice instalado (o define FILEFLOW_SOFFICE).".to_string(),
        );
    }
    match target {
        // Los formatos de texto se generan desde el PPTX intermedio con el
        // mismo motor Rust que ya usa PPTX. Convertir PPT→DOCX/ODT/EPUB
        // directamente con LibreOffice no es fiable porque son filtros de
        // Writer, no de Impress.
        "DOCX" | "ODT" | "RTF" | "TXT" | "MD" | "HTML" | "EPUB" => {
            let pptx = soffice_convert(bytes, "ppt", "pptx")?;
            convert(&pptx, "PPTX", target, title)
        }
        "PPTX" | "ODP" | "PDF" | "JPG" | "PNG" => {
            soffice_convert(bytes, "ppt", &target.to_ascii_lowercase())
        }
        _ => Err(format!(
            "La conversión de PPT a {target} no está disponible."
        )),
    }
}

/// Convierte con LibreOffice en modo consola y devuelve el archivo generado.
pub fn soffice_convert(
    bytes: &[u8],
    input_extension: &str,
    output_extension: &str,
) -> Result<Vec<u8>, String> {
    let program = external::soffice().ok_or_else(|| {
        "Esta ruta requiere LibreOffice instalado. Instálalo o define FILEFLOW_SOFFICE.".to_string()
    })?;
    let workspace = external::Workspace::new()?;
    let input = workspace.write(&format!("entrada.{input_extension}"), bytes)?;
    let outdir = workspace.path("salida");
    fs::create_dir_all(&outdir)
        .map_err(|_| "No se pudo preparar la carpeta de salida de LibreOffice.".to_string())?;
    let filter = match output_extension {
        "jpg" => "jpg:impress_jpg_Export",
        "png" => "png:impress_png_Export",
        other => other,
    };
    let convert = format!("--convert-to={filter}");
    let out_arg = format!("--outdir={}", outdir.display());
    let args: Vec<&std::ffi::OsStr> = vec![
        std::ffi::OsStr::new("--headless"),
        std::ffi::OsStr::new("--norestore"),
        std::ffi::OsStr::new(&convert),
        std::ffi::OsStr::new(&out_arg),
        input.as_os_str(),
    ];
    external::run_tool(&program, &args, None)?;
    let produced = outdir.join(format!("entrada.{output_extension}"));
    let output = fs::read(&produced).map_err(|_| {
        format!(
            "LibreOffice no generó el archivo .{output_extension}. Comprueba que la presentación no esté dañada."
        )
    })?;
    if output.is_empty() {
        return Err("LibreOffice generó un archivo vacío.".to_string());
    }
    Ok(output)
}

fn read_pptx(bytes: &[u8]) -> Result<Vec<Vec<String>>, String> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|_| {
        "La presentación PPTX está dañada o no es un archivo ZIP válido.".to_string()
    })?;
    let mut slides = Vec::new();
    for index in 1..=archive.len().max(1) {
        let name = format!("ppt/slides/slide{index}.xml");
        let mut entry = match archive.by_name(&name) {
            Ok(entry) => entry,
            Err(_) => break,
        };
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut entry, &mut xml)
            .map_err(|_| "No se pudo leer una diapositiva del PPTX.".to_string())?;
        slides.push(extract_paragraphs(&xml, "sld", "t"));
    }
    if slides.is_empty() {
        return Err("El PPTX no contiene diapositivas legibles.".to_string());
    }
    Ok(slides)
}

fn read_odp(bytes: &[u8]) -> Result<Vec<Vec<String>>, String> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|_| {
        "La presentación ODP está dañada o no es un archivo ZIP válido.".to_string()
    })?;
    let mut entry = archive
        .by_name("content.xml")
        .map_err(|_| "El ODP no contiene content.xml.".to_string())?;
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut entry, &mut xml)
        .map_err(|_| "No se pudo leer el contenido del ODP.".to_string())?;
    let pages = extract_paragraphs(&xml, "page", "p");
    if pages.is_empty() {
        return Err("El ODP no contiene diapositivas legibles.".to_string());
    }
    Ok(pages.into_iter().map(|page| vec![page]).collect())
}

/// Extrae párrafos de texto agrupados por elemento de página. Se comparan
/// nombres locales para no depender del prefijo de namespace (`a:`, `draw:`…).
fn extract_paragraphs(xml: &str, page_tag: &str, text_tag: &str) -> Vec<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut slides = Vec::new();
    let mut current: Option<String> = None;
    let mut in_text = false;
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(event)) => {
                let local = event.local_name();
                let name = local.as_ref();
                if name == page_tag && current.is_none() {
                    current = Some(String::new());
                } else if name == text_tag && current.is_some() {
                    in_text = true;
                }
            }
            Ok(Event::Text(value)) if in_text => {
                if let Some(slide) = current.as_mut() {
                    let text = value.xml_content(quick_xml::XmlVersion::Explicit1_0);
                    if !text.trim().is_empty() {
                        if !slide.is_empty() {
                            slide.push(' ');
                        }
                        slide.push_str(text.trim());
                    }
                }
            }
            Ok(Event::GeneralRef(reference)) if in_text => {
                if let Some(slide) = current.as_mut() {
                    let character = reference
                        .resolve_char_ref()
                        .ok()
                        .flatten()
                        .or_else(|| office::named_entity(&reference));
                    if let Some(character) = character {
                        slide.push(character);
                    }
                }
            }
            Ok(Event::End(event)) => {
                let local = event.local_name();
                let name = local.as_ref();
                if name == text_tag {
                    in_text = false;
                } else if name == page_tag
                    && let Some(slide) = current.take()
                    && !slide.trim().is_empty()
                {
                    slides.push(slide.trim().to_string());
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buffer.clear();
    }
    slides
}

fn write_odp(slides: &[Vec<String>], title: &str) -> Result<Vec<u8>, String> {
    let mut body = String::new();
    for (index, slide) in slides.iter().enumerate() {
        body.push_str(&format!(
            "<draw:page draw:name=\"{}\">",
            office::xml_escape(&format!("Diapositiva {}", index + 1))
        ));
        for paragraph in slide {
            body.push_str(&format!(
                "<draw:frame><draw:text-box><text:p>{}</text:p></draw:text-box></draw:frame>",
                office::xml_escape(paragraph)
            ));
        }
        body.push_str("</draw:page>");
    }
    let content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" office:version="1.2">
<office:body><office:presentation>{body}</office:presentation></office:body>
</office:document-content>"#
    );
    let meta = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-meta xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0" office:version="1.2">
<office:meta><meta:generator>FileFlow</meta:generator><dc:title xmlns:dc="http://purl.org/dc/elements/1.1/">{}</dc:title></office:meta>
</office:document-meta>"#,
        office::xml_escape(title)
    );
    let manifest = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.2">
<manifest:file-entry manifest:full-path="/" manifest:media-type="application/vnd.oasis.opendocument.presentation"/>
<manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
<manifest:file-entry manifest:full-path="meta.xml" manifest:media-type="text/xml"/>
</manifest:manifest>"#;
    office::build_zip_with_mimetype(
        "application/vnd.oasis.opendocument.presentation",
        &[
            ("content.xml", content.into_bytes()),
            ("meta.xml", meta.into_bytes()),
            ("META-INF/manifest.xml", manifest.as_bytes().to_vec()),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_pptx_text_per_slide() {
        let xml = r#"<p:sld xmlns:p="p" xmlns:a="a"><p:cSld><a:t>Hola</a:t><a:t>mundo</a:t></p:cSld></p:sld>"#;
        let slides = extract_paragraphs(xml, "sld", "t");
        assert_eq!(slides, vec!["Hola mundo".to_string()]);
    }

    #[test]
    fn extracts_odp_pages() {
        let xml = r#"<draw:page><text:p>Primera</text:p><text:p>Segunda</text:p></draw:page><draw:page><text:p>Tercera</text:p></draw:page>"#;
        let slides = extract_paragraphs(xml, "page", "p");
        assert_eq!(slides.len(), 2);
        assert!(slides[0].contains("Primera"));
    }

    #[test]
    fn presentations_without_text_are_rejected() {
        let error = convert(b"no es pptx", "PPTX", "PDF", "demo").unwrap_err();
        assert!(error.contains("dañada") || error.contains("ZIP"));
    }

    #[test]
    fn image_render_requires_libreoffice() {
        assert_eq!(
            route_available("PPTX", "PNG"),
            external::soffice().is_some()
        );
    }
}
