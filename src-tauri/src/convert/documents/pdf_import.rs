//! Extracción y normalización de texto y estructura desde documentos PDF.

use std::collections::{HashMap, HashSet};

use super::types::{Block, BlockKind, BlockStyle};

pub fn read_pdf_text(bytes: &[u8]) -> Result<Vec<Block>, String> {
    let pages = pdf_extract::extract_text_from_mem_by_pages(bytes).map_err(|_| {
        "No se pudo leer el PDF. Puede estar dañado o protegido con contraseña.".to_string()
    })?;
    if pages.is_empty() || pages.iter().all(|page| page.trim().is_empty()) {
        return Err(
            "El PDF no contiene texto seleccionable (posible escaneo). FileFlow no aplica OCR."
                .to_string(),
        );
    }

    let parsed_pages = pages
        .iter()
        .map(|page| {
            page.replace("\r\n", "\n")
                .replace('\r', "\n")
                .lines()
                .map(normalize_pdf_line)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let chrome = detect_running_chrome(&parsed_pages);
    let mut blocks: Vec<Block> = Vec::new();
    let total_pages = parsed_pages.len();

    for (page_idx, lines) in parsed_pages.iter().enumerate() {
        let mut paragraph_lines: Vec<String> = Vec::new();

        let flush = |blocks: &mut Vec<Block>, paragraph_lines: &mut Vec<String>| {
            if paragraph_lines.is_empty() {
                return;
            }
            let text = join_paragraph_lines(paragraph_lines);
            paragraph_lines.clear();
            if !text.trim().is_empty() {
                blocks.push(Block::paragraph(text));
            }
        };

        for line in lines {
            if line.is_empty() {
                flush(&mut blocks, &mut paragraph_lines);
                continue;
            }
            if is_page_number_line(line) || chrome.contains(&pdf_signature(line)) {
                flush(&mut blocks, &mut paragraph_lines);
                continue;
            }
            if let Some(level) = pdf_heading_level(line) {
                flush(&mut blocks, &mut paragraph_lines);
                blocks.push(Block {
                    kind: BlockKind::Heading(level),
                    text: line.clone(),
                    style: BlockStyle::default(),
                    runs: Vec::new(),
                });
                continue;
            }
            if let Some(item_text) = detect_list_item(line) {
                flush(&mut blocks, &mut paragraph_lines);
                blocks.push(Block {
                    kind: BlockKind::ListItem,
                    text: item_text,
                    style: BlockStyle::default(),
                    runs: Vec::new(),
                });
                continue;
            }
            paragraph_lines.push(line.clone());
        }
        flush(&mut blocks, &mut paragraph_lines);

        // Inserta salto de página entre páginas consecutivas del PDF
        if page_idx + 1 < total_pages && !blocks.is_empty() {
            blocks.push(Block::page_break());
        }
    }

    if blocks.is_empty() {
        return Err(
            "El PDF no contiene texto seleccionable (posible escaneo). FileFlow no aplica OCR."
                .to_string(),
        );
    }
    Ok(blocks)
}

/// Une líneas consecutivas de un mismo párrafo fluido eliminando saltos duros y uniendo palabras separadas con guion.
fn join_paragraph_lines(lines: &[String]) -> String {
    let mut result = String::new();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if i == 0 || result.is_empty() {
            result.push_str(trimmed);
            continue;
        }

        // Detección de palabra cortada al final de la línea: e.g. "organiza-" + "ción" -> "organización"
        if result.ends_with('-')
            && result.len() >= 2
            && result.as_bytes()[result.len() - 2].is_ascii_alphabetic()
            && trimmed
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_lowercase())
        {
            result.pop(); // Quita el guion
            result.push_str(trimmed);
        } else {
            result.push(' ');
            result.push_str(trimmed);
        }
    }
    result
}

/// Detecta si una línea es un elemento de lista con viñeta o número.
fn detect_list_item(line: &str) -> Option<String> {
    let trimmed = line.trim();
    for prefix in ["• ", "- ", "* ", "– ", "— "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            let item = rest.trim();
            if !item.is_empty() {
                return Some(item.to_string());
            }
        }
    }
    None
}

pub fn extract_pdf_raw_text(bytes: &[u8]) -> Result<String, String> {
    pdf_extract::extract_text_from_mem(bytes).map_err(|_| {
        "No se pudo leer el PDF. Puede estar dañado o protegido con contraseña.".to_string()
    })
}

/// Normaliza una línea extraída de un PDF colapsando espacios múltiples.
pub fn normalize_pdf_line(line: &str) -> String {
    let mut output = String::new();
    let mut previous_space = false;
    for character in line.chars() {
        if character == ' ' || character == '\t' {
            if !previous_space && !output.is_empty() {
                output.push(' ');
                previous_space = true;
            }
        } else {
            output.push(character);
            previous_space = false;
        }
    }
    output.trim().to_string()
}

/// Firma estable para detectar texto repetido (cabeceras / pies de página).
pub fn pdf_signature(line: &str) -> String {
    let mut output = String::new();
    let mut previous_space = false;
    let mut previous_digit = false;
    for character in line.chars() {
        if character.is_ascii_digit() {
            if !previous_digit {
                output.push('#');
                previous_space = false;
            }
            previous_digit = true;
            continue;
        }
        previous_digit = false;
        let mapped = character.to_ascii_lowercase();
        if mapped == ' ' {
            if !previous_space && !output.is_empty() {
                output.push(' ');
                previous_space = true;
            }
        } else {
            output.push(mapped);
            previous_space = false;
        }
    }
    output.trim().to_string()
}

pub fn is_page_number_line(line: &str) -> bool {
    let lower = line.trim().to_ascii_lowercase();
    if lower.is_empty() || !lower.chars().any(|character| character.is_ascii_digit()) {
        return false;
    }
    if lower.split_whitespace().count() > 5 {
        return false;
    }
    lower.starts_with("page ")
        || lower.starts_with("página ")
        || lower.starts_with("pagina ")
        || lower
            .chars()
            .all(|character| character.is_ascii_digit() || " /-–—".contains(character))
}

pub fn detect_running_chrome(pages: &[Vec<String>]) -> HashSet<String> {
    if pages.len() < 3 {
        return HashSet::new();
    }
    let mut counts: HashMap<String, usize> = HashMap::new();
    for lines in pages {
        let mut seen = HashSet::new();
        let top = lines.len().min(8);
        let bottom = lines.len().saturating_sub(8);
        for index in (0..top).chain(bottom..lines.len()) {
            let line = &lines[index];
            if line.is_empty() {
                continue;
            }
            let signature = pdf_signature(line);
            if signature.chars().count() < 8 {
                continue;
            }
            if seen.insert(signature.clone()) {
                *counts.entry(signature).or_default() += 1;
            }
        }
    }
    counts
        .into_iter()
        .filter(|(_, count)| *count >= 3 && *count * 2 >= pages.len())
        .map(|(signature, _)| signature)
        .collect()
}

pub fn pdf_heading_level(line: &str) -> Option<u8> {
    let trimmed = line.trim();
    if trimmed.len() > 90 || trimmed.ends_with(['.', ',', ';', ':']) {
        return None;
    }
    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let prefix = parts.next()?;
    let rest = parts.next()?.trim_start();
    let segments = prefix.trim_end_matches('.').split('.').collect::<Vec<_>>();
    if segments.is_empty() || segments.len() > 3 {
        return None;
    }
    if !segments.iter().all(|segment| {
        !segment.is_empty() && segment.len() <= 2 && segment.bytes().all(|b| b.is_ascii_digit())
    }) {
        return None;
    }
    let first = segments[0].parse::<u8>().ok()?;
    if first == 0 || first > 40 {
        return None;
    }
    if !rest.chars().next()?.is_uppercase() {
        return None;
    }
    Some(segments.len() as u8)
}
