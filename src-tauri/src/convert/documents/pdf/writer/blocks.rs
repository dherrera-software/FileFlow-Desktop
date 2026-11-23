//! Generación de páginas PDF a partir de bloques semánticos y tablas.

use lopdf::{Document, Object, ObjectId, Stream, dictionary};

use super::encoding::{pdf_escape, win_ansi_bytes};
use super::layout::{LineLayout, layout_block, line_font_size, render_line};
use crate::convert::documents::{Block, BlockKind};

pub const PAGE_WIDTH: f32 = 595.28;
pub const PAGE_HEIGHT: f32 = 841.89;

pub fn blocks_to_pdf(blocks: &[Block]) -> Result<Vec<u8>, String> {
    if blocks.is_empty() {
        return super::basic::text_to_pdf("");
    }

    let margin = 56.7_f32;
    let available = PAGE_WIDTH - margin * 2.0;
    let fallback_size = 11.0_f32;

    let mut document = Document::with_version("1.4");
    let font_normal = add_base_font(&mut document, "Helvetica");
    let font_bold = add_base_font(&mut document, "Helvetica-Bold");
    let font_italic = add_base_font(&mut document, "Helvetica-Oblique");
    let font_bold_italic = add_base_font(&mut document, "Helvetica-BoldOblique");

    let mut pages: Vec<Vec<u8>> = Vec::new();
    let mut content: Vec<u8> = Vec::new();
    let mut y = PAGE_HEIGHT - margin;

    for block in blocks {
        match &block.kind {
            BlockKind::PageBreak => {
                if !content.is_empty() {
                    pages.push(std::mem::take(&mut content));
                }
                y = PAGE_HEIGHT - margin;
                continue;
            }
            BlockKind::Table(rows) => {
                render_pdf_table(&mut content, &mut pages, &mut y, rows, margin, available);
                continue;
            }
            _ => {}
        }

        let heading = matches!(block.kind, BlockKind::Heading(_));
        let base_size = block
            .style
            .font_size
            .map(|value| value as f32 / 2.0)
            .unwrap_or_else(|| match block.kind {
                BlockKind::Heading(1) => 18.0,
                BlockKind::Heading(2) => 15.5,
                BlockKind::Heading(3) => 13.5,
                BlockKind::Heading(_) => 12.5,
                BlockKind::Code => 10.0,
                _ => fallback_size,
            });
        let lines = layout_block(&block.effective_runs(), base_size, available);
        if lines.is_empty() {
            continue;
        }

        let gap = if heading {
            base_size * 0.75
        } else {
            base_size * 0.45
        };
        for (index, line) in lines.iter().enumerate() {
            let size = line_font_size(line, base_size);
            let line_height = (size * 1.35).max(12.0);
            if y - line_height < margin {
                if !content.is_empty() {
                    pages.push(std::mem::take(&mut content));
                }
                y = PAGE_HEIGHT - margin;
            }
            y -= line_height;
            render_line(
                &mut content,
                line,
                &LineLayout {
                    align: block.style.align,
                    y,
                    available,
                    margin,
                    default_size: base_size,
                    force_bold: heading,
                    last_line: index + 1 == lines.len(),
                },
            );
        }
        y -= gap;
    }

    if !content.is_empty() || pages.is_empty() {
        pages.push(content);
    }

    let mut page_ids: Vec<ObjectId> = Vec::new();
    for page_content in pages {
        let content_id = document.add_object(Stream::new(dictionary! {}, page_content));
        let page_id = add_page(
            &mut document,
            font_normal,
            font_bold,
            font_italic,
            font_bold_italic,
            content_id,
        );
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

fn render_pdf_table(
    content: &mut Vec<u8>,
    pages: &mut Vec<Vec<u8>>,
    y: &mut f32,
    rows: &[Vec<String>],
    margin: f32,
    available: f32,
) {
    if rows.is_empty() {
        return;
    }
    let cols = rows.iter().map(Vec::len).max().unwrap_or(1).max(1);
    let col_w = available / cols as f32;
    let font_size = 9.5_f32;
    let row_h = 18.0_f32;

    for (row_idx, row) in rows.iter().enumerate() {
        if *y - row_h < margin {
            if !content.is_empty() {
                pages.push(std::mem::take(content));
            }
            *y = PAGE_HEIGHT - margin;
        }
        let is_header = row_idx == 0;
        let font = if is_header { "F2" } else { "F1" };

        // Fondo para encabezado
        if is_header {
            content.extend(
                format!(
                    "0.95 0.96 0.97 rg\n{:.2} {:.2} {:.2} {:.2} re f\n",
                    margin,
                    *y - row_h,
                    available,
                    row_h
                )
                .into_bytes(),
            );
        }
        // Borde inferior de fila
        content.extend(
            format!(
                "0.82 0.84 0.86 rg\n{:.2} {:.2} {:.2} 0.5 re f\n",
                margin,
                *y - row_h,
                available
            )
            .into_bytes(),
        );

        for (col_idx, cell) in row.iter().enumerate() {
            let x = margin + col_idx as f32 * col_w + 6.0;
            let text_y = *y - row_h + 5.0;
            let max_chars = (col_w / (font_size * 0.5)).floor().max(1.0) as usize;
            let truncated: String = cell.chars().take(max_chars).collect();
            content.extend(
                format!(
                    "BT\n/{} {:.1} Tf\n0.15 0.15 0.15 rg\n1 0 0 1 {:.2} {:.2} Tm\n",
                    font, font_size, x, text_y
                )
                .into_bytes(),
            );
            content.push(b'(');
            content.extend(pdf_escape(&win_ansi_bytes(&truncated)));
            content.extend(b") Tj\nET\n");
        }
        *y -= row_h;
    }
    *y -= 8.0; // Espaciado después de la tabla
}

pub fn add_base_font(document: &mut Document, base_font: &str) -> ObjectId {
    document.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => base_font,
        "Encoding" => "WinAnsiEncoding"
    })
}

pub fn add_page(
    document: &mut Document,
    font_normal: ObjectId,
    font_bold: ObjectId,
    font_italic: ObjectId,
    font_bold_italic: ObjectId,
    content_id: ObjectId,
) -> ObjectId {
    document.add_object(dictionary! {
        "Type" => "Page",
        "MediaBox" => media_box(),
        "Resources" => dictionary! {
            "Font" => dictionary! {
                "F1" => font_normal,
                "F2" => font_bold,
                "F3" => font_italic,
                "F4" => font_bold_italic,
            }
        },
        "Contents" => content_id
    })
}

pub fn media_box() -> Vec<Object> {
    vec![
        Object::Integer(0),
        Object::Integer(0),
        Object::Real(PAGE_WIDTH),
        Object::Real(PAGE_HEIGHT),
    ]
}

pub fn fix_page_tree(document: &mut Document, pages_id: ObjectId) {
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

pub fn finish_document(document: &mut Document, pages_id: ObjectId) -> Result<Vec<u8>, String> {
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
