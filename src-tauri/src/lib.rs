mod convert;

use convert::{conversion_count, convert_file, format_count, inspect_file, MAX_BYTES_LIMIT};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileInfo {
    name: String,
    format: String,
    size: u64,
    targets: Vec<String>,
    width: Option<u32>,
    height: Option<u32>,
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
        size: info.size,
        targets: info.targets,
        width: info.width,
        height: info.height,
    })
}

#[tauri::command]
fn convert(path: String, target: String, output_path: String) -> Result<(), String> {
    convert_file(Path::new(&path), &target, Path::new(&output_path))
}

#[tauri::command]
fn engine_info() -> EngineInfo {
    EngineInfo {
        name: "FileFlow Engine".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        max_bytes: MAX_BYTES_LIMIT,
        formats: format_count(),
        conversions: conversion_count(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![inspect, convert, engine_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn scratch(name: &str) -> PathBuf {
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("fileflow-ipc-{unique}-{name}"))
    }

    #[test]
    fn engine_info_reports_the_local_engine() {
        let info = engine_info();
        assert_eq!(info.name, "FileFlow Engine");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(info.formats, 10);
        assert_eq!(info.conversions, 13);
        assert_eq!(info.max_bytes, 25 * 1024 * 1024);
        assert!(!info.platform.is_empty());
    }

    #[test]
    fn command_layer_inspects_and_converts_a_file() {
        let source = scratch("nota.txt");
        let output = scratch("nota.md");
        fs::write(&source, "Hola FileFlow\n\nSegunda parte").unwrap();

        let inspected = inspect(source.to_string_lossy().into_owned()).unwrap();
        assert_eq!(inspected.name, source.file_name().unwrap().to_string_lossy().to_string());
        assert_eq!(inspected.format, "TXT");
        assert_eq!(inspected.targets.join(","), "PDF,HTML,MD");
        assert_eq!(inspected.width, None);

        convert(
            source.to_string_lossy().into_owned(),
            "MD".to_string(),
            output.to_string_lossy().into_owned(),
        )
        .unwrap();

        let markdown = fs::read_to_string(&output).unwrap();
        assert!(markdown.contains("Hola FileFlow"));
        assert!(markdown.contains("Segunda parte"));

        let _ = fs::remove_file(source);
        let _ = fs::remove_file(output);
    }

    #[test]
    fn command_layer_rejects_unavailable_targets() {
        let source = scratch("datos.csv");
        let output = scratch("datos.pdf");
        fs::write(&source, "a,b\n1,2\n").unwrap();
        let error = convert(
            source.to_string_lossy().into_owned(),
            "PDF".to_string(),
            output.to_string_lossy().into_owned(),
        )
        .unwrap_err();
        assert!(error.contains("no está disponible"));
        let _ = fs::remove_file(source);
    }
}
