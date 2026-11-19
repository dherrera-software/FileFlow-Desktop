//! Conversión de documentos. El modelo intermedio son bloques de texto con
//! un tipo semántico básico, suficiente para conservar párrafos y encabezados
//! entre TXT, Markdown, HTML, DOCX, ODT, RTF, EPUB y PDF.

pub mod markup;
pub mod office;
pub mod pdf;
pub mod text;
pub mod tts;

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

use crate::convert::sheets;
use crate::convert::support;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockKind {
    Paragraph,
    Heading(u8),
    ListItem,
    Quote,
    Code,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub kind: BlockKind,
    pub text: String,
}

impl Block {
    pub fn paragraph(text: impl Into<String>) -> Self {
        Block {
            kind: BlockKind::Paragraph,
            text: text.into(),
        }
    }

    #[cfg(test)]
    pub fn heading(level: u8, text: impl Into<String>) -> Self {
        Block {
            kind: BlockKind::Heading(level.clamp(1, 6)),
            text: text.into(),
        }
    }
}

pub fn convert(bytes: &[u8], source: &str, target: &str, title: &str) -> Result<Vec<u8>, String> {
    if source == "DOC" {
        return convert_legacy_doc(bytes, target);
    }
    if source == "PDF" {
        match target {
            "JPG" | "PNG" => return pdf::render_first_page(bytes, target),
            "XLSX" | "CSV" => return pdf_table(bytes, target),
            _ => {}
        }
    }
    // Rutas con mayor fidelidad que usan el texto original directamente.
    let text = match source {
        "TXT" | "MD" | "HTML" => Some(support::decode_text(bytes).text),
        _ => None,
    };
    if let Some(text) = text.as_deref() {
        if source == "TXT" && target == "MP3" {
            return tts::text_to_mp3(text);
        }
        match (source, target) {
            ("TXT", "MD") => return Ok(text::txt_to_markdown(text).into_bytes()),
            ("TXT", "HTML") => return Ok(text::txt_to_html(text, title).into_bytes()),
            ("MD", "HTML") => return Ok(markup::markdown_to_html(text, title).into_bytes()),
            ("HTML", "MD") => {
                return Ok(markup::html_to_markdown(text)?.into_bytes());
            }
            _ => {}
        }
    }

    let blocks = read_blocks(bytes, source)?;
    if blocks.is_empty() {
        return Err("El documento no contiene texto que se pueda convertir.".to_string());
    }
    match target {
        "TXT" => Ok(blocks_to_text(&blocks).into_bytes()),
        "MD" => Ok(blocks_to_markdown(&blocks).into_bytes()),
        "HTML" => {
            let body = blocks_to_html_body(&blocks);
            Ok(text::html_document(title, &body, true).into_bytes())
        }
        "DOCX" => office::write_docx(&blocks, title),
        "ODT" => office::write_odt(&blocks, title),
        "RTF" => Ok(office::write_rtf(&blocks, title)),
        "EPUB" => office::write_epub(&blocks, title),
        "PDF" => pdf::text_to_pdf(&blocks_to_text(&blocks)),
        _ => Err(format!(
            "La conversión de {source} a {target} no está disponible."
        )),
    }
}

fn convert_legacy_doc(bytes: &[u8], target: &str) -> Result<Vec<u8>, String> {
    if crate::convert::external::soffice().is_none() {
        return Err(
            "Convertir DOC requiere LibreOffice instalado (o define FILEFLOW_SOFFICE).".to_string(),
        );
    }
    match target {
        "MD" => {
            let text = crate::convert::presentations::soffice_convert(bytes, "doc", "txt")?;
            let text = String::from_utf8_lossy(&text).into_owned();
            Ok(text::txt_to_markdown(&text).into_bytes())
        }
        "DOCX" | "ODT" | "RTF" | "TXT" | "HTML" | "PDF" | "EPUB" => {
            crate::convert::presentations::soffice_convert(
                bytes,
                "doc",
                &target.to_ascii_lowercase(),
            )
        }
        _ => Err(format!(
            "La conversión de DOC a {target} no está disponible."
        )),
    }
}

/// Extrae filas de un PDF de texto. Es una heurística deliberadamente
/// conservadora: separa por tabulaciones, comas y dos o más espacios, igual
/// que el backend web. No inventa celdas ni aplica OCR.
fn pdf_table(bytes: &[u8], target: &str) -> Result<Vec<u8>, String> {
    let blocks = read_pdf_text(bytes)?;
    let text = blocks_to_text(&blocks);
    let rows = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(split_pdf_line)
        .filter(|row| !row.is_empty())
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return Err("El PDF no contiene filas de texto que se puedan tabular.".to_string());
    }
    sheets::convert_rows(rows, target)
}

fn split_pdf_line(line: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut spaces = 0usize;
    let push = |cells: &mut Vec<String>, current: &mut String| {
        let value = current.trim().to_string();
        if !value.is_empty() {
            cells.push(value);
        }
        current.clear();
    };
    for character in line.chars() {
        match character {
            '\t' | ',' => {
                push(&mut cells, &mut current);
                spaces = 0;
            }
            ' ' => {
                spaces += 1;
                if spaces >= 2 {
                    push(&mut cells, &mut current);
                }
            }
            _ => {
                if spaces == 1 && !current.is_empty() {
                    current.push(' ');
                }
                spaces = 0;
                current.push(character);
            }
        }
    }
    push(&mut cells, &mut current);
    cells
}

fn read_blocks(bytes: &[u8], source: &str) -> Result<Vec<Block>, String> {
    match source {
        "TXT" => Ok(plain_text_blocks(&support::decode_text(bytes).text)),
        "MD" => Ok(markdown_to_blocks(&support::decode_text(bytes).text)),
        "HTML" => {
            let html = support::decode_text(bytes).text;
            let markdown = markup::html_to_markdown(&html)?;
            Ok(markdown_to_blocks(&markdown))
        }
        "DOCX" => office::read_docx(bytes),
        "ODT" => office::read_odt(bytes),
        "RTF" => office::read_rtf(bytes),
        "EPUB" => office::read_epub(bytes),
        "PDF" => read_pdf_text(bytes),
        _ => Err(format!("La conversión desde {source} no está disponible.")),
    }
}

fn read_pdf_text(bytes: &[u8]) -> Result<Vec<Block>, String> {
    let text = pdf_extract::extract_text_from_mem(bytes).map_err(|_| {
        "No se pudo leer el PDF. Puede estar dañado o protegido con contraseña.".to_string()
    })?;
    if text.trim().is_empty() {
        return Err(
            "El PDF no contiene texto seleccionable (posible escaneo). FileFlow no aplica OCR."
                .to_string(),
        );
    }
    Ok(plain_text_blocks(&text))
}

pub fn plain_text_blocks(text: &str) -> Vec<Block> {
    let normalized = text::normalize_newlines(text)
        .trim_start_matches('\u{feff}')
        .to_string();
    let mut blocks = Vec::new();
    let mut current = String::new();
    for line in normalized.lines() {
        if line.trim().is_empty() {
            if !current.trim().is_empty() {
                blocks.push(Block::paragraph(current.trim().to_string()));
                current.clear();
            }
        } else {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line.trim_end());
        }
    }
    if !current.trim().is_empty() {
        blocks.push(Block::paragraph(current.trim().to_string()));
    }
    if blocks.is_empty() && !normalized.trim().is_empty() {
        blocks.push(Block::paragraph(normalized.trim().to_string()));
    }
    blocks
}

pub fn markdown_to_blocks(markdown: &str) -> Vec<Block> {
    let normalized = text::normalize_newlines(markdown);
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut kind = BlockKind::Paragraph;
    let mut in_code = false;
    for event in Parser::new_ext(&normalized, options) {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    current.clear();
                    kind = BlockKind::Heading(level as u8);
                }
                Tag::Item => {
                    current.clear();
                    kind = BlockKind::ListItem;
                }
                Tag::BlockQuote(_) => {
                    current.clear();
                    kind = BlockKind::Quote;
                }
                Tag::CodeBlock(_) => {
                    current.clear();
                    kind = BlockKind::Code;
                    in_code = true;
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) | TagEnd::Paragraph | TagEnd::Item | TagEnd::CodeBlock => {
                    let text = current.trim().to_string();
                    if !text.is_empty() {
                        blocks.push(Block {
                            kind: kind.clone(),
                            text,
                        });
                    }
                    current.clear();
                    if matches!(tag_end, TagEnd::Heading(_) | TagEnd::CodeBlock) {
                        kind = BlockKind::Paragraph;
                        in_code = false;
                    }
                }
                TagEnd::BlockQuote(_) => {
                    let text = current.trim().to_string();
                    if !text.is_empty() {
                        blocks.push(Block {
                            kind: BlockKind::Quote,
                            text,
                        });
                    }
                    current.clear();
                    kind = BlockKind::Paragraph;
                }
                _ => {}
            },
            Event::Text(value) => current.push_str(&value),
            Event::Code(value) => current.push_str(&value),
            Event::SoftBreak => {
                if in_code {
                    current.push('\n');
                } else {
                    current.push(' ');
                }
            }
            Event::HardBreak => current.push('\n'),
            _ => {}
        }
    }
    blocks
}

pub fn blocks_to_text(blocks: &[Block]) -> String {
    let mut output = blocks
        .iter()
        .map(|block| block.text.trim().to_string())
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    output.push('\n');
    output
}

pub fn blocks_to_markdown(blocks: &[Block]) -> String {
    let mut lines = Vec::new();
    for block in blocks {
        match block.kind {
            BlockKind::Heading(level) => lines.push(format!(
                "{} {}",
                "#".repeat(level.clamp(1, 6) as usize),
                block.text
            )),
            BlockKind::ListItem => lines.push(format!("- {}", block.text.replace('\n', " "))),
            BlockKind::Quote => lines.push(
                block
                    .text
                    .lines()
                    .map(|line| format!("> {line}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            BlockKind::Code => lines.push(format!("```\n{}\n```", block.text)),
            BlockKind::Paragraph => lines.push(block.text.clone()),
        }
    }
    format!("{}\n", lines.join("\n\n"))
}

pub fn blocks_to_html_body(blocks: &[Block]) -> String {
    let mut body = String::new();
    for block in blocks {
        let content = text::escape_html(&block.text).replace('\n', "<br>");
        match block.kind {
            BlockKind::Heading(level) => body.push_str(&format!(
                "<h{level}>{content}</h{level}>\n",
                level = level.clamp(1, 6)
            )),
            BlockKind::ListItem => body.push_str(&format!("<ul><li>{content}</li></ul>\n")),
            BlockKind::Quote => body.push_str(&format!("<blockquote>{content}</blockquote>\n")),
            BlockKind::Code => body.push_str(&format!(
                "<pre><code>{}</code></pre>\n",
                text::escape_html(&block.text)
            )),
            BlockKind::Paragraph => body.push_str(&format!("<p>{content}</p>\n")),
        }
    }
    body.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_blocks_keep_headings_and_lists() {
        let blocks = markdown_to_blocks("# Título\n\nTexto **fuerte**\n\n- uno\n- dos\n");
        assert!(
            blocks
                .iter()
                .any(|block| block.kind == BlockKind::Heading(1))
        );
        assert!(blocks.iter().any(|block| block.text.contains("fuerte")));
        assert_eq!(
            blocks
                .iter()
                .filter(|block| block.kind == BlockKind::ListItem)
                .count(),
            2
        );
    }

    #[test]
    fn document_round_trip_docx_to_markdown() {
        let source = "# Encabezado\n\nPrimer párrafo.\n\nSegundo párrafo.".as_bytes();
        let docx = convert(source, "MD", "DOCX", "Doc").unwrap();
        let markdown = convert(&docx, "DOCX", "MD", "Doc").unwrap();
        let markdown = String::from_utf8(markdown).unwrap();
        assert!(markdown.contains("# Encabezado"));
        assert!(markdown.contains("Segundo párrafo."));
    }

    #[test]
    fn pdf_output_starts_with_pdf_header() {
        let pdf = convert(b"Hola mundo", "TXT", "PDF", "Nota").unwrap();
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn corrupt_zip_is_rejected_with_a_clear_message() {
        let error = convert(b"PK\x03\x04basura", "DOCX", "MD", "Doc").unwrap_err();
        assert!(error.contains("dañado") || error.contains("ZIP"));
    }
}
