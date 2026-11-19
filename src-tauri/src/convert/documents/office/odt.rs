//! Conversión hacia y desde OpenDocument Text (.odt).

use quick_xml::Reader;
use quick_xml::events::Event;

use super::super::{Block, BlockKind};
use super::zip_utils::{
    attribute, build_zip_with_mimetype, named_entity, push_block, read_zip_entry, xml_escape,
};

pub fn read_odt(bytes: &[u8]) -> Result<Vec<Block>, String> {
    let xml = read_zip_entry(bytes, "content.xml", "ODT")?;
    let xml = String::from_utf8_lossy(&xml);
    Ok(parse_odt_xml(&xml))
}

pub fn write_odt(blocks: &[Block], title: &str) -> Result<Vec<u8>, String> {
    let mut body = String::new();
    for block in blocks {
        let (tag, extra) = match block.kind {
            BlockKind::Heading(level) => (
                "text:h",
                format!(r#" text:outline-level="{}""#, level.clamp(1, 6)),
            ),
            _ => ("text:p", String::new()),
        };
        let text = if block.kind == BlockKind::ListItem {
            format!("• {}", block.text)
        } else {
            block.text.clone()
        };
        let content = xml_escape(&text)
            .replace('\t', "<text:tab/>")
            .replace('\n', "<text:line-break/>");
        body.push_str(&format!(
            "<{tag}{extra} text:style-name=\"Standard\">{content}</{tag}>"
        ));
    }
    let content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" office:version="1.2">
<office:meta><dc:title xmlns:dc="http://purl.org/dc/elements/1.1/">{}</dc:title></office:meta>
<office:automatic-styles><style:style style:name="Standard" style:family="paragraph"/></office:automatic-styles>
<office:body><office:text>{body}</office:text></office:body>
</office:document-content>"#,
        xml_escape(title)
    );
    let manifest = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.2">
<manifest:file-entry manifest:full-path="/" manifest:media-type="application/vnd.oasis.opendocument.text"/>
<manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
</manifest:manifest>"#;
    build_zip_with_mimetype(
        "application/vnd.oasis.opendocument.text",
        &[
            ("content.xml", content.into_bytes()),
            ("META-INF/manifest.xml", manifest.as_bytes().to_vec()),
        ],
    )
}

fn parse_odt_xml(xml: &str) -> Vec<Block> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut kind = BlockKind::Paragraph;
    let mut in_block = false;
    while let Ok(event) = reader.read_event() {
        if matches!(event, Event::Eof) {
            break;
        }
        match event {
            Event::Start(start) => match start.local_name().as_ref() {
                "p" => {
                    in_block = true;
                    current.clear();
                    kind = BlockKind::Paragraph;
                }
                "h" => {
                    in_block = true;
                    current.clear();
                    let level = attribute(&start, "outline-level")
                        .and_then(|value| value.parse::<u8>().ok())
                        .unwrap_or(1);
                    kind = BlockKind::Heading(level);
                }
                "tab" | "s" => current.push(' '),
                "line-break" => current.push('\n'),
                _ => {}
            },
            Event::Empty(empty) => match empty.local_name().as_ref() {
                "tab" | "s" => current.push(' '),
                "line-break" => current.push('\n'),
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
            Event::End(end) if matches!(end.local_name().as_ref(), "p" | "h") && in_block => {
                push_block(&mut blocks, kind.clone(), &current);
                in_block = false;
            }
            _ => {}
        }
    }
    blocks
}
