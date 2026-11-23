//! Conversión de documentos entre TXT, Markdown, HTML, DOCX, ODT, RTF, EPUB y PDF.

pub mod markup;
pub mod office;
pub mod pdf;
pub mod pdf_import;
pub mod text;
pub mod tts;
pub mod types;

pub use types::{Alignment, Block, BlockKind, BlockStyle, Run};

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

use crate::convert::sheets;
use crate::convert::support;

pub fn convert(bytes: &[u8], source: &str, target: &str, title: &str) -> Result<Vec<u8>, String> {
    if source == "DOC" {
        return convert_legacy_doc(bytes, target);
    }
    if source == "PDF" {
        match target {
            "JPG" | "PNG" => return pdf::render_first_page(bytes, target),
            "XLSX" | "CSV" => return pdf_table(bytes, target),
            _ => {}
        }
    }
    // Rutas con mayor fidelidad que usan el texto original directamente.
    let text = match source {
        "TXT" | "MD" | "HTML" => Some(support::decode_text(bytes).text),
        _ => None,
    };
    if let Some(text) = text.as_deref() {
        if source == "TXT" && target == "MP3" {
            return tts::text_to_mp3(text);
        }
        match (source, target) {
            ("TXT", "MD") => return Ok(text::txt_to_markdown(text).into_bytes()),
            ("TXT", "HTML") => return Ok(text::txt_to_html(text, title).into_bytes()),
            ("MD", "HTML") => return Ok(markup::markdown_to_html(text, title).into_bytes()),
            ("HTML", "MD") => {
                return Ok(markup::html_to_markdown(text)?.into_bytes());
            }
            _ => {}
        }
    }

    let blocks = read_blocks(bytes, source)?;
    if blocks.is_empty() {
        return Err("El documento no contiene texto que se pueda convertir.".to_string());
    }
    match target {
        "TXT" => Ok(blocks_to_text(&blocks).into_bytes()),
        "MD" => Ok(blocks_to_markdown(&blocks).into_bytes()),
        "HTML" => {
            let body = blocks_to_html_body(&blocks);
            Ok(text::html_document(title, &body, true).into_bytes())
        }
        "DOCX" => office::write_docx(&blocks, title),
        "ODT" => office::write_odt(&blocks, title),
        "RTF" => Ok(office::write_rtf(&blocks, title)),
        "EPUB" => office::write_epub(&blocks, title),
        "PDF" => pdf::blocks_to_pdf(&blocks),
        _ => Err(format!(
            "La conversión de {source} a {target} no está disponible."
        )),
    }
}

fn convert_legacy_doc(bytes: &[u8], target: &str) -> Result<Vec<u8>, String> {
    if crate::convert::external::soffice().is_none() {
        return Err(
            "Convertir DOC requiere LibreOffice instalado (o define FILEFLOW_SOFFICE).".to_string(),
        );
    }
    match target {
        "MD" => {
            let text = crate::convert::presentations::soffice_convert(bytes, "doc", "txt")?;
            let text = String::from_utf8_lossy(&text).into_owned();
            Ok(text::txt_to_markdown(&text).into_bytes())
        }
        "DOCX" | "ODT" | "RTF" | "TXT" | "HTML" | "PDF" | "EPUB" => {
            crate::convert::presentations::soffice_convert(
                bytes,
                "doc",
                &target.to_ascii_lowercase(),
            )
        }
        _ => Err(format!(
            "La conversión de DOC a {target} no está disponible."
        )),
    }
}

fn pdf_table(bytes: &[u8], target: &str) -> Result<Vec<u8>, String> {
    let text = pdf_import::extract_pdf_raw_text(bytes)?;
    let rows = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(split_pdf_line)
        .filter(|row| !row.is_empty())
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return Err("El PDF no contiene filas de texto que se puedan tabular.".to_string());
    }
    sheets::convert_rows(rows, target)
}

fn split_pdf_line(line: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut spaces = 0usize;
    let push = |cells: &mut Vec<String>, current: &mut String| {
        let value = current.trim().to_string();
        if !value.is_empty() {
            cells.push(value);
        }
        current.clear();
    };
    for character in line.chars() {
        match character {
            '\t' | ',' => {
                push(&mut cells, &mut current);
                spaces = 0;
            }
            ' ' => {
                spaces += 1;
                if spaces >= 2 {
                    push(&mut cells, &mut current);
                }
            }
            _ => {
                if spaces == 1 && !current.is_empty() {
                    current.push(' ');
                }
                spaces = 0;
                current.push(character);
            }
        }
    }
    push(&mut cells, &mut current);
    cells
}

fn read_blocks(bytes: &[u8], source: &str) -> Result<Vec<Block>, String> {
    match source {
        "TXT" => Ok(plain_text_blocks(&support::decode_text(bytes).text)),
        "MD" => Ok(markdown_to_blocks(&support::decode_text(bytes).text)),
        "HTML" => {
            let html = support::decode_text(bytes).text;
            let markdown = markup::html_to_markdown(&html)?;
            Ok(markdown_to_blocks(&markdown))
        }
        "DOCX" => office::read_docx(bytes),
        "ODT" => office::read_odt(bytes),
        "RTF" => office::read_rtf(bytes),
        "EPUB" => office::read_epub(bytes),
        "PDF" => pdf_import::read_pdf_text(bytes),
        _ => Err(format!("La conversión desde {source} no está disponible.")),
    }
}

pub fn plain_text_blocks(text: &str) -> Vec<Block> {
    let normalized = text::normalize_newlines(text)
        .trim_start_matches('\u{feff}')
        .to_string();
    let mut blocks = Vec::new();
    let mut current = String::new();
    for line in normalized.lines() {
        if line.trim().is_empty() {
            if !current.trim().is_empty() {
                blocks.push(Block::paragraph(current.trim().to_string()));
                current.clear();
            }
        } else {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line.trim_end());
        }
    }
    if !current.trim().is_empty() {
        blocks.push(Block::paragraph(current.trim().to_string()));
    }
    if blocks.is_empty() && !normalized.trim().is_empty() {
        blocks.push(Block::paragraph(normalized.trim().to_string()));
    }
    blocks
}

pub fn markdown_to_blocks(markdown: &str) -> Vec<Block> {
    let normalized = text::normalize_newlines(markdown);
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut kind = BlockKind::Paragraph;
    let mut in_code = false;
    for event in Parser::new_ext(&normalized, options) {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    current.clear();
                    kind = BlockKind::Heading(level as u8);
                }
                Tag::Item => {
                    current.clear();
                    kind = BlockKind::ListItem;
                }
                Tag::BlockQuote(_) => {
                    current.clear();
                    kind = BlockKind::Quote;
                }
                Tag::CodeBlock(_) => {
                    current.clear();
                    kind = BlockKind::Code;
                    in_code = true;
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) | TagEnd::Paragraph | TagEnd::Item | TagEnd::CodeBlock => {
                    let text = current.trim().to_string();
                    if !text.is_empty() {
                        blocks.push(Block {
                            kind: kind.clone(),
                            text,
                            style: BlockStyle::default(),
                            runs: Vec::new(),
                        });
                    }
                    current.clear();
                    if matches!(tag_end, TagEnd::Heading(_) | TagEnd::CodeBlock) {
                        kind = BlockKind::Paragraph;
                        in_code = false;
                    }
                }
                TagEnd::BlockQuote(_) => {
                    let text = current.trim().to_string();
                    if !text.is_empty() {
                        blocks.push(Block {
                            kind: BlockKind::Quote,
                            text,
                            style: BlockStyle::default(),
                            runs: Vec::new(),
                        });
                    }
                    current.clear();
                    kind = BlockKind::Paragraph;
                }
                _ => {}
            },
            Event::Text(value) => current.push_str(&value),
            Event::Code(value) => current.push_str(&value),
            Event::SoftBreak => {
                if in_code {
                    current.push('\n');
                } else {
                    current.push(' ');
                }
            }
            Event::HardBreak => current.push('\n'),
            _ => {}
        }
    }
    blocks
}

pub fn blocks_to_text(blocks: &[Block]) -> String {
    let mut output = blocks
        .iter()
        .map(|block| match &block.kind {
            BlockKind::Table(rows) => rows
                .iter()
                .map(|r| r.join("\t"))
                .collect::<Vec<_>>()
                .join("\n"),
            BlockKind::PageBreak => "\n---\n".to_string(),
            _ => block.text.trim().to_string(),
        })
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    output.push('\n');
    output
}

fn run_to_markdown(run: &Run, base: &BlockStyle) -> String {
    let mut out = run.text.clone();
    if run.style.italic && !base.italic {
        out = format!("*{out}*");
    }
    if run.style.bold && !base.bold {
        out = format!("**{out}**");
    }
    out
}

fn runs_to_markdown(block: &Block) -> String {
    block
        .effective_runs()
        .iter()
        .map(|run| run_to_markdown(run, &block.style))
        .collect::<String>()
}

pub fn blocks_to_markdown(blocks: &[Block]) -> String {
    let mut lines = Vec::new();
    for block in blocks {
        match &block.kind {
            BlockKind::PageBreak => lines.push("---\n".to_string()),
            BlockKind::Table(rows) => {
                if !rows.is_empty() {
                    let header = rows[0].join(" | ");
                    let sep = rows[0]
                        .iter()
                        .map(|c| "-".repeat(c.len().max(3)))
                        .collect::<Vec<_>>()
                        .join(" | ");
                    let mut tbl = vec![format!("| {header} |"), format!("| {sep} |")];
                    for row in &rows[1..] {
                        tbl.push(format!("| {} |", row.join(" | ")));
                    }
                    lines.push(tbl.join("\n"));
                }
            }
            BlockKind::Heading(level) => lines.push(format!(
                "{} {}",
                "#".repeat((*level).clamp(1, 6) as usize),
                runs_to_markdown(block)
            )),
            BlockKind::ListItem => {
                lines.push(format!("- {}", runs_to_markdown(block).replace('\n', " ")))
            }
            BlockKind::Quote => lines.push(
                runs_to_markdown(block)
                    .lines()
                    .map(|line| format!("> {line}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            BlockKind::Code => lines.push(format!("```\n{}\n```", runs_to_markdown(block))),
            BlockKind::Paragraph => lines.push(runs_to_markdown(block)),
        }
    }
    format!("{}\n", lines.join("\n\n"))
}

fn css_color(value: &str) -> Option<String> {
    let hex = value.trim().trim_start_matches('#');
    if hex.len() == 6 && hex.chars().all(|character| character.is_ascii_hexdigit()) {
        Some(format!("#{hex}"))
    } else {
        None
    }
}

fn text_style_parts(style: &BlockStyle) -> Vec<String> {
    let mut parts = Vec::new();
    if style.bold {
        parts.push("font-weight:bold".to_string());
    }
    if style.italic {
        parts.push("font-style:italic".to_string());
    }
    if style.underline {
        parts.push("text-decoration:underline".to_string());
    }
    if let Some(size) = style.font_size {
        let points = size as f32 / 2.0;
        parts.push(format!("font-size:{points}pt"));
    }
    if let Some(color) = style.color.as_deref().and_then(css_color) {
        parts.push(format!("color:{color}"));
    }
    if let Some(family) = &style.font_family {
        parts.push(format!("font-family:'{}'", text::escape_html(family)));
    }
    parts
}

fn css_attr(parts: &[String]) -> String {
    if parts.is_empty() {
        String::new()
    } else {
        format!("style=\"{}\"", parts.join(";"))
    }
}

fn style_to_css(style: &BlockStyle) -> String {
    let mut parts = text_style_parts(style);
    match style.align {
        Alignment::Center => parts.push("text-align:center".to_string()),
        Alignment::Right => parts.push("text-align:right".to_string()),
        Alignment::Justify => parts.push("text-align:justify".to_string()),
        Alignment::Left => {}
    }
    css_attr(&parts)
}

fn runs_to_html(block: &Block) -> String {
    if block.runs.is_empty() {
        return text::escape_html(&block.text).replace('\n', "<br>");
    }
    let mut output = String::new();
    for run in &block.runs {
        let content = text::escape_html(&run.text).replace('\n', "<br>");
        let css = css_attr(&text_style_parts(&run.style));
        if css.is_empty() {
            output.push_str(&content);
        } else {
            output.push_str(&format!("<span {css}>{content}</span>"));
        }
    }
    output
}

pub fn blocks_to_html_body(blocks: &[Block]) -> String {
    let mut body = String::new();
    for block in blocks {
        let style_attr = style_to_css(&block.style);
        let content = runs_to_html(block);
        match &block.kind {
            BlockKind::PageBreak => {
                body.push_str("<hr class=\"page-break\" style=\"page-break-after:always;border:none;margin:2rem 0;border-top:1px dashed #ccc\"/>\n");
            }
            BlockKind::Table(rows) => {
                body.push_str(
                    "<table style=\"border-collapse:collapse;width:100%;margin:1rem 0;\">\n",
                );
                for (row_idx, row) in rows.iter().enumerate() {
                    body.push_str("<tr>");
                    let tag = if row_idx == 0 { "th" } else { "td" };
                    for cell in row {
                        body.push_str(&format!(
                            "<{tag} style=\"border:1px solid #ccc;padding:0.4rem 0.6rem\">{}</{tag}>",
                            text::escape_html(cell)
                        ));
                    }
                    body.push_str("</tr>\n");
                }
                body.push_str("</table>\n");
            }
            BlockKind::Heading(level) => {
                let lvl = (*level).clamp(1, 6);
                if style_attr.is_empty() {
                    body.push_str(&format!("<h{lvl}>{content}</h{lvl}>\n"));
                } else {
                    body.push_str(&format!("<h{lvl} {style_attr}>{content}</h{lvl}>\n"));
                }
            }
            BlockKind::ListItem => {
                if style_attr.is_empty() {
                    body.push_str(&format!("<ul><li>{content}</li></ul>\n"));
                } else {
                    body.push_str(&format!("<ul><li {style_attr}>{content}</li></ul>\n"));
                }
            }
            BlockKind::Quote => {
                if style_attr.is_empty() {
                    body.push_str(&format!("<blockquote>{content}</blockquote>\n"));
                } else {
                    body.push_str(&format!(
                        "<blockquote {style_attr}>{content}</blockquote>\n"
                    ));
                }
            }
            BlockKind::Code => body.push_str(&format!(
                "<pre><code {style_attr}>{}</code></pre>\n",
                text::escape_html(&block.text)
            )),
            BlockKind::Paragraph => {
                if style_attr.is_empty() {
                    body.push_str(&format!("<p>{content}</p>\n"));
                } else {
                    body.push_str(&format!("<p {style_attr}>{content}</p>\n"));
                }
            }
        }
    }
    body.trim().to_string()
}

#[cfg(test)]
mod tests;
