//! Generación de documentos DOCX nativos con tablas y saltos de página.

use crate::convert::documents::office::zip_utils::{build_zip, xml_escape};
use crate::convert::documents::{Alignment, Block, BlockKind, BlockStyle};

pub fn write_docx(blocks: &[Block], title: &str) -> Result<Vec<u8>, String> {
    let mut body = String::new();
    for block in blocks {
        match &block.kind {
            BlockKind::PageBreak => {
                body.push_str(r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#);
            }
            BlockKind::Table(rows) => {
                body.push_str(&docx_table(rows));
            }
            _ => {
                body.push_str(&docx_paragraph(block));
            }
        }
    }
    if body.is_empty() {
        body.push_str(&docx_paragraph(&Block::paragraph("")));
    }
    let document = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<w:body>{body}<w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1418" w:right="1418" w:bottom="1418" w:left="1418"/></w:sectPr></w:body>
</w:document>"#
    );
    let styles = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:docDefaults>
<w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri"/><w:sz w:val="22"/><w:szCs w:val="22"/></w:rPr></w:rPrDefault>
<w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="240" w:lineRule="auto"/></w:pPr></w:pPrDefault>
</w:docDefaults>
<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/></w:style>
<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:pPr><w:outlineLvl w:val="0"/><w:spacing w:before="240" w:after="120"/></w:pPr><w:rPr><w:b/><w:sz w:val="32"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/><w:pPr><w:outlineLvl w:val="1"/><w:spacing w:before="200" w:after="80"/></w:pPr><w:rPr><w:b/><w:sz w:val="26"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading3"><w:name w:val="heading 3"/><w:basedOn w:val="Normal"/><w:pPr><w:outlineLvl w:val="2"/><w:spacing w:before="160" w:after="60"/></w:pPr><w:rPr><w:b/><w:sz w:val="24"/></w:rPr></w:style>
</w:styles>"#;
    let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
</Types>"#;
    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
</Relationships>"#;
    let document_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;
    let core = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/">
<dc:title>{}</dc:title><dc:creator>FileFlow</dc:creator>
</cp:coreProperties>"#,
        xml_escape(title)
    );
    build_zip(&[
        ("[Content_Types].xml", content_types.as_bytes().to_vec()),
        ("_rels/.rels", rels.as_bytes().to_vec()),
        ("docProps/core.xml", core.into_bytes()),
        ("word/document.xml", document.into_bytes()),
        ("word/styles.xml", styles.as_bytes().to_vec()),
        (
            "word/_rels/document.xml.rels",
            document_rels.as_bytes().to_vec(),
        ),
    ])
}

pub fn docx_table(rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    out.push_str(
        r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/><w:tblBorders><w:top w:val="single" w:sz="4" w:space="0" w:color="D1D5DB"/><w:left w:val="none"/><w:bottom w:val="single" w:sz="4" w:space="0" w:color="D1D5DB"/><w:right w:val="none"/><w:insideH w:val="single" w:sz="4" w:space="0" w:color="E5E7EB"/><w:insideV w:val="none"/></w:tblBorders></w:tblPr>"#,
    );
    for (row_idx, row) in rows.iter().enumerate() {
        out.push_str("<w:tr>");
        for cell in row {
            let is_header = row_idx == 0;
            let tc_pr = if is_header {
                r#"<w:tcPr><w:shd w:val="clear" w:fill="F3F4F6"/></w:tcPr>"#
            } else {
                ""
            };
            let b_tag = if is_header { "<w:b/>" } else { "" };
            let escaped = xml_escape(cell);
            out.push_str(&format!(
                r#"<w:tc>{tc_pr}<w:p><w:pPr><w:spacing w:after="60" w:line="240" w:lineRule="auto"/></w:pPr><w:r><w:rPr>{b_tag}</w:rPr><w:t xml:space="preserve">{escaped}</w:t></w:r></w:p></w:tc>"#
            ));
        }
        out.push_str("</w:tr>");
    }
    out.push_str("</w:tbl>");
    out
}

pub fn docx_paragraph(block: &Block) -> String {
    let mut p_parts: Vec<String> = Vec::new();
    match block.kind {
        BlockKind::Heading(level) => p_parts.push(format!(
            r#"<w:pStyle w:val="Heading{}"/>"#,
            level.clamp(1, 3)
        )),
        BlockKind::Quote => p_parts.push(r#"<w:ind w:left="720"/>"#.to_string()),
        BlockKind::ListItem => p_parts.push(r#"<w:ind w:left="720"/>"#.to_string()),
        BlockKind::Code => p_parts.push(r#"<w:shd w:val="clear" w:fill="F2F2F2"/>"#.to_string()),
        BlockKind::Paragraph | BlockKind::PageBreak | BlockKind::Table(_) => {}
    }
    match block.style.align {
        Alignment::Center => p_parts.push(r#"<w:jc w:val="center"/>"#.to_string()),
        Alignment::Right => p_parts.push(r#"<w:jc w:val="right"/>"#.to_string()),
        Alignment::Justify => p_parts.push(r#"<w:jc w:val="both"/>"#.to_string()),
        Alignment::Left => {}
    }
    let p_pr = if p_parts.is_empty() {
        String::new()
    } else {
        format!("<w:pPr>{}</w:pPr>", p_parts.concat())
    };

    let mut runs = block.effective_runs();
    if block.kind == BlockKind::ListItem
        && let Some(first) = runs.first_mut()
    {
        first.text = format!("• {}", first.text);
    }

    let mut content = String::new();
    for run in &runs {
        content.push_str(&docx_run(&run.text, &run.style));
    }
    if content.is_empty() {
        content.push_str(r#"<w:r><w:t xml:space="preserve"></w:t></w:r>"#);
    }
    format!("<w:p>{p_pr}{content}</w:p>")
}

pub fn docx_run(text: &str, style: &BlockStyle) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut inner = String::new();
    for (line_index, line) in text.split('\n').enumerate() {
        if line_index > 0 {
            inner.push_str("<w:br/>");
        }
        for (tab_index, part) in line.split('\t').enumerate() {
            if tab_index > 0 {
                inner.push_str("<w:tab/>");
            }
            if !part.is_empty() {
                inner.push_str(&format!(
                    r#"<w:t xml:space="preserve">{}</w:t>"#,
                    xml_escape(part)
                ));
            }
        }
    }
    let r_pr = run_properties(style);
    format!("<w:r>{r_pr}{inner}</w:r>")
}

pub fn run_properties(style: &BlockStyle) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(family) = &style.font_family {
        let family = xml_escape(family);
        parts.push(format!(
            r#"<w:rFonts w:ascii="{family}" w:hAnsi="{family}"/>"#
        ));
    }
    if style.bold {
        parts.push("<w:b/>".to_string());
    }
    if style.italic {
        parts.push("<w:i/>".to_string());
    }
    if let Some(color) = &style.color {
        let hex = color.trim().trim_start_matches('#');
        if hex.len() == 6 && hex.chars().all(|character| character.is_ascii_hexdigit()) {
            parts.push(format!(r#"<w:color w:val="{hex}"/>"#));
        }
    }
    if let Some(size) = style.font_size {
        parts.push(format!(r#"<w:sz w:val="{size}"/><w:szCs w:val="{size}"/>"#));
    }
    if style.underline {
        parts.push(r#"<w:u w:val="single"/>"#.to_string());
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("<w:rPr>{}</w:rPr>", parts.join(""))
    }
}
