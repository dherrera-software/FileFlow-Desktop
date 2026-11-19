//! Generación de documentos PDF nativos (texto, tablas, imágenes).

use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ExtendedColorType, ImageReader, Rgb, RgbImage};
use lopdf::{Document, Object, ObjectId, Stream, dictionary};
use std::io::Cursor;

pub const PAGE_WIDTH: f32 = 595.28;
pub const PAGE_HEIGHT: f32 = 841.89;

pub fn text_to_pdf(text: &str) -> Result<Vec<u8>, String> {
    let margin = 56.7_f32;
    let font_size = 11.0_f32;
    let line_height = 15.4_f32;
    let max_lines = ((PAGE_HEIGHT - margin * 2.0) / line_height)
        .floor()
        .max(1.0) as usize;
    let mut lines = Vec::new();
    for line in text.replace("\r\n", "\n").replace('\r', "\n").split('\n') {
        lines.extend(wrap_text(line, 86));
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    let pages = lines
        .chunks(max_lines)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<_>>();

    let mut document = Document::with_version("1.4");
    let font_id = document.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
        "Encoding" => "WinAnsiEncoding"
    });
    let mut page_ids = Vec::new();
    for page_lines in pages {
        let mut content = Vec::new();
        let origin_y = PAGE_HEIGHT - margin - font_size;
        content.extend(format!("BT\n/F1 {font_size:.0} Tf\n{line_height:.1} TL\n1 0 0 1 {margin:.2} {origin_y:.2} Tm\n").into_bytes());
        for line in page_lines {
            content.push(b'(');
            content.extend(pdf_escape(&win_ansi_bytes(&line)));
            content.extend(b") Tj\nT*\n");
        }
        content.extend(b"ET\n");
        let content_id = document.add_object(Stream::new(dictionary! {}, content));
        let page_id = document.add_object(dictionary! {
            "Type" => "Page",
            "MediaBox" => media_box(),
            "Resources" => dictionary! {
                "Font" => dictionary! {
                    "F1" => font_id
                }
            },
            "Contents" => content_id
        });
        page_ids.push(page_id);
    }
    let pages_id = document.add_object(dictionary! {
        "Type" => "Pages",
        "Kids" => page_ids.into_iter().map(Object::Reference).collect::<Vec<_>>(),
        "Count" => Object::Integer(0)
    });
    fix_page_tree(&mut document, pages_id);
    finish_document(&mut document, pages_id)
}

pub fn image_to_pdf(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let (rgb, width, height) = decode_rgb(bytes)?;
    let mut jpeg = Vec::new();
    JpegEncoder::new_with_quality(&mut jpeg, 92)
        .encode(rgb.as_raw(), width, height, ExtendedColorType::Rgb8)
        .map_err(|_| "No se pudo preparar la imagen para el PDF.".to_string())?;

    let margin = 36.0_f32;
    let scale = ((PAGE_WIDTH - margin * 2.0) / width as f32)
        .min((PAGE_HEIGHT - margin * 2.0) / height as f32);
    let draw_width = width as f32 * scale;
    let draw_height = height as f32 * scale;
    let x = (PAGE_WIDTH - draw_width) / 2.0;
    let y = (PAGE_HEIGHT - draw_height) / 2.0;
    let content = format!("q\n{draw_width:.2} 0 0 {draw_height:.2} {x:.2} {y:.2} cm\n/Im0 Do\nQ\n");

    let mut document = Document::with_version("1.4");
    let image_id = document.add_object(Stream::new(
        dictionary! {
            "Type" => "XObject",
            "Subtype" => "Image",
            "Width" => width as i64,
            "Height" => height as i64,
            "ColorSpace" => "DeviceRGB",
            "BitsPerComponent" => 8,
            "Filter" => "DCTDecode"
        },
        jpeg,
    ));
    let content_id = document.add_object(Stream::new(dictionary! {}, content.into_bytes()));
    let page_id = document.add_object(dictionary! {
        "Type" => "Page",
        "MediaBox" => media_box(),
        "Resources" => dictionary! {
            "XObject" => dictionary! {
                "Im0" => image_id
            }
        },
        "Contents" => content_id
    });
    let pages_id = document.add_object(dictionary! {
        "Type" => "Pages",
        "Kids" => vec![Object::Reference(page_id)],
        "Count" => 1
    });
    fix_page_tree(&mut document, pages_id);
    finish_document(&mut document, pages_id)
}

/// Genera un PDF con una tabla monoespaciada (una hoja de cálculo por página).
pub fn table_to_pdf(rows: &[Vec<String>]) -> Result<Vec<u8>, String> {
    if rows.is_empty() {
        return text_to_pdf("(Hoja vacía)");
    }
    let columns = rows.iter().map(Vec::len).max().unwrap_or(1).max(1);
    let max_characters = 118usize;
    let mut widths = vec![8usize; columns];
    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            let length = cell.chars().count().clamp(1, 40);
            widths[index] = widths[index].max(length);
        }
    }
    let separators = 3 * columns.saturating_sub(1);
    let total: usize = widths.iter().sum::<usize>() + separators;
    if total > max_characters {
        let available = max_characters.saturating_sub(separators).max(columns);
        let sum: usize = widths.iter().sum();
        let mut scaled = widths
            .iter()
            .map(|width| ((*width * available) / sum.max(1)).max(1))
            .collect::<Vec<_>>();
        let mut current: usize = scaled.iter().sum();
        let mut index = 0;
        while current < available {
            scaled[index % columns] += 1;
            current += 1;
            index += 1;
        }
        widths = scaled;
    }

    let mut lines: Vec<String> = Vec::new();
    for (row_index, row) in rows.iter().enumerate() {
        let cells = (0..columns)
            .map(|index| {
                let value = row.get(index).map(String::as_str).unwrap_or("");
                wrap_cell(value, widths[index])
            })
            .collect::<Vec<_>>();
        let height = cells.iter().map(Vec::len).max().unwrap_or(1).max(1);
        for line_index in 0..height {
            let line = cells
                .iter()
                .enumerate()
                .map(|(index, cell)| {
                    let value = cell.get(line_index).map(String::as_str).unwrap_or("");
                    pad_cell(value, widths[index])
                })
                .collect::<Vec<_>>()
                .join(" | ");
            lines.push(line);
        }
        if row_index == 0 {
            let separator = widths
                .iter()
                .map(|width| "-".repeat(*width))
                .collect::<Vec<_>>()
                .join("-+-");
            lines.push(separator);
        }
    }

    let font_size = 8.0_f32;
    let line_height = 10.5_f32;
    let margin = 28.0_f32;
    let max_lines = ((PAGE_HEIGHT - margin * 2.0) / line_height)
        .floor()
        .max(1.0) as usize;

    let mut document = Document::with_version("1.4");
    let font_id = document.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Courier",
        "Encoding" => "WinAnsiEncoding"
    });
    let mut page_ids = Vec::new();
    for chunk in lines.chunks(max_lines) {
        let mut content = Vec::new();
        let origin_y = PAGE_HEIGHT - margin - font_size;
        content.extend(
            format!(
                "BT\n/F1 {font_size:.0} Tf\n{line_height:.1} TL\n1 0 0 1 {margin:.2} {origin_y:.2} Tm\n"
            )
            .into_bytes(),
        );
        for line in chunk {
            content.push(b'(');
            content.extend(pdf_escape(&win_ansi_bytes(line)));
            content.extend(b") Tj\nT*\n");
        }
        content.extend(b"ET\n");
        let content_id = document.add_object(Stream::new(dictionary! {}, content));
        let page_id = document.add_object(dictionary! {
            "Type" => "Page",
            "MediaBox" => media_box(),
            "Resources" => dictionary! {
                "Font" => dictionary! {
                    "F1" => font_id
                }
            },
            "Contents" => content_id
        });
        page_ids.push(page_id);
    }
    let pages_id = document.add_object(dictionary! {
        "Type" => "Pages",
        "Kids" => page_ids.into_iter().map(Object::Reference).collect::<Vec<_>>(),
        "Count" => Object::Integer(0)
    });
    fix_page_tree(&mut document, pages_id);
    finish_document(&mut document, pages_id)
}

fn wrap_cell(value: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for source_line in value.replace("\r\n", "\n").replace('\r', "\n").split('\n') {
        let mut current = String::new();
        for character in source_line.chars() {
            if current.chars().count() >= width {
                lines.push(std::mem::take(&mut current));
            }
            current.push(character);
        }
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn pad_cell(value: &str, width: usize) -> String {
    let mut output: String = value.chars().take(width).collect();
    let padding = width.saturating_sub(output.chars().count());
    output.push_str(&" ".repeat(padding));
    output
}

#[cfg(test)]
pub fn image_dimensions(bytes: &[u8]) -> Result<(u32, u32), String> {
    let image = decode_image(bytes)?;
    Ok((image.width(), image.height()))
}

fn decode_rgb(bytes: &[u8]) -> Result<(RgbImage, u32, u32), String> {
    let decoded = decode_image(bytes)?;
    let (source_width, source_height) = (decoded.width(), decoded.height());
    if source_width == 0 || source_height == 0 {
        return Err("La imagen no tiene un tamaño válido.".to_string());
    }
    let image = if source_width.max(source_height) > 4096 {
        let scale = 4096.0 / source_width.max(source_height) as f32;
        let width = ((source_width as f32) * scale).round().max(1.0) as u32;
        let height = ((source_height as f32) * scale).round().max(1.0) as u32;
        decoded.resize(width, height, image::imageops::FilterType::Triangle)
    } else {
        decoded
    };
    let (width, height) = (image.width(), image.height());
    let rgb = if image.color().has_alpha() {
        let rgba = image.to_rgba8();
        let mut rgb = RgbImage::new(width, height);
        for (x, y, pixel) in rgba.enumerate_pixels() {
            let alpha = pixel[3] as f32 / 255.0;
            let blend = |channel: u8| -> u8 {
                (channel as f32 * alpha + 255.0 * (1.0 - alpha)).round() as u8
            };
            rgb.put_pixel(
                x,
                y,
                Rgb([blend(pixel[0]), blend(pixel[1]), blend(pixel[2])]),
            );
        }
        rgb
    } else {
        image.to_rgb8()
    };
    Ok((rgb, width, height))
}

fn decode_image(bytes: &[u8]) -> Result<DynamicImage, String> {
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| "No se pudo leer la imagen.".to_string())?;
    let mut limits = image::Limits::no_limits();
    limits.max_image_width = Some(12_000);
    limits.max_image_height = Some(12_000);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);
    reader
        .decode()
        .map_err(|_| "La imagen es demasiado grande o no se pudo leer.".to_string())
}

fn media_box() -> Vec<Object> {
    vec![
        Object::Integer(0),
        Object::Integer(0),
        Object::Real(PAGE_WIDTH),
        Object::Real(PAGE_HEIGHT),
    ]
}

fn fix_page_tree(document: &mut Document, pages_id: ObjectId) {
    let page_ids = match document.get_object(pages_id) {
        Ok(Object::Dictionary(dict)) => match dict.get(b"Kids").ok() {
            Some(Object::Array(kids)) => kids
                .iter()
                .filter_map(|kid| match kid {
                    Object::Reference(id) => Some(*id),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };
    if let Ok(Object::Dictionary(dict)) = document.get_object_mut(pages_id) {
        dict.set("Count", Object::Integer(page_ids.len() as i64));
    }
    for page_id in page_ids {
        if let Ok(Object::Dictionary(dict)) = document.get_object_mut(page_id) {
            dict.set("Parent", Object::Reference(pages_id));
            dict.set("Type", Object::Name(b"Page".to_vec()));
        }
    }
}

fn finish_document(document: &mut Document, pages_id: ObjectId) -> Result<Vec<u8>, String> {
    let catalog_id = document.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id
    });
    document.trailer.set("Root", Object::Reference(catalog_id));
    let mut output = Vec::new();
    document
        .save_to(&mut output)
        .map_err(|_| "No se pudo generar el PDF.".to_string())?;
    Ok(output)
}

fn wrap_text(line: &str, max_characters: usize) -> Vec<String> {
    if line.is_empty() {
        return vec![String::new()];
    }
    let mut output = Vec::new();
    let mut current = String::new();
    for word in line.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if candidate.chars().count() > max_characters && !current.is_empty() {
            output.push(std::mem::take(&mut current));
            current = word.to_string();
        } else {
            current = candidate;
        }
        while current.chars().count() > max_characters {
            let split_at = current
                .char_indices()
                .nth(max_characters)
                .map(|(index, _)| index)
                .unwrap_or(current.len());
            output.push(current[..split_at].to_string());
            current = current[split_at..].to_string();
        }
    }
    if !current.is_empty() {
        output.push(current);
    }
    if output.is_empty() {
        vec![String::new()]
    } else {
        output
    }
}

fn win_ansi_bytes(value: &str) -> Vec<u8> {
    value
        .chars()
        .map(|character| {
            let code = character as u32;
            if code == 9 || code < 0x20 {
                b' '
            } else if code <= 0xff {
                code as u8
            } else {
                win_ansi_extra(code).unwrap_or(b'?')
            }
        })
        .collect()
}

fn win_ansi_extra(code: u32) -> Option<u8> {
    Some(match code {
        0x20ac => 0x80,
        0x201a => 0x82,
        0x0192 => 0x83,
        0x201e => 0x84,
        0x2026 => 0x85,
        0x2020 => 0x86,
        0x2021 => 0x87,
        0x02c6 => 0x88,
        0x2030 => 0x89,
        0x0160 => 0x8a,
        0x2039 => 0x8b,
        0x0152 => 0x8c,
        0x017d => 0x8e,
        0x2018 => 0x91,
        0x2019 => 0x92,
        0x201c => 0x93,
        0x201d => 0x94,
        0x2022 => 0x95,
        0x2013 => 0x96,
        0x2014 => 0x97,
        0x02dc => 0x98,
        0x2122 => 0x99,
        0x0161 => 0x9a,
        0x203a => 0x9b,
        0x0153 => 0x9c,
        0x017e => 0x9e,
        0x0178 => 0x9f,
        _ => return None,
    })
}

fn pdf_escape(bytes: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    for byte in bytes {
        if matches!(byte, b'\\' | b'(' | b')') {
            output.push(b'\\');
        }
        output.push(*byte);
    }
    output
}
