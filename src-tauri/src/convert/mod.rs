mod markup;
mod pdf;
mod tabular;
mod text;

use std::fs;
use std::path::{Path, PathBuf};

const MAX_BYTES: u64 = 25 * 1024 * 1024;

pub const MAX_BYTES_LIMIT: u64 = MAX_BYTES;
const FORMATS: [&str; 10] = ["TXT", "HTML", "MD", "JPG", "JPEG", "PNG", "CSV", "TSV", "JSON", "XML"];

pub fn format_count() -> usize {
    FORMATS.len()
}

pub fn conversion_count() -> usize {
    FORMATS.iter().map(|format| targets_for(format).len()).sum()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileInspection {
    pub name: String,
    pub format: String,
    pub size: u64,
    pub targets: Vec<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

pub fn inspect_file(path: &Path) -> Result<FileInspection, String> {
    let name = file_name(path);
    let format = detect_format(&name)?;
    let size = file_size(path)?;
    ensure_size(size)?;
    let (width, height) = if is_image(&format) {
        let bytes = fs::read(path).map_err(|error| io_message(&error, "leer la imagen"))?;
        let (width, height) = pdf::image_dimensions(&bytes)?;
        (Some(width), Some(height))
    } else {
        (None, None)
    };
    Ok(FileInspection {
        name,
        format: format.to_string(),
        size,
        targets: targets_for(&format).into_iter().map(str::to_string).collect(),
        width,
        height,
    })
}

pub fn convert_file(path: &Path, target: &str, output: &Path) -> Result<(), String> {
    if same_path(path, output) {
        return Err("Elige un archivo de salida distinto al original.".to_string());
    }
    let name = file_name(path);
    let source = detect_format(&name)?;
    let target = target.trim().to_ascii_uppercase();
    if !targets_for(&source).contains(&target.as_str()) {
        return Err(format!("La conversión de {source} a {target} no está disponible."));
    }
    let size = file_size(path)?;
    ensure_size(size)?;
    let bytes = fs::read(path).map_err(|error| io_message(&error, "leer el archivo"))?;
    let stem = stem_of(&name);
    let converted = convert_bytes(&bytes, &source, &target, &stem)?;
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            return Err("La carpeta de destino no existe.".to_string());
        }
    }
    fs::write(output, converted).map_err(|error| io_message(&error, "guardar el archivo"))
}

fn convert_bytes(bytes: &[u8], source: &str, target: &str, title: &str) -> Result<Vec<u8>, String> {
    if is_image(source) && target == "PDF" {
        return pdf::image_to_pdf(bytes);
    }
    let text = decode_text(bytes)?;
    let output = match (source, target) {
        ("TXT", "MD") => text::txt_to_markdown(&text),
        ("TXT", "HTML") => text::txt_to_html(&text, title),
        ("TXT", "PDF") => return Ok(pdf::text_to_pdf(&text)?),
        ("HTML", "MD") => markup::html_to_markdown(&text)?,
        ("MD", "HTML") => markup::markdown_to_html(&text, title),
        ("CSV", "JSON") => tabular::csv_to_json(&text)?,
        ("CSV", "XML") => tabular::csv_to_xml(&text)?,
        ("TSV", "CSV") => tabular::tsv_to_csv(&text)?,
        ("JSON", "CSV") => tabular::json_to_csv(&text)?,
        ("XML", "CSV") => tabular::xml_to_csv(&text)?,
        _ => return Err(format!("La conversión de {source} a {target} no está disponible.")),
    };
    Ok(output.into_bytes())
}

fn decode_text(bytes: &[u8]) -> Result<String, String> {
    let text = String::from_utf8(bytes.to_vec()).map_err(|_| {
        "El archivo no está en UTF-8. Guárdalo como UTF-8 e inténtalo de nuevo.".to_string()
    })?;
    Ok(text.trim_start_matches('\u{feff}').to_string())
}

fn detect_format(file_name: &str) -> Result<&'static str, String> {
    let extension = Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let format = match extension.as_str() {
        "txt" => "TXT",
        "html" | "htm" => "HTML",
        "md" | "markdown" => "MD",
        "jpg" => "JPG",
        "jpeg" => "JPEG",
        "png" => "PNG",
        "csv" => "CSV",
        "tsv" => "TSV",
        "json" => "JSON",
        "xml" => "XML",
        _ => {
            return Err(if extension.is_empty() {
                "Este tipo de archivo no es un formato compatible. FileFlow admite documentos de texto (TXT, HTML, MD), imágenes (JPG, PNG) y datos (CSV, TSV, JSON, XML).".to_string()
            } else {
                format!(".{extension} no es un formato compatible. FileFlow admite documentos de texto (TXT, HTML, MD), imágenes (JPG, PNG) y datos (CSV, TSV, JSON, XML).")
            });
        }
    };
    Ok(format)
}

fn targets_for(format: &str) -> Vec<&'static str> {
    match format {
        "TXT" => vec!["PDF", "HTML", "MD"],
        "HTML" => vec!["MD"],
        "MD" => vec!["HTML"],
        "JPG" | "JPEG" | "PNG" => vec!["PDF"],
        "CSV" => vec!["JSON", "XML"],
        "TSV" => vec!["CSV"],
        "JSON" => vec!["CSV"],
        "XML" => vec!["CSV"],
        _ => Vec::new(),
    }
}

fn is_image(format: &str) -> bool {
    matches!(format, "JPG" | "JPEG" | "PNG")
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "archivo".to_string())
}

fn stem_of(file_name: &str) -> String {
    Path::new(file_name)
        .file_stem()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "archivo".to_string())
}

fn file_size(path: &Path) -> Result<u64, String> {
    let meta = fs::metadata(path).map_err(|error| io_message(&error, "leer el archivo"))?;
    if !meta.is_file() {
        return Err("Selecciona un archivo, no una carpeta.".to_string());
    }
    Ok(meta.len())
}

fn ensure_size(size: u64) -> Result<(), String> {
    if size == 0 {
        return Err("Este archivo está vacío; no hay nada para convertir.".to_string());
    }
    if size > MAX_BYTES {
        return Err(format!(
            "Este archivo pesa {}. El límite es de 25 MB por archivo.",
            format_bytes(size)
        ));
    }
    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut index = 0;
    while value >= 1024.0 && index < units.len() - 1 {
        value /= 1024.0;
        index += 1;
    }
    if index == 0 || value >= 10.0 {
        format!("{value:.0} {}", units[index])
    } else {
        format!("{value:.1} {}", units[index])
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => weak_path(left) == weak_path(right),
    }
}

fn weak_path(path: &Path) -> PathBuf {
    let mut clean = PathBuf::new();
    for part in path.components() {
        clean.push(part);
    }
    clean
}

fn io_message(error: &std::io::Error, action: &str) -> String {
    if error.kind() == std::io::ErrorKind::NotFound {
        format!("No se pudo {action}: el archivo no existe.")
    } else {
        format!("No se pudo {action}.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn scratch(name: &str) -> PathBuf {
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("fileflow-{unique}-{name}"))
    }

    #[test]
    fn converts_text_document_on_disk() {
        let source = scratch("nota.txt");
        let output = scratch("nota.html");
        fs::write(&source, "Hola <mundo>\nSegunda línea\n\nOtro párrafo").unwrap();
        convert_file(&source, "HTML", &output).unwrap();
        let html = fs::read_to_string(&output).unwrap();
        assert!(html.contains("&lt;mundo&gt;"));
        assert!(html.contains("<br>"));
        assert!(html.contains("Otro párrafo"));
        let _ = fs::remove_file(source);
        let _ = fs::remove_file(output);
    }

    #[test]
    fn refuses_to_overwrite_the_source() {
        let source = scratch("datos.csv");
        fs::write(&source, "a,b\n1,2\n").unwrap();
        let error = convert_file(&source, "JSON", &source).unwrap_err();
        assert!(error.contains("distinto"));
        let _ = fs::remove_file(source);
    }

    #[test]
    fn rejects_unknown_and_empty_files() {
        let unknown = scratch("foto.gif");
        fs::write(&unknown, b"GIF89a").unwrap();
        assert!(inspect_file(&unknown).unwrap_err().contains(".gif"));
        let empty = scratch("vacio.txt");
        fs::write(&empty, b"").unwrap();
        assert!(inspect_file(&empty).unwrap_err().contains("vacío"));
        let _ = fs::remove_file(unknown);
        let _ = fs::remove_file(empty);
    }
}
