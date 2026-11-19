//! Pruebas unitarias para documentos de oficina (DOCX, ODT, RTF, EPUB).

use super::super::{Block, BlockKind};
use super::docx::{read_docx, write_docx};
use super::epub::{read_epub, write_epub};
use super::odt::{read_odt, write_odt};
use super::rtf::{read_rtf, write_rtf};

#[test]
fn docx_round_trip_keeps_paragraphs() {
    let blocks = vec![
        Block::heading(1, "Título"),
        Block::paragraph("Primera línea\nSegunda línea"),
    ];
    let bytes = write_docx(&blocks, "Doc").unwrap();
    assert!(bytes.starts_with(b"PK"));
    let read = read_docx(&bytes).unwrap();
    assert!(read.iter().any(|block| block.text.contains("Título")));
    assert!(
        read.iter()
            .any(|block| block.text.contains("Segunda línea"))
    );
}

#[test]
fn odt_round_trip_keeps_headings() {
    let blocks = vec![Block::heading(2, "Sección"), Block::paragraph("Contenido")];
    let bytes = write_odt(&blocks, "Doc").unwrap();
    let read = read_odt(&bytes).unwrap();
    assert!(read.iter().any(|block| block.kind == BlockKind::Heading(2)));
    assert!(read.iter().any(|block| block.text == "Contenido"));
}

#[test]
fn rtf_round_trip_keeps_accents() {
    let blocks = vec![Block::paragraph("cañón y acción")];
    let bytes = write_rtf(&blocks, "Doc");
    let read = read_rtf(&bytes).unwrap();
    assert_eq!(read[0].text, "cañón y acción");
}

#[test]
fn epub_round_trip_keeps_text() {
    let blocks = vec![
        Block::heading(1, "Capítulo"),
        Block::paragraph("Hola mundo"),
    ];
    let bytes = write_epub(&blocks, "Libro").unwrap();
    let read = read_epub(&bytes).unwrap();
    assert!(read.iter().any(|block| block.text.contains("Hola mundo")));
}
