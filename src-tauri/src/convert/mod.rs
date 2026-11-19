mod archives;
mod audio;
mod catalog;
mod documents;
mod external;
mod images;
mod presentations;
mod sheets;
mod support;
mod video;

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub use audio::AudioOptions;
pub use catalog::{Category, FORMATS, format_count};

const MAX_BYTES: u64 = 25 * 1024 * 1024;

pub const MAX_BYTES_LIMIT: u64 = MAX_BYTES;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertOptions {
    pub audio: Option<AudioOptions>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileInspection {
    pub name: String,
    pub format: String,
    pub category: String,
    pub category_label: String,
    pub mime: String,
    pub size: u64,
    pub targets: Vec<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_ms: Option<u64>,
}

pub fn inspect_file(path: &Path) -> Result<FileInspection, String> {
    let name = file_name(path);
    let format = detect_format(&name)?;
    let size = file_size(path)?;
    ensure_size(size)?;
    let bytes = fs::read(path).map_err(|error| io_message(&error, "leer el archivo"))?;
    support::validate_signature(format, &bytes)?;
    let size_hint = match format.category {
        Category::Image => images::dimensions(&bytes, format.id),
        Category::Video => video::dimensions(&bytes, format.id),
        _ => None,
    };
    let (width, height) = match size_hint {
        Some((width, height)) => (Some(width), Some(height)),
        None => (None, None),
    };
    let duration_ms = match format.category {
        Category::Audio => audio::duration_ms(&bytes, format.id),
        Category::Video => video::duration_ms(path),
        _ => None,
    };
    let targets = available_targets(format.id);
    if targets.is_empty() {
        return Err(format!(
            "No hay conversiones disponibles para {} en este equipo. {}",
            format.id,
            unavailable_hint(format.id)
        ));
    }
    Ok(FileInspection {
        name,
        format: format.id.to_string(),
        category: format.category.key().to_string(),
        category_label: format.category.label().to_string(),
        mime: format.primary_mime().to_string(),
        size,
        targets,
        width,
        height,
        duration_ms,
    })
}

pub fn convert_file(
    path: &Path,
    target: &str,
    output: &Path,
    options: ConvertOptions,
) -> Result<(), String> {
    if same_path(path, output) {
        return Err("Elige un archivo de salida distinto al original.".to_string());
    }
    let name = file_name(path);
    let format = detect_format(&name)?;
    let source = format.id;
    let target = target.trim().to_ascii_uppercase();
    if !targets_for(source)
        .iter()
        .any(|available| *available == target.as_str())
        || !route_available(source, &target)
    {
        return Err(format!(
            "La conversión de {source} a {target} no está disponible en este equipo. {}",
            unavailable_hint(source)
        ));
    }
    let size = file_size(path)?;
    ensure_size(size)?;
    let bytes = fs::read(path).map_err(|error| io_message(&error, "leer el archivo"))?;
    support::validate_signature(format, &bytes)?;
    let title = stem_of(&name);
    let converted = convert_bytes(&bytes, source, &target, &title, options)?;
    if output
        .parent()
        .is_some_and(|parent| !parent.as_os_str().is_empty() && !parent.exists())
    {
        return Err("La carpeta de destino no existe.".to_string());
    }
    fs::write(output, converted).map_err(|error| io_message(&error, "guardar el archivo"))
}

fn convert_bytes(
    bytes: &[u8],
    source: &str,
    target: &str,
    title: &str,
    options: ConvertOptions,
) -> Result<Vec<u8>, String> {
    let format = catalog::format(source)
        .ok_or_else(|| format!("El formato {source} no está registrado."))?;
    match format.category {
        Category::Document => documents::convert(bytes, source, target, title),
        Category::Image => images::convert(bytes, source, target, title),
        Category::Sheet => sheets::convert(bytes, source, target, title),
        Category::Presentation => presentations::convert(bytes, source, target, title),
        Category::Video => video::convert(bytes, source, target, options.audio.unwrap_or_default()),
        Category::Audio => audio::convert(bytes, source, target, options.audio.unwrap_or_default()),
        Category::Archive => archives::convert(bytes, source, target),
    }
}

/// Comprueba si una ruta declarada puede completarse con los motores
/// instalados. Se llama al inspeccionar y antes de convertir, de modo que la UI
/// nunca muestra una combinación que vaya a fallar por falta de herramienta.
fn route_available(source: &str, target: &str) -> bool {
    let Some(format) = catalog::format(source) else {
        return false;
    };
    match format.category {
        Category::Document => match source {
            "DOC" => external::soffice().is_some(),
            "PDF" => match target {
                "JPG" | "PNG" => external::ghostscript().is_some(),
                _ => true,
            },
            "TXT" => target != "MP3" || documents::tts::available(),
            _ => true,
        },
        Category::Sheet => true,
        Category::Image => {
            if matches!(source, "HEIC" | "AVIF") {
                return external::ffmpeg().is_ok();
            }
            match target {
                "AVIF" => external::ffmpeg_has_encoder("libaom-av1"),
                "MP4" => video::route_available("MP4"),
                _ => true,
            }
        }
        Category::Presentation => presentations::route_available(source, target),
        Category::Audio => audio::target_available(target),
        Category::Video => video::route_available(target),
        Category::Archive => archives::route_available(source, target),
    }
}

fn available_targets(source: &str) -> Vec<String> {
    catalog::targets_for(source)
        .into_iter()
        .filter(|target| route_available(source, target))
        .map(str::to_string)
        .collect()
}

fn unavailable_hint(source: &str) -> &'static str {
    match source {
        "DOC" | "PPT" => {
            "Convertir formatos binarios de Office requiere LibreOffice (o define FILEFLOW_SOFFICE)."
        }
        "PDF" => "Rasterizar PDF requiere Ghostscript (o define FILEFLOW_GS).",
        "RAR" | "7Z" => "Instala WinRAR o 7-Zip (o define FILEFLOW_RAR / FILEFLOW_7Z).",
        "HEIC" | "AVIF" => "Estos códecs requieren ffmpeg (o define FILEFLOW_FFMPEG).",
        "TXT" => "Narrar texto requiere Windows, PowerShell y ffmpeg con MP3.",
        _ => "Instala el motor local necesario y vuelve a intentarlo.",
    }
}

/// Rutas realmente disponibles en este equipo (las que la UI puede ofrecer).
pub fn available_conversion_count() -> usize {
    FORMATS
        .iter()
        .map(|format| available_targets(format.id).len())
        .sum()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatrixTarget {
    pub format: &'static str,
    pub available: bool,
    /// Herramienta que falta cuando la ruta no se puede completar.
    pub requires: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatrixFormat {
    pub format: &'static str,
    pub extensions: Vec<&'static str>,
    pub targets: Vec<MatrixTarget>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatrixCategory {
    pub key: &'static str,
    pub label: &'static str,
    pub formats: Vec<MatrixFormat>,
}

/// Matriz completa declarada por el catálogo, anotada con la disponibilidad
/// real del equipo. La UI la usa para mostrar todas las rutas sin duplicar la
/// tabla en TypeScript.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionMatrix {
    pub categories: Vec<MatrixCategory>,
    pub declared: usize,
    pub available: usize,
}

const CATEGORY_ORDER: [Category; 7] = [
    Category::Document,
    Category::Image,
    Category::Sheet,
    Category::Presentation,
    Category::Audio,
    Category::Video,
    Category::Archive,
];

pub fn conversion_matrix() -> ConversionMatrix {
    let mut declared = 0;
    let mut available = 0;
    let categories = CATEGORY_ORDER
        .iter()
        .map(|category| {
            let formats = FORMATS
                .iter()
                .filter(|format| format.category == *category)
                .map(|format| {
                    let targets = catalog::targets_for(format.id)
                        .into_iter()
                        .map(|target| {
                            let is_available = route_available(format.id, target);
                            declared += 1;
                            if is_available {
                                available += 1;
                            }
                            MatrixTarget {
                                format: target,
                                available: is_available,
                                requires: route_requirement(format.id, target),
                            }
                        })
                        .collect();
                    MatrixFormat {
                        format: format.id,
                        extensions: format.extensions.to_vec(),
                        targets,
                    }
                })
                .collect();
            MatrixCategory {
                key: category.key(),
                label: category.label(),
                formats,
            }
        })
        .collect();
    ConversionMatrix {
        categories,
        declared,
        available,
    }
}

/// Herramienta local que falta para completar una ruta declarada.
pub fn route_requirement(source: &str, target: &str) -> Option<&'static str> {
    if route_available(source, target) {
        return None;
    }
    let format = catalog::format(source)?;
    Some(match format.category {
        Category::Document => match source {
            "DOC" => "LibreOffice",
            "PDF" => "Ghostscript",
            "TXT" => "Voz de Windows y FFmpeg",
            _ => "Motor local",
        },
        Category::Image => match target {
            "AVIF" => "FFmpeg con libaom",
            _ => "FFmpeg",
        },
        Category::Presentation => "LibreOffice",
        Category::Audio => "FFmpeg",
        Category::Video => "FFmpeg",
        Category::Archive => {
            if source == "7Z" || target == "7Z" {
                "7-Zip"
            } else {
                "WinRAR"
            }
        }
        Category::Sheet => "Motor local",
    })
}

fn targets_for(format: &str) -> Vec<String> {
    catalog::targets_for(format)
        .into_iter()
        .map(str::to_string)
        .collect()
}

fn detect_format(file_name: &str) -> Result<&'static catalog::Format, String> {
    if let Some(format) = catalog::detect(file_name) {
        return Ok(format);
    }
    let extension = Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let categories = FORMATS
        .iter()
        .map(|format| format.category.label())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(", ");
    if extension.is_empty() {
        Err(format!(
            "Este archivo no tiene extensión y no se puede identificar. FileFlow admite: {categories}."
        ))
    } else {
        Err(format!(
            ".{extension} no es un formato compatible. FileFlow admite: {categories}."
        ))
    }
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
mod tests;
