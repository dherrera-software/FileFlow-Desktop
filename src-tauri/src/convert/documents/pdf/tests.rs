//! Pruebas unitarias para generación de PDF.

use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, Rgb, RgbImage};

use super::writer::{blocks_to_pdf, image_dimensions, image_to_pdf, text_to_pdf};
use crate::convert::documents::{Block, BlockKind, BlockStyle, Run};

#[test]
fn text_pdf_stores_spanish_as_winansi_bytes() {
    let pdf = text_to_pdf("cañón (sur)").unwrap();
    assert!(pdf.starts_with(b"%PDF"));
    assert!(pdf.windows(6).any(|window| window == b"ca\xf1\xf3n "));
    assert!(pdf.windows(7).any(|window| window == br"\(sur\)"));
}

#[test]
fn block_pdf_wraps_long_text_and_uses_winansi_bytes() {
    let text = format!("cañón {}", "palabra ".repeat(60));
    let block = Block::from_runs(
        BlockKind::Paragraph,
        vec![Run::new(text, BlockStyle::default())],
        BlockStyle::default(),
    );
    let pdf = blocks_to_pdf(&[block]).unwrap();
    assert!(pdf.starts_with(b"%PDF"));
    assert!(pdf.windows(6).any(|window| window == b"ca\xf1\xf3n "));
    let positioned_lines = pdf.windows(4).filter(|window| *window == b" Tm\n").count();
    assert!(
        positioned_lines > 1,
        "el texto largo debe repartirse en varias líneas"
    );
}

#[test]
fn block_pdf_keeps_mixed_run_styles() {
    let mut bold = BlockStyle::default();
    bold.bold = true;
    let mut colored = BlockStyle::default();
    colored.color = Some("1E3A8A".to_string());
    let block = Block::from_runs(
        BlockKind::Paragraph,
        vec![Run::new("Negrita", bold), Run::new(" y azul", colored)],
        BlockStyle::default(),
    );
    let pdf = blocks_to_pdf(&[block]).unwrap();
    assert!(pdf.windows(4).any(|window| window == b"/F1 "));
    assert!(pdf.windows(4).any(|window| window == b"/F2 "));
    assert!(
        pdf.windows(20)
            .any(|window| window == b"0.118 0.227 0.541 rg")
    );
}

#[test]
fn block_pdf_handles_page_breaks_and_tables() {
    let b1 = Block::paragraph("Página 1");
    let b2 = Block::page_break();
    let b3 = Block::paragraph("Página 2");
    let table = Block::table(vec![
        vec!["Columna 1".to_string(), "Columna 2".to_string()],
        vec!["Dato A".to_string(), "Dato B".to_string()],
    ]);
    let pdf = blocks_to_pdf(&[b1, b2, b3, table]).unwrap();
    assert!(pdf.starts_with(b"%PDF"));
    let doc = lopdf::Document::load_mem(&pdf).unwrap();
    assert!(
        doc.get_pages().len() >= 2,
        "debe generar al menos 2 páginas"
    );
}

#[test]
fn unicode_mapping_does_not_corrupt_special_characters() {
    let text = "Viñeta: • guion: — flecha: → acento: łódź";
    let block = Block::paragraph(text);
    let pdf = blocks_to_pdf(&[block]).unwrap();
    assert!(pdf.starts_with(b"%PDF"));
    // La viñeta WinAnsi (0x95) debe estar presente
    assert!(pdf.windows(1).any(|w| w == [0x95]));
}

#[test]
fn png_becomes_a_pdf() {
    let mut bytes = Vec::new();
    let image = RgbImage::from_pixel(2, 2, Rgb([12, 80, 140]));
    PngEncoder::new(&mut bytes)
        .write_image(image.as_raw(), 2, 2, ExtendedColorType::Rgb8)
        .unwrap();
    let pdf = image_to_pdf(&bytes).unwrap();
    assert!(pdf.starts_with(b"%PDF"));
    assert_eq!(image_dimensions(&bytes).unwrap(), (2, 2));
}
