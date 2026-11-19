mod convert;

use convert::{
    ConversionMatrix, ConvertOptions, MAX_BYTES_LIMIT, available_conversion_count,
    conversion_matrix as build_conversion_matrix, convert_file, format_count, inspect_file,
};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileInfo {
    name: String,
    format: String,
    category: String,
    category_label: String,
    mime: String,
    size: u64,
    targets: Vec<String>,
    width: Option<u32>,
    height: Option<u32>,
    duration_ms: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EngineInfo {
    name: String,
    version: String,
    platform: String,
    max_bytes: u64,
    formats: usize,
    conversions: usize,
}

#[tauri::command]
fn inspect(path: String) -> Result<FileInfo, String> {
    let info = inspect_file(Path::new(&path))?;
    Ok(FileInfo {
        name: info.name,
        format: info.format,
        category: info.category,
        category_label: info.category_label,
        mime: info.mime,
        size: info.size,
        targets: info.targets,
        width: info.width,
        height: info.height,
        duration_ms: info.duration_ms,
    })
}

#[tauri::command]
fn convert(
    path: String,
    target: String,
    output_path: String,
    options: Option<ConvertOptions>,
) -> Result<(), String> {
    convert_file(
        Path::new(&path),
        &target,
        Path::new(&output_path),
        options.unwrap_or_default(),
    )
}

#[tauri::command]
fn engine_info() -> EngineInfo {
    EngineInfo {
        name: "FileFlow Engine".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        max_bytes: MAX_BYTES_LIMIT,
        formats: format_count(),
        conversions: available_conversion_count(),
    }
}

#[tauri::command]
fn conversion_matrix() -> ConversionMatrix {
    build_conversion_matrix()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            inspect,
            convert,
            engine_info,
            conversion_matrix
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::AudioOptions;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn scratch(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("fileflow-ipc-{unique}-{name}"))
    }

    #[test]
    fn engine_info_reports_the_local_engine() {
        let info = engine_info();
        assert_eq!(info.name, "FileFlow Engine");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(info.formats, 50);
        assert!(info.conversions > 200);
        assert_eq!(info.max_bytes, 25 * 1024 * 1024);
        assert!(!info.platform.is_empty());
    }

    #[test]
    fn conversion_matrix_matches_the_catalog() {
        let matrix = conversion_matrix();
        assert_eq!(matrix.categories.len(), 7);
        let formats = matrix
            .categories
            .iter()
            .map(|category| category.formats.len())
            .sum::<usize>();
        assert_eq!(formats, format_count());
        assert_eq!(matrix.declared, 476);
        assert_eq!(matrix.available, available_conversion_count());
        assert!(matrix.available > 0);
        assert!(matrix.available <= matrix.declared);
        for category in &matrix.categories {
            assert!(!category.formats.is_empty());
            for format in &category.formats {
                assert!(!format.targets.is_empty());
                for target in &format.targets {
                    assert_eq!(target.available, target.requires.is_none());
                }
            }
        }
    }

    #[test]
    fn command_layer_inspects_and_converts_a_file() {
        let source = scratch("nota.txt");
        let output = scratch("nota.md");
        fs::write(&source, "Hola FileFlow\n\nSegunda parte").unwrap();

        let inspected = inspect(source.to_string_lossy().into_owned()).unwrap();
        assert_eq!(
            inspected.name,
            source.file_name().unwrap().to_string_lossy().to_string()
        );
        assert_eq!(inspected.format, "TXT");
        assert_eq!(inspected.category, "document");
        assert!(inspected.targets.contains(&"DOCX".to_string()));
        assert!(inspected.targets.contains(&"PDF".to_string()));
        assert!(!inspected.targets.is_empty());
        assert_eq!(inspected.width, None);
        assert_eq!(inspected.duration_ms, None);

        convert(
            source.to_string_lossy().into_owned(),
            "MD".to_string(),
            output.to_string_lossy().into_owned(),
            None,
        )
        .unwrap();

        let markdown = fs::read_to_string(&output).unwrap();
        assert!(markdown.contains("Hola FileFlow"));
        assert!(markdown.contains("Segunda parte"));

        let _ = fs::remove_file(source);
        let _ = fs::remove_file(output);
    }

    #[test]
    fn command_layer_accepts_audio_parameters() {
        let options = ConvertOptions {
            audio: Some(AudioOptions {
                bitrate_kbps: 128,
                sample_rate_hz: 44_100,
                channels: 2,
            }),
        };
        let serialized = serde_json::to_string(&options).unwrap();
        assert!(serialized.contains("bitrateKbps"));
        let restored: ConvertOptions = serde_json::from_str(&serialized).unwrap();
        assert_eq!(restored.audio.unwrap().bitrate_kbps, 128);
    }

    #[test]
    fn command_layer_rejects_unavailable_targets() {
        let source = scratch("datos.csv");
        let output = scratch("datos.doc");
        fs::write(&source, "a,b\n1,2\n").unwrap();
        let error = convert(
            source.to_string_lossy().into_owned(),
            "DOC".to_string(),
            output.to_string_lossy().into_owned(),
            None,
        )
        .unwrap_err();
        assert!(error.contains("no está disponible"));
        let _ = fs::remove_file(source);
    }
}
