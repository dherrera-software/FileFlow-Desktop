//! Módulo DOCX: lectura y escritura modular de documentos Microsoft Word.

pub mod reader;
pub mod styles;
pub mod writer;

pub use reader::parse_word_xml;
pub use styles::Stylesheet;
pub use writer::write_docx;

use crate::convert::documents::Block;
use crate::convert::documents::office::zip_utils::read_zip_entry;

pub fn read_docx(bytes: &[u8]) -> Result<Vec<Block>, String> {
    let document = read_zip_entry(bytes, "word/document.xml", "DOCX")?;
    let styles = read_zip_entry(bytes, "word/styles.xml", "DOCX").unwrap_or_default();
    let document = String::from_utf8_lossy(&document);
    let styles = String::from_utf8_lossy(&styles);
    let sheet = Stylesheet::parse(&styles);
    Ok(parse_word_xml(&document, &sheet))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::documents::{BlockKind, BlockStyle, Run};

    fn document_xml(bytes: &[u8]) -> String {
        let xml = read_zip_entry(bytes, "word/document.xml", "DOCX").unwrap();
        String::from_utf8(xml).unwrap()
    }

    #[test]
    fn writer_does_not_emit_placeholder_braces() {
        let blocks = vec![Block::paragraph("Una línea\nOtra línea")];
        let bytes = write_docx(&blocks, "Doc").unwrap();
        let xml = document_xml(&bytes);
        assert!(!xml.contains("{r_pr}"));
        assert!(xml.contains("<w:br/>"));
    }

    #[test]
    fn writer_splits_runs_and_preserves_styles() {
        let mut bold = BlockStyle::default();
        bold.bold = true;
        let mut normal = BlockStyle::default();
        normal.italic = true;
        let block = Block::from_runs(
            BlockKind::Paragraph,
            vec![Run::new("Negrita", bold), Run::new(" normal", normal)],
            BlockStyle::default(),
        );
        let bytes = write_docx(&[block], "Doc").unwrap();
        let xml = document_xml(&bytes);
        assert!(xml.contains("<w:b/>"));
        assert!(xml.contains("<w:i/>"));
        assert!(xml.contains("Negrita"));
    }

    #[test]
    fn reader_resolves_localized_heading_styles() {
        let document = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>
<w:p><w:pPr><w:pStyle w:val="Ttulo1"/></w:pPr><w:r><w:t>Sección</w:t></w:r></w:p>
<w:p><w:r><w:rPr><w:b/><w:color w:val="334155"/><w:sz w:val="19"/></w:rPr><w:t>Texto</w:t></w:r></w:p>
</w:body></w:document>"#;
        let styles = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:style w:type="paragraph" w:styleId="Ttulo1"><w:name w:val="heading 1"/><w:pPr><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:sz w:val="36"/></w:rPr></w:style>
</w:styles>"#;
        let sheet = Stylesheet::parse(styles);
        let blocks = parse_word_xml(document, &sheet);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].kind, BlockKind::Heading(1));
        assert!(blocks[0].style.bold);
        assert_eq!(blocks[0].style.font_size, Some(36));
        assert_eq!(blocks[1].runs[0].style.font_size, Some(19));
        assert_eq!(blocks[1].runs[0].style.color.as_deref(), Some("334155"));
    }

    #[test]
    fn table_round_trip_preserves_cells() {
        let rows = vec![
            vec!["Col 1".to_string(), "Col 2".to_string()],
            vec!["Val A".to_string(), "Val B".to_string()],
        ];
        let block = Block::table(rows);
        let bytes = write_docx(&[block], "TablaDoc").unwrap();
        let read_blocks = read_docx(&bytes).unwrap();
        assert_eq!(read_blocks.len(), 1);
        if let BlockKind::Table(read_rows) = &read_blocks[0].kind {
            assert_eq!(read_rows.len(), 2);
            assert_eq!(read_rows[0][0], "Col 1");
            assert_eq!(read_rows[1][1], "Val B");
        } else {
            panic!("Expected BlockKind::Table");
        }
    }
}
