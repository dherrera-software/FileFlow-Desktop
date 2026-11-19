//! Archivos comprimidos. ZIP se lee y escribe en Rust puro; RAR y 7Z delegan
//! en UnRAR/WinRAR o 7-Zip, que se detectan antes de ofrecer la ruta.
//!
//! La conversión entre contenedores extrae el contenido y lo vuelve a
//! empaquetar. Los nombres se sanean y se rechazan rutas absolutas, `..` y
//! enlaces simbólicos. Se aplica un límite de expansión para evitar bombas ZIP.

use std::ffi::OsStr;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

use crate::convert::external;

const MAX_UNPACKED_BYTES: u64 = 100 * 1024 * 1024;
const MAX_ENTRIES: usize = 2000;

pub fn convert(bytes: &[u8], source: &str, target: &str) -> Result<Vec<u8>, String> {
    let workspace = external::Workspace::new()?;
    let content = workspace.path("contenido");
    fs::create_dir_all(&content)
        .map_err(|_| "No se pudo preparar el espacio para extraer el archivo.".to_string())?;
    extract(bytes, source, &content)?;
    create(&content, target)
}

pub fn route_available(source: &str, target: &str) -> bool {
    can_extract(source) && can_create(target)
}

fn can_extract(source: &str) -> bool {
    match source {
        "ZIP" => true,
        "RAR" => external::unrar().is_some() || external::seven_zip().is_some(),
        "7Z" => external::seven_zip().is_some(),
        _ => false,
    }
}

fn can_create(target: &str) -> bool {
    match target {
        "ZIP" => true,
        "RAR" => external::rar().is_some(),
        "7Z" => external::seven_zip().is_some(),
        _ => false,
    }
}

fn extract(bytes: &[u8], source: &str, destination: &Path) -> Result<(), String> {
    let workspace = external::Workspace::new()?;
    match source {
        "ZIP" => extract_zip(bytes, destination),
        "RAR" | "7Z" => {
            let tool = if source == "RAR" {
                external::unrar().or_else(external::seven_zip)
            } else {
                external::seven_zip()
            };
            let tool = tool.ok_or_else(|| {
                format!(
                    "Para extraer {source} necesitas UnRAR/WinRAR o 7-Zip instalado. FileFlow no incluye esos binarios."
                )
            })?;
            let input =
                workspace.write(&format!("entrada.{}", source.to_ascii_lowercase()), bytes)?;
            let mut args: Vec<&OsStr> = vec![OsStr::new("x"), OsStr::new("-y")];
            let output_argument;
            if external::seven_zip().as_deref() == Some(tool.as_path()) {
                output_argument = format!("-o{}", destination.display());
                args.push(OsStr::new(&output_argument));
                args.push(input.as_os_str());
            } else {
                output_argument = format!("{}{}", destination.display(), std::path::MAIN_SEPARATOR);
                args.push(input.as_os_str());
                args.push(OsStr::new(&output_argument));
            }
            external::run_tool(&tool, &args, None)?;
            ensure_expanded_size(destination)
        }
        _ => Err(format!("La extracción desde {source} no está disponible.")),
    }
}

fn extract_zip(bytes: &[u8], destination: &Path) -> Result<(), String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| {
        "El ZIP está dañado o usa un método de compresión no soportado.".to_string()
    })?;
    if archive.len() > MAX_ENTRIES {
        return Err(format!(
            "El ZIP contiene más de {MAX_ENTRIES} archivos; extráelo por partes."
        ));
    }
    let mut total = 0u64;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|_| "No se pudo leer una entrada del ZIP.".to_string())?;
        let relative = entry.enclosed_name().ok_or_else(|| {
            "El ZIP contiene una ruta insegura (absoluta o con «..»).".to_string()
        })?;
        if entry.is_dir() {
            continue;
        }
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(
                "El ZIP contiene enlaces simbólicos; por seguridad no se extraen.".to_string(),
            );
        }
        total = total.saturating_add(entry.size());
        if total > MAX_UNPACKED_BYTES {
            return Err(
                "El ZIP se expande por encima de 100 MB; extráelo con una herramienta dedicada."
                    .to_string(),
            );
        }
        let output = destination.join(&relative);
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)
                .map_err(|_| "No se pudo crear una carpeta al extraer el ZIP.".to_string())?;
        }
        let mut file = fs::File::create(&output)
            .map_err(|_| "No se pudo escribir un archivo al extraer el ZIP.".to_string())?;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let read = entry
                .read(&mut buffer)
                .map_err(|_| "El ZIP contiene datos corruptos.".to_string())?;
            if read == 0 {
                break;
            }
            file.write_all(&buffer[..read])
                .map_err(|_| "No se pudo escribir un archivo al extraer el ZIP.".to_string())?;
        }
    }
    ensure_expanded_size(destination)
}

fn ensure_expanded_size(directory: &Path) -> Result<(), String> {
    let mut total = 0u64;
    for file in files_in(directory)? {
        let size = fs::metadata(&file)
            .map_err(|_| "No se pudo revisar el contenido extraído.".to_string())?
            .len();
        total = total.saturating_add(size);
        if total > MAX_UNPACKED_BYTES {
            return Err(
                "El archivo comprimido se expande por encima de 100 MB; extráelo con una herramienta dedicada."
                    .to_string(),
            );
        }
    }
    Ok(())
}

fn create(directory: &Path, target: &str) -> Result<Vec<u8>, String> {
    let files = files_in(directory)?;
    if files.is_empty() {
        return Err("El archivo comprimido no contiene archivos.".to_string());
    }
    match target {
        "ZIP" => create_zip(directory, &files),
        "RAR" | "7Z" => create_with_tool(directory, target),
        _ => Err(format!("La creación de {target} no está disponible.")),
    }
}

fn create_zip(directory: &Path, files: &[PathBuf]) -> Result<Vec<u8>, String> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut output);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for file in files {
            let relative = file
                .strip_prefix(directory)
                .map_err(|_| "Ruta interna no válida.".to_string())?;
            let name = relative.to_string_lossy().replace('\\', "/");
            writer
                .start_file(name, options)
                .map_err(|_| "No se pudo añadir un archivo al ZIP.".to_string())?;
            let bytes = fs::read(file)
                .map_err(|_| "No se pudo leer un archivo al crear el ZIP.".to_string())?;
            writer
                .write_all(&bytes)
                .map_err(|_| "No se pudo escribir el ZIP.".to_string())?;
        }
        writer
            .finish()
            .map_err(|_| "No se pudo cerrar el ZIP.".to_string())?;
    }
    Ok(output.into_inner())
}

fn create_with_tool(directory: &Path, target: &str) -> Result<Vec<u8>, String> {
    let (program, args) = if target == "RAR" {
        let program = external::rar().ok_or_else(|| {
            "Crear RAR requiere WinRAR (Rar.exe). Instálalo o define FILEFLOW_RAR.".to_string()
        })?;
        (program, vec!["a", "-r", "-idq", "-y"])
    } else {
        let program = external::seven_zip().ok_or_else(|| {
            "Crear 7Z requiere 7-Zip. Instálalo o define FILEFLOW_7Z.".to_string()
        })?;
        (program, vec!["a", "-t7z", "-mx=5", "-y"])
    };
    let workspace = external::Workspace::new()?;
    let output = workspace.path(&format!("salida.{}", target.to_ascii_lowercase()));
    let output_text = output.to_string_lossy().into_owned();
    let mut full: Vec<&OsStr> = args.iter().map(OsStr::new).collect();
    full.push(OsStr::new(&output_text));
    full.push(OsStr::new("."));
    external::run_tool(&program, &full, Some(directory))?;
    let bytes =
        fs::read(&output).map_err(|_| format!("{} no generó el archivo {target}.", target))?;
    if bytes.is_empty() {
        return Err(format!("La conversión produjo un {target} vacío."));
    }
    Ok(bytes)
}

/// Lista recursiva y ordenada de archivos, sin seguir enlaces simbólicos.
fn files_in(directory: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    collect_files(directory, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|_| "No se pudo leer el contenido extraído.".to_string())?;
    for entry in entries {
        let entry = entry.map_err(|_| "No se pudo leer el contenido extraído.".to_string())?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|_| "No se pudo revisar el contenido extraído.".to_string())?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            collect_files(&path, files)?;
        } else if metadata.is_file() {
            files.push(path);
        }
        if files.len() > MAX_ENTRIES {
            return Err(format!(
                "El archivo comprimido contiene más de {MAX_ENTRIES} archivos."
            ));
        }
    }
    Ok(())
}

/// Mapa nombre -> contenido usado por las pruebas y por futuros empaquetados.
#[cfg(test)]
fn sample_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut buffer);
        let options = zip::write::SimpleFileOptions::default();
        for (name, content) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(content).unwrap();
        }
        writer.finish().unwrap();
    }
    buffer.into_inner()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn read_zip_entries(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let mut map = BTreeMap::new();
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).unwrap();
            let mut content = Vec::new();
            entry.read_to_end(&mut content).unwrap();
            if !entry.is_dir() {
                map.insert(entry.name().to_string(), content);
            }
        }
        map
    }

    #[test]
    fn zip_repacks_without_losing_files() {
        let zip = sample_zip(&[
            ("carpeta/nota.txt", b"hola" as &[u8]),
            ("datos.csv", b"a,b\n1,2\n"),
        ]);
        let repacked = convert(&zip, "ZIP", "ZIP").unwrap();
        let entries = read_zip_entries(&repacked);
        assert_eq!(
            entries.get("carpeta/nota.txt").map(Vec::as_slice),
            Some(&b"hola"[..])
        );
        assert!(entries.contains_key("datos.csv"));
    }

    #[test]
    fn zip_slip_paths_are_rejected() {
        let zip = sample_zip(&[("../fuera.txt", b"x" as &[u8])]);
        let error = convert(&zip, "ZIP", "ZIP").unwrap_err();
        assert!(error.contains("insegura") || error.contains("ruta"));
    }

    #[test]
    fn corrupt_zip_is_rejected() {
        let error = convert(b"PK\x03\x04basura", "ZIP", "ZIP").unwrap_err();
        assert!(error.contains("dañado") || error.contains("ZIP"));
    }

    #[test]
    fn zip_and_rar_round_trip_when_winrar_is_available() {
        if !route_available("ZIP", "RAR") || !route_available("RAR", "ZIP") {
            eprintln!("WinRAR no disponible: se omite la prueba RAR");
            return;
        }
        let zip = sample_zip(&[("carpeta/nota.txt", b"hola" as &[u8])]);
        let rar = convert(&zip, "ZIP", "RAR").unwrap();
        assert!(rar.starts_with(b"Rar!"));
        let repacked = convert(&rar, "RAR", "ZIP").unwrap();
        let entries = read_zip_entries(&repacked);
        assert_eq!(
            entries.get("carpeta/nota.txt").map(Vec::as_slice),
            Some(&b"hola"[..])
        );
    }

    #[test]
    fn archive_route_requirements_are_explicit() {
        assert!(route_available("ZIP", "ZIP"));
        assert_eq!(route_available("ZIP", "RAR"), external::rar().is_some());
        assert_eq!(
            route_available("7Z", "ZIP"),
            external::seven_zip().is_some()
        );
    }
}
