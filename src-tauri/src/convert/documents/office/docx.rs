//! Lectura y escritura de documentos DOCX.

use quick_xml::Reader;
use quick_xml::events::Event;

use super::super::{Block, BlockKind};
use super::zip_utils::{
    attribute, build_zip, heading_level, named_entity, push_block, read_zip_entry, xml_escape,
};

pub fn read_docx(bytes: &[u8]) -> Result<Vec<Block>, String> {
    let xml = read_zip_entry(bytes, "word/document.xml", "DOCX")?;
    let xml = String::from_utf8_lossy(&xml);
    Ok(parse_word_xml(&xml))
}

pub fn write_docx(blocks: &[Block], title: &str) -> Result<Vec<u8>, String> {
    let mut body = String::new();
    for block in blocks {
        body.push_str(&docx_paragraph(block));
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
<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:pPr><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:sz w:val="32"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/><w:pPr><w:outlineLvl w:val="1"/></w:pPr><w:rPr><w:b/><w:sz w:val="26"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading3"><w:name w:val="heading 3"/><w:basedOn w:val="Normal"/><w:pPr><w:outlineLvl w:val="2"/></w:pPr><w:rPr><w:b/><w:sz w:val="24"/></w:rPr></w:style>
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

fn docx_paragraph(block: &Block) -> String {
    let style = match block.kind {
        BlockKind::Heading(level) => format!(
            r#"<w:pPr><w:pStyle w:val="Heading{}"/></w:pPr>"#,
            level.clamp(1, 3)
        ),
        BlockKind::Quote => r#"<w:pPr><w:ind w:left="720"/></w:pPr>"#.to_string(),
        BlockKind::ListItem => r#"<w:pPr><w:ind w:left="720"/></w:pPr>"#.to_string(),
        BlockKind::Code => r#"<w:pPr><w:shd w:val="clear" w:fill="F2F2F2"/></w:pPr>"#.to_string(),
        BlockKind::Paragraph => String::new(),
    };
    let text = if block.kind == BlockKind::ListItem {
        format!("• {}", block.text)
    } else {
        block.text.clone()
    };
    format!(
        "<w:p>{style}<w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
        xml_escape(&text).replace('\n', "</w:t></w:r><w:r><w:br/><w:t xml:space=\"preserve\">")
    )
}

fn parse_word_xml(xml: &str) -> Vec<Block> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut kind = BlockKind::Paragraph;
    let mut in_paragraph = false;
    while let Ok(event) = reader.read_event() {
        if matches!(event, Event::Eof) {
            break;
        }
        match event {
            Event::Start(start) => match start.local_name().as_ref() {
                "p" => {
                    in_paragraph = true;
                    current.clear();
                    kind = BlockKind::Paragraph;
                }
                "pStyle" => {
                    if let Some(value) = attribute(&start, "val")
                        && let Some(level) = heading_level(&value)
                    {
                        kind = BlockKind::Heading(level);
                    }
                }
                "tab" => current.push('\t'),
                "br" => current.push('\n'),
                _ => {}
            },
            Event::Empty(empty) => match empty.local_name().as_ref() {
                "pStyle" => {
                    if let Some(value) = attribute(&empty, "val")
                        && let Some(level) = heading_level(&value)
                    {
                        kind = BlockKind::Heading(level);
                    }
                }
                "tab" => current.push('\t'),
                "br" => current.push('\n'),
                _ => {}
            },
            Event::Text(text) => {
                current.push_str(&text.xml_content(quick_xml::XmlVersion::Explicit1_0))
            }
            Event::GeneralRef(reference) => {
                if let Ok(Some(character)) = reference.resolve_char_ref() {
                    current.push(character);
                } else if let Some(character) = named_entity(&reference) {
                    current.push(character);
                }
            }
            Event::CData(data) => current.push_str(data.as_ref()),
            Event::End(end) if end.local_name().as_ref() == "p" && in_paragraph => {
                push_block(&mut blocks, kind.clone(), &current);
                in_paragraph = false;
            }
            _ => {}
        }
    }
    blocks
}
