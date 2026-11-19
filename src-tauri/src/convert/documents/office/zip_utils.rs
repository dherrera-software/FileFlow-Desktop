//! Utilidades para empaquetado ZIP y lectura de XML en documentos de oficina.

use quick_xml::events::BytesStart;
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use super::super::{Block, BlockKind};

pub fn read_zip_entry(bytes: &[u8], name: &str, kind: &str) -> Result<Vec<u8>, String> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|_| format!("El archivo {kind} está dañado o no es un paquete ZIP válido."))?;
    let mut file = archive
        .by_name(name)
        .map_err(|_| format!("El archivo {kind} no contiene la parte esperada ({name})."))?;
    let mut output = Vec::new();
    file.read_to_end(&mut output)
        .map_err(|_| format!("No se pudo leer el contenido del archivo {kind}."))?;
    Ok(output)
}

pub fn build_zip(files: &[(&str, Vec<u8>)]) -> Result<Vec<u8>, String> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in files {
        writer
            .start_file(*name, SimpleFileOptions::default())
            .map_err(|_| "No se pudo empaquetar el archivo de salida.".to_string())?;
        writer
            .write_all(data)
            .map_err(|_| "No se pudo escribir el archivo de salida.".to_string())?;
    }
    let cursor = writer
        .finish()
        .map_err(|_| "No se pudo cerrar el archivo de salida.".to_string())?;
    Ok(cursor.into_inner())
}

pub fn build_zip_with_mimetype(
    mimetype: &str,
    files: &[(&str, Vec<u8>)],
) -> Result<Vec<u8>, String> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file(
            "mimetype",
            SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
        )
        .map_err(|_| "No se pudo empaquetar el archivo de salida.".to_string())?;
    writer
        .write_all(mimetype.as_bytes())
        .map_err(|_| "No se pudo escribir el archivo de salida.".to_string())?;
    for (name, data) in files {
        writer
            .start_file(*name, SimpleFileOptions::default())
            .map_err(|_| "No se pudo empaquetar el archivo de salida.".to_string())?;
        writer
            .write_all(data)
            .map_err(|_| "No se pudo escribir el archivo de salida.".to_string())?;
    }
    let cursor = writer
        .finish()
        .map_err(|_| "No se pudo cerrar el archivo de salida.".to_string())?;
    Ok(cursor.into_inner())
}

pub fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn push_block(blocks: &mut Vec<Block>, kind: BlockKind, text: &str) {
    let text = text.trim().to_string();
    if text.is_empty() {
        return;
    }
    blocks.push(Block { kind, text });
}

pub fn heading_level(value: &str) -> Option<u8> {
    let lower = value.to_ascii_lowercase();
    let digits = lower
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect::<String>();
    digits.parse::<u8>().ok()
}

pub fn attribute(start: &BytesStart<'_>, name: &str) -> Option<String> {
    for attribute in start.attributes().flatten() {
        if attribute.key.local_name().as_ref() == name {
            return attribute
                .normalized_value(quick_xml::XmlVersion::Explicit1_0)
                .ok()
                .map(|value| value.into_owned());
        }
    }
    None
}

pub fn named_entity(reference: &quick_xml::events::BytesRef<'_>) -> Option<char> {
    match reference.as_ref() {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => None,
    }
}

pub fn find_attribute_in_xml(xml: &str, element: &str, attribute_name: &str) -> Option<String> {
    let mut reader = quick_xml::Reader::from_str(xml);
    while let Ok(event) = reader.read_event() {
        if matches!(event, quick_xml::events::Event::Eof) {
            break;
        }
        if let quick_xml::events::Event::Start(start) | quick_xml::events::Event::Empty(start) =
            event
            && start.local_name().as_ref() == element
            && let Some(value) = attribute(&start, attribute_name)
        {
            return Some(value);
        }
    }
    None
}
