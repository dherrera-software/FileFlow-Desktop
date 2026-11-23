//! Conversión hacia y desde OpenDocument Text (.odt).

use std::collections::HashMap;

use quick_xml::Reader;
use quick_xml::events::Event;

use super::super::{Alignment, Block, BlockKind, BlockStyle, Run};
use super::zip_utils::{
    attribute, build_zip_with_mimetype, named_entity, push_block_with_runs, read_zip_entry,
    xml_escape,
};

/// Recolecta las definiciones de estilo declaradas en `content.xml`.
/// Devuelve los estilos de párrafo y los estilos de texto (`text:span`) por
/// separado, porque en ODF el mismo nombre puede existir en ambas familias.
fn collect_odt_styles(xml: &str) -> (HashMap<String, BlockStyle>, HashMap<String, BlockStyle>) {
    let mut paragraphs = HashMap::new();
    let mut texts = HashMap::new();
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut current_name: Option<String> = None;
    let mut current_family = String::new();
    let mut current = BlockStyle::default();
    let mut in_style = false;

    while let Ok(event) = reader.read_event() {
        if matches!(event, Event::Eof) {
            break;
        }
        match event {
            Event::Start(start) => match start.local_name().as_ref() {
                "style" => {
                    if let Some(name) = attribute(&start, "name") {
                        let family = attribute(&start, "family").unwrap_or_default();
                        if family == "paragraph" || family == "text" {
                            current_name = Some(name);
                            current_family = family;
                            current = BlockStyle::default();
                            in_style = true;
                        }
                    }
                }
                "paragraph-properties" if in_style => {
                    if let Some(align) = attribute(&start, "text-align") {
                        current.align = match align.as_str() {
                            "center" => Alignment::Center,
                            "right" | "end" => Alignment::Right,
                            "justify" => Alignment::Justify,
                            _ => Alignment::Left,
                        };
                    }
                }
                "text-properties" if in_style => apply_odt_text_properties(&start, &mut current),
                _ => {}
            },
            Event::Empty(empty) => match empty.local_name().as_ref() {
                "paragraph-properties" if in_style => {
                    if let Some(align) = attribute(&empty, "text-align") {
                        current.align = match align.as_str() {
                            "center" => Alignment::Center,
                            "right" | "end" => Alignment::Right,
                            "justify" => Alignment::Justify,
                            _ => Alignment::Left,
                        };
                    }
                }
                "text-properties" if in_style => {
                    apply_odt_text_properties(&empty, &mut current);
                }
                _ => {}
            },
            Event::End(end) if end.local_name().as_ref() == "style" && in_style => {
                if let Some(name) = current_name.take() {
                    if current_family == "text" {
                        texts.insert(name, current.clone());
                    } else {
                        paragraphs.insert(name, current.clone());
                    }
                }
                in_style = false;
                current = BlockStyle::default();
            }
            _ => {}
        }
    }
    (paragraphs, texts)
}

fn apply_odt_text_properties(start: &quick_xml::events::BytesStart<'_>, style: &mut BlockStyle) {
    if let Some(weight) = attribute(start, "font-weight") {
        style.bold = weight == "bold";
    }
    if let Some(font_style) = attribute(start, "font-style") {
        style.italic = font_style == "italic";
    }
    if let Some(decoration) = attribute(start, "text-decoration-line") {
        style.underline = decoration.contains("underline");
    }
    if let Some(color) = attribute(start, "color") {
        style.color = Some(color);
    }
    if let Some(size) = attribute(start, "font-size") {
        let points = size.trim_end_matches("pt").parse::<f32>().unwrap_or(11.0);
        style.font_size = Some((points * 2.0) as u8);
    }
    if let Some(family) = attribute(start, "font-family") {
        style.font_family = Some(family.trim_matches('\'').to_string());
    }
}

pub fn read_odt(bytes: &[u8]) -> Result<Vec<Block>, String> {
    let xml = read_zip_entry(bytes, "content.xml", "ODT")?;
    let xml = String::from_utf8_lossy(&xml);
    let (paragraphs, texts) = collect_odt_styles(&xml);
    Ok(parse_odt_xml(&xml, &paragraphs, &texts))
}

fn register_style(
    styles: &mut HashMap<BlockStyle, String>,
    next_index: &mut usize,
    style: &BlockStyle,
    prefix: &str,
) -> String {
    if let Some(name) = styles.get(style) {
        return name.clone();
    }
    let name = format!("{prefix}{}", next_index);
    *next_index += 1;
    styles.insert(style.clone(), name.clone());
    name
}

fn odt_text_props(style: &BlockStyle) -> String {
    let mut props: Vec<String> = Vec::new();
    if let Some(size) = style.font_size {
        props.push(format!(r#"fo:font-size="{}pt""#, size as f32 / 2.0));
    }
    if style.bold {
        props.push(r#"fo:font-weight="bold""#.to_string());
    }
    if style.italic {
        props.push(r#"fo:font-style="italic""#.to_string());
    }
    if style.underline {
        props.push(r#"style:text-underline-style="solid""#.to_string());
    }
    if let Some(color) = &style.color {
        props.push(format!(r#"fo:color="{color}""#));
    }
    if let Some(family) = &style.font_family {
        props.push(format!(r#"fo:font-family="'{}'""#, xml_escape(family)));
    }
    props.join(" ")
}

fn odt_paragraph_props(style: &BlockStyle) -> String {
    match style.align {
        Alignment::Center => r#"fo:text-align="center""#.to_string(),
        Alignment::Right => r#"fo:text-align="right""#.to_string(),
        Alignment::Justify => r#"fo:text-align="justify""#.to_string(),
        Alignment::Left => r#"fo:text-align="left""#.to_string(),
    }
}

fn odt_escape(text: &str) -> String {
    xml_escape(text)
        .replace('\t', "<text:tab/>")
        .replace('\n', "<text:line-break/>")
}

pub fn write_odt(blocks: &[Block], title: &str) -> Result<Vec<u8>, String> {
    let mut body = String::new();
    let mut paragraph_styles: HashMap<BlockStyle, String> = HashMap::new();
    let mut text_styles: HashMap<BlockStyle, String> = HashMap::new();
    let mut next_paragraph = 1usize;
    let mut next_text = 1usize;

    for block in blocks {
        let style_attr = if block.style != BlockStyle::default() {
            let name = register_style(
                &mut paragraph_styles,
                &mut next_paragraph,
                &block.style,
                "P",
            );
            format!(r#" text:style-name="{name}""#)
        } else {
            String::new()
        };

        let (tag, extra) = match block.kind {
            BlockKind::Heading(level) => (
                "text:h",
                format!(r#" text:outline-level="{}"{style_attr}"#, level.clamp(1, 6)),
            ),
            _ => ("text:p", style_attr),
        };

        let mut runs = block.effective_runs();
        if block.kind == BlockKind::ListItem
            && let Some(first) = runs.first_mut()
        {
            first.text = format!("• {}", first.text);
        }

        let mut content = String::new();
        for run in &runs {
            let mut style = run.style.clone();
            style.align = Alignment::Left;
            let escaped = odt_escape(&run.text);
            if style == BlockStyle::default() {
                content.push_str(&escaped);
            } else {
                let name = register_style(&mut text_styles, &mut next_text, &style, "T");
                content.push_str(&format!(
                    r#"<text:span text:style-name="{name}">{escaped}</text:span>"#
                ));
            }
        }
        body.push_str(&format!("<{tag}{extra}>{content}</{tag}>"));
    }

    let mut automatic_styles =
        String::from(r#"<style:style style:name="Standard" style:family="paragraph"/>"#);
    for (style, name) in &paragraph_styles {
        if *style == BlockStyle::default() {
            continue;
        }
        let paragraph_props = odt_paragraph_props(style);
        let text_props = odt_text_props(style);
        let p_node = format!("<style:paragraph-properties {paragraph_props}/>");
        let t_node = if text_props.is_empty() {
            String::new()
        } else {
            format!("<style:text-properties {text_props}/>")
        };
        automatic_styles.push_str(&format!(
            r#"<style:style style:name="{name}" style:family="paragraph" style:parent-style-name="Standard">{p_node}{t_node}</style:style>"#
        ));
    }
    for (style, name) in &text_styles {
        let text_props = odt_text_props(style);
        if text_props.is_empty() {
            continue;
        }
        automatic_styles.push_str(&format!(
            r#"<style:style style:name="{name}" style:family="text"><style:text-properties {text_props}/></style:style>"#
        ));
    }

    let content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" office:version="1.2">
<office:meta><dc:title xmlns:dc="http://purl.org/dc/elements/1.1/">{}</dc:title></office:meta>
<office:automatic-styles>{automatic_styles}</office:automatic-styles>
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

fn flush_run(runs: &mut Vec<Run>, text: &mut String, style: &BlockStyle) {
    if text.is_empty() {
        return;
    }
    runs.push(Run::new(std::mem::take(text), style.clone()));
}

fn merge_text_style(base: &mut BlockStyle, defined: &BlockStyle) {
    if defined.bold {
        base.bold = true;
    }
    if defined.italic {
        base.italic = true;
    }
    if defined.underline {
        base.underline = true;
    }
    if defined.font_size.is_some() {
        base.font_size = defined.font_size;
    }
    if defined.color.is_some() {
        base.color = defined.color.clone();
    }
    if defined.font_family.is_some() {
        base.font_family = defined.font_family.clone();
    }
}

fn parse_odt_xml(
    xml: &str,
    paragraph_styles: &HashMap<String, BlockStyle>,
    text_styles: &HashMap<String, BlockStyle>,
) -> Vec<Block> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut blocks = Vec::new();
    let mut runs: Vec<Run> = Vec::new();
    let mut run_text = String::new();
    let mut style_stack: Vec<BlockStyle> = Vec::new();
    let mut kind = BlockKind::Paragraph;
    let mut paragraph_style = BlockStyle::default();
    let mut in_block = false;

    while let Ok(event) = reader.read_event() {
        if matches!(event, Event::Eof) {
            break;
        }
        match event {
            Event::Start(start) => match start.local_name().as_ref() {
                "p" | "h" => {
                    in_block = true;
                    runs.clear();
                    run_text.clear();
                    style_stack.clear();
                    paragraph_style = attribute(&start, "style-name")
                        .and_then(|name| paragraph_styles.get(&name))
                        .cloned()
                        .unwrap_or_default();
                    kind = if start.local_name().as_ref() == "h" {
                        let level = attribute(&start, "outline-level")
                            .and_then(|value| value.parse::<u8>().ok())
                            .unwrap_or(1);
                        BlockKind::Heading(level)
                    } else {
                        BlockKind::Paragraph
                    };
                    style_stack.push(paragraph_style.clone());
                }
                "span" if in_block => {
                    flush_run(
                        &mut runs,
                        &mut run_text,
                        style_stack.last().unwrap_or(&paragraph_style),
                    );
                    let mut style = style_stack
                        .last()
                        .cloned()
                        .unwrap_or_else(|| paragraph_style.clone());
                    if let Some(name) = attribute(&start, "style-name")
                        && let Some(defined) = text_styles.get(&name)
                    {
                        merge_text_style(&mut style, defined);
                    }
                    style_stack.push(style);
                }
                "ta" | "s" if in_block => {
                    let count = attribute(&start, "c")
                        .and_then(|value| value.parse::<usize>().ok())
                        .unwrap_or(1)
                        .min(64);
                    run_text.push_str(&" ".repeat(count));
                }
                "line-break" if in_block => run_text.push('\n'),
                _ => {}
            },
            Event::Empty(empty) => match empty.local_name().as_ref() {
                "ta" | "s" if in_block => {
                    let count = attribute(&empty, "c")
                        .and_then(|value| value.parse::<usize>().ok())
                        .unwrap_or(1)
                        .min(64);
                    run_text.push_str(&" ".repeat(count));
                }
                "line-break" if in_block => run_text.push('\n'),
                _ => {}
            },
            Event::Text(text) => {
                if in_block {
                    run_text.push_str(&text.xml_content(quick_xml::XmlVersion::Explicit1_0));
                }
            }
            Event::GeneralRef(reference) => {
                if in_block {
                    if let Ok(Some(character)) = reference.resolve_char_ref() {
                        run_text.push(character);
                    } else if let Some(character) = named_entity(&reference) {
                        run_text.push(character);
                    }
                }
            }
            Event::CData(data) => {
                if in_block {
                    run_text.push_str(data.as_ref());
                }
            }
            Event::End(end) => match end.local_name().as_ref() {
                "span" if in_block => {
                    flush_run(
                        &mut runs,
                        &mut run_text,
                        style_stack.last().unwrap_or(&paragraph_style),
                    );
                    if style_stack.len() > 1 {
                        style_stack.pop();
                    }
                }
                "p" | "h" if in_block => {
                    flush_run(
                        &mut runs,
                        &mut run_text,
                        style_stack.last().unwrap_or(&paragraph_style),
                    );
                    push_block_with_runs(
                        &mut blocks,
                        kind.clone(),
                        std::mem::take(&mut runs),
                        paragraph_style.clone(),
                    );
                    in_block = false;
                    style_stack.clear();
                }
                _ => {}
            },
            _ => {}
        }
    }
    blocks
}
