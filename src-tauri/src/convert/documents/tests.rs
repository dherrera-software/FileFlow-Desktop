//! Pruebas unitarias para el módulo central de documentos.

use super::*;

#[test]
fn markdown_blocks_keep_headings_and_lists() {
    let blocks = markdown_to_blocks("# Titulo\n\nTexto **fuerte**\n\n- uno\n- dos\n");
    assert!(
        blocks
            .iter()
            .any(|block| block.kind == BlockKind::Heading(1))
    );
    assert!(blocks.iter().any(|block| block.text.contains("fuerte")));
    assert_eq!(
        blocks
            .iter()
            .filter(|block| block.kind == BlockKind::ListItem)
            .count(),
        2
    );
}

#[test]
fn document_round_trip_docx_to_markdown() {
    let source = "# Encabezado\n\nPrimer parrafo.\n\nSegundo parrafo.".as_bytes();
    let docx = convert(source, "MD", "DOCX", "Doc").unwrap();
    let markdown = convert(&docx, "DOCX", "MD", "Doc").unwrap();
    let markdown = String::from_utf8(markdown).unwrap();
    assert!(markdown.contains("# Encabezado"));
    assert!(markdown.contains("Segundo parrafo."));
}

#[test]
fn pdf_output_starts_with_pdf_header() {
    let pdf = convert(b"Hola mundo", "TXT", "PDF", "Nota").unwrap();
    assert!(pdf.starts_with(b"%PDF"));
}

#[test]
fn corrupt_zip_is_rejected_with_a_clear_message() {
    let error = convert(b"PK\x03\x04basura", "DOCX", "MD", "Doc").unwrap_err();
    assert!(error.contains("dañado") || error.contains("ZIP"));
}
