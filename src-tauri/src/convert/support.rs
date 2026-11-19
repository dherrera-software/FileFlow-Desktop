//! Utilidades compartidas: decodificación de texto, detección de separador y
//! validación de firmas (MIME real) antes de convertir.

use crate::convert::catalog::{self, Format};

pub struct DecodedText {
    pub text: String,
    /// Etiqueta de la codificación detectada; se conserva para diagnósticos y pruebas.
    #[allow(dead_code)]
    pub encoding: &'static str,
}

/// Decodifica UTF-8 (con o sin BOM), UTF-16 con BOM y, como último recurso,
/// Windows-1252 para archivos heredados. Devuelve también la codificación usada.
pub fn decode_text(bytes: &[u8]) -> DecodedText {
    if let Some(rest) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        return DecodedText {
            text: String::from_utf8_lossy(rest).into_owned(),
            encoding: "utf-8",
        };
    }
    if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        let (text, _, _) = encoding_rs::UTF_16LE.decode(rest);
        return DecodedText {
            text: text.into_owned(),
            encoding: "utf-16le",
        };
    }
    if let Some(rest) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        let (text, _, _) = encoding_rs::UTF_16BE.decode(rest);
        return DecodedText {
            text: text.into_owned(),
            encoding: "utf-16be",
        };
    }
    match std::str::from_utf8(bytes) {
        Ok(text) => DecodedText {
            text: text.to_string(),
            encoding: "utf-8",
        },
        Err(_) => {
            let (text, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
            DecodedText {
                text: text.into_owned(),
                encoding: "windows-1252",
            }
        }
    }
}

/// Detecta el separador más probable de un CSV: coma, punto y coma o tabulación.
/// Ignora separadores dentro de comillas y líneas vacías.
pub fn detect_separator(text: &str) -> char {
    let mut candidates = [0usize; 3];
    let delimiters = [',', ';', '\t'];
    if let Some(line) = text.lines().find(|line| !line.trim().is_empty()) {
        let mut quoted = false;
        for character in line.chars() {
            if character == '"' {
                quoted = !quoted;
                continue;
            }
            if quoted {
                continue;
            }
            if let Some(index) = delimiters.iter().position(|value| *value == character) {
                candidates[index] += 1;
            }
        }
    }
    let best = candidates
        .iter()
        .enumerate()
        .max_by_key(|(index, count)| (**count, std::cmp::Reverse(*index)));
    match best {
        Some((index, count)) if *count > 0 => delimiters[index],
        _ => ',',
    }
}

/// Comprueba que los bytes correspondan al formato declarado por la extensión.
/// No sustituye la validación profunda del decodificador, pero atrapa archivos
/// renombrados y archivos corruptos evidentes.
pub fn validate_signature(format: &Format, bytes: &[u8]) -> Result<(), String> {
    let ok = match format.id {
        "PDF" => bytes.starts_with(b"%PDF"),
        "DOCX" | "ODT" | "XLSX" | "ODS" | "EPUB" | "PPTX" | "ODP" => is_zip(bytes),
        "DOC" | "XLS" | "PPT" => is_ole2(bytes),
        "RTF" => {
            let head = bytes.iter().take(32).copied().collect::<Vec<_>>();
            let head = String::from_utf8_lossy(&head);
            head.trim_start().starts_with("{\\rtf")
        }
        "JPG" => bytes.starts_with(&[0xFF, 0xD8, 0xFF]),
        "PNG" => bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]),
        "WEBP" => bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP"),
        "BMP" => bytes.starts_with(b"BM"),
        "GIF" => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        "TIFF" => bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*"),
        "ICO" => bytes.starts_with(&[0x00, 0x00, 0x01, 0x00]),
        "HEIC" => is_isobmff(
            bytes,
            &[
                "heic", "heix", "hevc", "hevx", "heim", "heis", "mif1", "msf1",
            ],
        ),
        "AVIF" => is_isobmff(bytes, &["avif", "avis"]),
        "SVG" => {
            let head = String::from_utf8_lossy(&bytes[..bytes.len().min(4096)]);
            head.contains("<svg") || head.contains("<?xml")
        }
        "MP3" => {
            bytes.starts_with(b"ID3")
                || (bytes.len() > 1 && bytes[0] == 0xFF && bytes[1] & 0xE0 == 0xE0)
        }
        "WAV" => bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WAVE"),
        "FLAC" => bytes.starts_with(b"fLaC"),
        "AAC" => {
            bytes.starts_with(b"ID3")
                || (bytes.len() > 1 && bytes[0] == 0xFF && bytes[1] & 0xF6 == 0xF0)
        }
        "OGG" | "OPUS" => bytes.starts_with(b"OggS"),
        "M4A" => is_isobmff(bytes, &["M4A ", "M4A", "mp42", "isom", "iso5", "dash"]),
        "WMA" | "WMV" => bytes.starts_with(&[0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11]),
        "AIFF" => {
            bytes.starts_with(b"FORM") && matches!(bytes.get(8..12), Some(b"AIFF") | Some(b"AIFC"))
        }
        "MP4" | "MOV" => is_isobmff(
            bytes,
            &[
                "isom", "iso2", "iso5", "iso6", "mp41", "mp42", "avc1", "M4V ", "qt  ", "dash",
                "MSNV",
            ],
        ),
        "AVI" => bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"AVI "),
        "MKV" | "WEBM" => bytes.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]),
        "FLV" => bytes.starts_with(b"FLV"),
        "ZIP" => is_zip(bytes),
        "RAR" => bytes.starts_with(b"Rar!\x1A\x07"),
        "7Z" => bytes.starts_with(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]),
        // Formatos basados en texto: la validación real la hace su parser.
        "TXT" | "MD" | "HTML" | "CSV" | "TSV" | "JSON" | "XML" | "YAML" | "SQL" => true,
        _ => true,
    };
    if ok {
        Ok(())
    } else {
        Err(format!(
            "El archivo no parece un {} válido (extensión .{}). ¿Está dañado o fue renombrado?",
            format.id, format.output_extension
        ))
    }
}

fn is_zip(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04])
        || bytes.starts_with(&[0x50, 0x4B, 0x05, 0x06])
        || bytes.starts_with(&[0x50, 0x4B, 0x07, 0x08])
}

fn is_ole2(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1])
}

fn is_isobmff(bytes: &[u8], brands: &[&str]) -> bool {
    if bytes.len() < 12 || &bytes[4..8] != b"ftyp" {
        return false;
    }
    let size = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    let end = size.clamp(12, bytes.len());
    bytes[8..end]
        .chunks(4)
        .any(|chunk| brands.iter().any(|brand| chunk == brand.as_bytes()))
}

pub fn extension_for(id: &str) -> &'static str {
    catalog::format(id)
        .map(|format| format.output_extension)
        .unwrap_or("bin")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_boms_and_legacy_encodings() {
        assert_eq!(decode_text(&[0xEF, 0xBB, 0xBF, b'a']).text, "a");
        assert_eq!(decode_text(&[0xFF, 0xFE, b'a', 0x00]).text, "a");
        // 0xF1 es "ñ" en Windows-1252 (no es UTF-8 válido).
        let legacy = decode_text(&[0x63, 0x61, 0xF1, 0xF3, 0x6E]);
        assert_eq!(legacy.text, "cañón");
        assert_eq!(legacy.encoding, "windows-1252");
    }

    #[test]
    fn detects_common_separators() {
        assert_eq!(detect_separator("a,b,c\n1,2,3"), ',');
        assert_eq!(detect_separator("a;b;c\n1;2;3"), ';');
        assert_eq!(detect_separator("a\tb\tc\n1\t2\t3"), '\t');
        assert_eq!(detect_separator("una sola columna"), ',');
        assert_eq!(detect_separator("a,\"b;c\"\n"), ',');
    }

    #[test]
    fn rejects_renamed_files() {
        let jpg = catalog::format("JPG").unwrap();
        assert!(validate_signature(jpg, b"no soy una imagen").is_err());
        let png = catalog::format("PNG").unwrap();
        assert!(validate_signature(png, &[0x89, b'P', b'N', b'G']).is_err());
        let pdf = catalog::format("PDF").unwrap();
        assert!(validate_signature(pdf, b"%PDF-1.7").is_ok());
        let heic = catalog::format("HEIC").unwrap();
        let mut heic_bytes = vec![0, 0, 0, 24, b'f', b't', b'y', b'p', b'h', b'e', b'i', b'c'];
        heic_bytes.extend_from_slice(&[0, 0, 0, 0, b'm', b'i', b'f', b'1']);
        assert!(validate_signature(heic, &heic_bytes).is_ok());
        assert!(validate_signature(heic, b"no soy un HEIC").is_err());
        let avif = catalog::format("AVIF").unwrap();
        let mut avif_bytes = vec![0, 0, 0, 24, b'f', b't', b'y', b'p', b'a', b'v', b'i', b'f'];
        avif_bytes.extend_from_slice(&[0, 0, 0, 0, b'm', b'i', b'f', b'1']);
        assert!(validate_signature(avif, &avif_bytes).is_ok());
    }
}
