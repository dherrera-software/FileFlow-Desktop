//! Lector de documento XML para DOCX con soporte de tablas y saltos de página.

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use super::styles::{Stylesheet, alignment_from, apply_run_element};
use crate::convert::documents::office::zip_utils::{attribute, push_block_with_runs};
use crate::convert::documents::{Alignment, Block, BlockKind, BlockStyle, Run};

pub fn parse_word_xml(xml: &str, sheet: &Stylesheet) -> Vec<Block> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut blocks = Vec::new();

    let mut runs: Vec<Run> = Vec::new();
    let mut run_text = String::new();
    let mut run_style = BlockStyle::default();
    let mut in_run = false;
    let mut in_rpr = false;
    let mut in_text = false;
    let mut in_deleted = 0usize;

    let mut in_paragraph = false;
    let mut paragraph_style = sheet.style_for(None);
    let mut paragraph_align: Option<Alignment> = None;
    let mut paragraph_numbered = paragraph_style.numbered;

    // Manejo de tablas anidadas
    let mut in_table = false;
    let mut in_cell = false;
    let mut current_table: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut current_cell_text = String::new();

    while let Ok(event) = reader.read_event() {
        if matches!(event, Event::Eof) {
            break;
        }
        match event {
            Event::Start(start) => match start.local_name().as_ref() {
                "tbl" => {
                    in_table = true;
                    current_table.clear();
                }
                "tr" => {
                    current_row.clear();
                }
                "tc" => {
                    in_cell = true;
                    current_cell_text.clear();
                }
                "p" => {
                    in_paragraph = true;
                    runs.clear();
                    run_text.clear();
                    paragraph_style = sheet.style_for(None);
                    paragraph_align = None;
                    paragraph_numbered = paragraph_style.numbered;
                }
                "pStyle" => {
                    if in_paragraph
                        && !in_run
                        && let Some(id) = attribute(&start, "val")
                    {
                        paragraph_style = sheet.style_for(Some(&id));
                    }
                }
                "numPr" => {
                    if in_paragraph && !in_run {
                        paragraph_numbered = true;
                    }
                }
                "jc" => {
                    if in_paragraph
                        && !in_run
                        && let Some(value) = attribute(&start, "val")
                    {
                        paragraph_align = Some(alignment_from(&value));
                    }
                }
                "r" => {
                    in_run = true;
                    run_text.clear();
                    run_style = paragraph_style.style.clone();
                }
                "rPr" => {
                    if in_run {
                        in_rpr = true;
                    }
                }
                "rStyle" => {
                    if in_rpr && let Some(id) = attribute(&start, "val") {
                        sheet.resolved_patch(&id).apply(&mut run_style);
                    }
                }
                "t" => {
                    in_text = in_run;
                }
                "tab" | "ta" => {
                    if in_run && !in_rpr {
                        run_text.push('\t');
                    }
                }
                "br" => {
                    if in_run && !in_rpr {
                        if attribute(&start, "type").as_deref() == Some("page") {
                            // Salto de página explícito
                            if in_paragraph {
                                flush_paragraph(
                                    &mut blocks,
                                    &paragraph_style,
                                    paragraph_numbered,
                                    paragraph_align,
                                    std::mem::take(&mut runs),
                                );
                            }
                            blocks.push(Block::page_break());
                        } else {
                            run_text.push('\n');
                        }
                    }
                }
                "cr" => {
                    if in_run && !in_rpr {
                        run_text.push('\n');
                    }
                }
                "lastRenderedPageBreak" => {
                    if in_paragraph {
                        flush_paragraph(
                            &mut blocks,
                            &paragraph_style,
                            paragraph_numbered,
                            paragraph_align,
                            std::mem::take(&mut runs),
                        );
                    }
                    blocks.push(Block::page_break());
                }
                "noBreakHyphen" => {
                    if in_run && !in_rpr {
                        run_text.push('-');
                    }
                }
                "del" => in_deleted += 1,
                name => {
                    if in_rpr {
                        apply_run_property(name, &start, &mut run_style);
                    }
                }
            },
            Event::Empty(start) => match start.local_name().as_ref() {
                "pStyle" => {
                    if in_paragraph
                        && !in_run
                        && let Some(id) = attribute(&start, "val")
                    {
                        paragraph_style = sheet.style_for(Some(&id));
                    }
                }
                "numPr" => {
                    if in_paragraph && !in_run {
                        paragraph_numbered = true;
                    }
                }
                "jc" => {
                    if in_paragraph
                        && !in_run
                        && let Some(value) = attribute(&start, "val")
                    {
                        paragraph_align = Some(alignment_from(&value));
                    }
                }
                "rStyle" => {
                    if in_rpr && let Some(id) = attribute(&start, "val") {
                        sheet.resolved_patch(&id).apply(&mut run_style);
                    }
                }
                "tab" | "ta" => {
                    if in_run && !in_rpr {
                        run_text.push('\t');
                    }
                }
                "br" => {
                    if in_run && !in_rpr {
                        if attribute(&start, "type").as_deref() == Some("page") {
                            if in_paragraph {
                                flush_paragraph(
                                    &mut blocks,
                                    &paragraph_style,
                                    paragraph_numbered,
                                    paragraph_align,
                                    std::mem::take(&mut runs),
                                );
                            }
                            blocks.push(Block::page_break());
                        } else {
                            run_text.push('\n');
                        }
                    }
                }
                "cr" => {
                    if in_run && !in_rpr {
                        run_text.push('\n');
                    }
                }
                "lastRenderedPageBreak" => {
                    if in_paragraph {
                        flush_paragraph(
                            &mut blocks,
                            &paragraph_style,
                            paragraph_numbered,
                            paragraph_align,
                            std::mem::take(&mut runs),
                        );
                    }
                    blocks.push(Block::page_break());
                }
                "noBreakHyphen" => {
                    if in_run && !in_rpr {
                        run_text.push('-');
                    }
                }
                name => {
                    if in_rpr {
                        apply_run_property(name, &start, &mut run_style);
                    }
                }
            },
            Event::Text(text) => {
                if in_text && in_deleted == 0 {
                    let content = text.xml_content(quick_xml::XmlVersion::Explicit1_0);
                    run_text.push_str(&content);
                    if in_cell {
                        current_cell_text.push_str(&content);
                    }
                }
            }
            Event::GeneralRef(reference) => {
                if in_text && in_deleted == 0 {
                    if let Ok(Some(character)) = reference.resolve_char_ref() {
                        run_text.push(character);
                        if in_cell {
                            current_cell_text.push(character);
                        }
                    } else if let Some(character) =
                        crate::convert::documents::office::zip_utils::named_entity(&reference)
                    {
                        run_text.push(character);
                        if in_cell {
                            current_cell_text.push(character);
                        }
                    }
                }
            }
            Event::CData(data) => {
                if in_text && in_deleted == 0 {
                    run_text.push_str(data.as_ref());
                    if in_cell {
                        current_cell_text.push_str(data.as_ref());
                    }
                }
            }
            Event::End(end) => match end.local_name().as_ref() {
                "t" => in_text = false,
                "rPr" => in_rpr = false,
                "r" => {
                    in_run = false;
                    if !run_text.is_empty() {
                        runs.push(Run::new(std::mem::take(&mut run_text), run_style.clone()));
                    }
                }
                "del" => in_deleted = in_deleted.saturating_sub(1),
                "p" if in_paragraph => {
                    if !in_table {
                        flush_paragraph(
                            &mut blocks,
                            &paragraph_style,
                            paragraph_numbered,
                            paragraph_align,
                            std::mem::take(&mut runs),
                        );
                    }
                    in_paragraph = false;
                    paragraph_style = sheet.style_for(None);
                    paragraph_align = None;
                    paragraph_numbered = paragraph_style.numbered;
                }
                "tc" => {
                    in_cell = false;
                    current_row.push(std::mem::take(&mut current_cell_text).trim().to_string());
                }
                "tr" => {
                    if !current_row.is_empty() {
                        current_table.push(std::mem::take(&mut current_row));
                    }
                }
                "tbl" => {
                    in_table = false;
                    if !current_table.is_empty() {
                        blocks.push(Block::table(std::mem::take(&mut current_table)));
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    blocks
}

fn flush_paragraph(
    blocks: &mut Vec<Block>,
    paragraph_style: &super::styles::ResolvedStyle,
    paragraph_numbered: bool,
    paragraph_align: Option<Alignment>,
    runs: Vec<Run>,
) {
    if runs.is_empty() {
        return;
    }
    let kind = if let Some(level) = paragraph_style.heading {
        BlockKind::Heading(level)
    } else if paragraph_numbered {
        BlockKind::ListItem
    } else {
        BlockKind::Paragraph
    };
    let mut block_style = paragraph_style.style.clone();
    if let Some(align) = paragraph_align {
        block_style.align = align;
    }
    push_block_with_runs(blocks, kind, runs, block_style);
}

fn apply_run_property(name: &str, start: &BytesStart<'_>, style: &mut BlockStyle) {
    let mut patch = super::styles::StylePatch {
        bold: Some(style.bold),
        italic: Some(style.italic),
        underline: Some(style.underline),
        font_size: style.font_size,
        color: style.color.clone(),
        font_family: style.font_family.clone(),
        ..super::styles::StylePatch::default()
    };
    apply_run_element(name, start, &mut patch);
    patch.apply(style);
}
