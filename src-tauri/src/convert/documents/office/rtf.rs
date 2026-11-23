//! Conversión hacia y desde Rich Text Format (.rtf).

use super::super::{Alignment, Block, BlockKind, BlockStyle, Run};

fn flush_run(runs: &mut Vec<Run>, text: &mut String, style: &BlockStyle) {
    if text.is_empty() {
        return;
    }
    runs.push(Run::new(std::mem::take(text), style.clone()));
}

fn flush_paragraph(
    blocks: &mut Vec<Block>,
    runs: &mut Vec<Run>,
    text: &mut String,
    style: &mut BlockStyle,
    align: &mut Alignment,
    in_paragraph: &mut bool,
) {
    flush_run(runs, text, style);
    if *in_paragraph {
        let mut block_style = style.clone();
        block_style.align = *align;
        let block = Block::from_runs(BlockKind::Paragraph, std::mem::take(runs), block_style);
        if !block.text.is_empty() {
            blocks.push(block);
        }
    }
    runs.clear();
    text.clear();
    *style = BlockStyle::default();
    *align = Alignment::Left;
    *in_paragraph = false;
}

pub fn read_rtf(bytes: &[u8]) -> Result<Vec<Block>, String> {
    let mut index = 0usize;
    let mut depth = 0usize;
    let mut skip_depth: Option<usize> = None;

    let mut blocks: Vec<Block> = Vec::new();
    let mut runs: Vec<Run> = Vec::new();
    let mut run_text = String::new();
    let mut run_style = BlockStyle::default();
    let mut paragraph_align = Alignment::Left;
    let mut in_paragraph = false;

    while index < bytes.len() {
        match bytes[index] {
            b'{' => {
                depth += 1;
                index += 1;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                if skip_depth.is_some_and(|skip| depth < skip) {
                    skip_depth = None;
                }
                index += 1;
            }
            b'\\' => {
                index += 1;
                if index >= bytes.len() {
                    break;
                }
                match bytes[index] {
                    b'\\' | b'{' | b'}' => {
                        if skip_depth.is_none() {
                            run_text.push(bytes[index] as char);
                            in_paragraph = true;
                        }
                        index += 1;
                    }
                    b'\'' => {
                        let has_hex = index + 2 < bytes.len();
                        if has_hex
                            && let Ok(value) = u8::from_str_radix(
                                &String::from_utf8_lossy(&bytes[index + 1..index + 3]),
                                16,
                            )
                            && skip_depth.is_none()
                        {
                            let single = [value];
                            let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(&single);
                            run_text.push_str(&decoded);
                            in_paragraph = true;
                        }
                        index += if has_hex { 3 } else { 1 };
                    }
                    b'*' => index += 1,
                    b'\n' | b'\r' => {
                        if skip_depth.is_none() {
                            run_text.push('\n');
                            in_paragraph = true;
                        }
                        index += 1;
                    }
                    _ => {
                        let start = index;
                        while index < bytes.len()
                            && (bytes[index].is_ascii_alphabetic() || bytes[index] == b'-')
                        {
                            index += 1;
                        }
                        let word = String::from_utf8_lossy(&bytes[start..index]).to_string();
                        let mut numeric = String::new();
                        while index < bytes.len()
                            && (bytes[index].is_ascii_digit() || bytes[index] == b'-')
                        {
                            numeric.push(bytes[index] as char);
                            index += 1;
                        }
                        if index < bytes.len() && bytes[index] == b' ' {
                            index += 1;
                        }
                        if is_rtf_destination(&word) && skip_depth.is_none() {
                            skip_depth = Some(depth);
                        }
                        if skip_depth.is_none() {
                            match word.as_str() {
                                "par" => flush_paragraph(
                                    &mut blocks,
                                    &mut runs,
                                    &mut run_text,
                                    &mut run_style,
                                    &mut paragraph_align,
                                    &mut in_paragraph,
                                ),
                                "pard" => {
                                    flush_paragraph(
                                        &mut blocks,
                                        &mut runs,
                                        &mut run_text,
                                        &mut run_style,
                                        &mut paragraph_align,
                                        &mut in_paragraph,
                                    );
                                }
                                "line" => {
                                    run_text.push('\n');
                                    in_paragraph = true;
                                }
                                "tab" => {
                                    run_text.push('\t');
                                    in_paragraph = true;
                                }
                                "u" => {
                                    if let Ok(code) = numeric.parse::<i32>() {
                                        let value =
                                            if code < 0 { code + 65536 } else { code } as u32;
                                        if let Some(character) = char::from_u32(value) {
                                            run_text.push(character);
                                            in_paragraph = true;
                                        }
                                    }
                                    // \uc1: se descarta el carácter de reserva que sigue.
                                    if index < bytes.len()
                                        && !matches!(bytes[index], b'\\' | b'{' | b'}')
                                    {
                                        index += 1;
                                    }
                                }
                                "b" | "i" | "ul" => {
                                    let active = numeric.is_empty() || numeric != "0";
                                    flush_run(&mut runs, &mut run_text, &run_style);
                                    match word.as_str() {
                                        "b" => run_style.bold = active,
                                        "i" => run_style.italic = active,
                                        "ul" => run_style.underline = active,
                                        _ => {}
                                    }
                                    in_paragraph = true;
                                }
                                "fs" => {
                                    if let Ok(size) = numeric.parse::<u8>() {
                                        flush_run(&mut runs, &mut run_text, &run_style);
                                        run_style.font_size = Some(size);
                                    }
                                    in_paragraph = true;
                                }
                                "qc" => {
                                    paragraph_align = Alignment::Center;
                                    in_paragraph = true;
                                }
                                "qr" => {
                                    paragraph_align = Alignment::Right;
                                    in_paragraph = true;
                                }
                                "qj" => {
                                    paragraph_align = Alignment::Justify;
                                    in_paragraph = true;
                                }
                                "ql" => {
                                    paragraph_align = Alignment::Left;
                                    in_paragraph = true;
                                }
                                "emdash" => {
                                    run_text.push('—');
                                    in_paragraph = true;
                                }
                                "endash" => {
                                    run_text.push('–');
                                    in_paragraph = true;
                                }
                                "lquote" => {
                                    run_text.push('‘');
                                    in_paragraph = true;
                                }
                                "rquote" => {
                                    run_text.push('’');
                                    in_paragraph = true;
                                }
                                "ldblquote" => {
                                    run_text.push('“');
                                    in_paragraph = true;
                                }
                                "rdblquote" => {
                                    run_text.push('”');
                                    in_paragraph = true;
                                }
                                "bullet" => {
                                    run_text.push('•');
                                    in_paragraph = true;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            byte => {
                if skip_depth.is_none() && byte != b'\n' && byte != b'\r' {
                    run_text.push(byte as char);
                    in_paragraph = true;
                }
                index += 1;
            }
        }
    }

    flush_paragraph(
        &mut blocks,
        &mut runs,
        &mut run_text,
        &mut run_style,
        &mut paragraph_align,
        &mut in_paragraph,
    );

    if blocks.is_empty() {
        return Err("El RTF no contiene texto legible.".to_string());
    }
    Ok(blocks)
}

fn normalize_hex(color: &str) -> Option<String> {
    let hex = color.trim().trim_start_matches('#');
    if hex.len() == 6 && hex.chars().all(|character| character.is_ascii_hexdigit()) {
        Some(hex.to_ascii_lowercase())
    } else {
        None
    }
}

fn rtf_run(run: &Run, colors: &[String]) -> String {
    if run.text.is_empty() {
        return String::new();
    }
    let mut output = String::from("{");
    output.push_str(if run.style.bold { "\\b " } else { "\\b0 " });
    output.push_str(if run.style.italic { "\\i " } else { "\\i0 " });
    output.push_str(if run.style.underline {
        "\\ul "
    } else {
        "\\ul0 "
    });
    if let Some(size) = run.style.font_size {
        output.push_str(&format!("\\fs{size} "));
    }
    if let Some(color) = run.style.color.as_deref().and_then(normalize_hex)
        && let Some(index) = colors.iter().position(|entry| entry == &color)
    {
        output.push_str(&format!("\\cf{} ", index + 1));
    }
    output.push_str(&rtf_escape(&run.text));
    output.push('}');
    output
}

pub fn write_rtf(blocks: &[Block], _title: &str) -> Vec<u8> {
    let mut colors: Vec<String> = Vec::new();
    for block in blocks {
        for run in block.effective_runs() {
            if let Some(color) = run.style.color.as_deref().and_then(normalize_hex)
                && !colors.contains(&color)
            {
                colors.push(color);
            }
        }
    }
    let color_table = if colors.is_empty() {
        String::new()
    } else {
        let entries = colors
            .iter()
            .map(|color| {
                let red = u8::from_str_radix(&color[0..2], 16).unwrap_or(0);
                let green = u8::from_str_radix(&color[2..4], 16).unwrap_or(0);
                let blue = u8::from_str_radix(&color[4..6], 16).unwrap_or(0);
                format!("\\red{red}\\green{green}\\blue{blue};")
            })
            .collect::<String>();
        format!("{{\\colortbl;{entries}}}\n")
    };

    let mut body = String::new();
    for block in blocks {
        body.push_str("\\pard ");
        match block.style.align {
            Alignment::Center => body.push_str("\\qc "),
            Alignment::Right => body.push_str("\\qr "),
            Alignment::Justify => body.push_str("\\qj "),
            Alignment::Left => {}
        }
        let heading = matches!(block.kind, BlockKind::Heading(_));
        let mut runs = block.effective_runs();
        if heading {
            for run in &mut runs {
                run.style.bold = true;
            }
        }
        if block.kind == BlockKind::ListItem
            && let Some(first) = runs.first_mut()
        {
            first.text = format!("• {}", first.text);
        }
        for run in &runs {
            body.push_str(&rtf_run(run, &colors));
        }
        body.push_str("\\par\n");
    }

    format!(
        "{{\\rtf1\\ansi\\deff0\\uc1\n{{\\fonttbl{{\\f0 Calibri;}}{{\\f1 Consolas;}}}}\n{color_table}\\viewkind4\\pard\\f0\\fs22\n{body}}}"
    )
    .into_bytes()
}

fn rtf_escape(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '{' => output.push_str("\\{"),
            '}' => output.push_str("\\}"),
            '\t' => output.push_str("\\tab "),
            '\n' => output.push_str("\\line "),
            '\r' => {}
            character if (character as u32) < 0x80 => output.push(character),
            character => {
                let code = character as i32;
                let signed = if code > 32767 { code - 65536 } else { code };
                output.push_str(&format!("\\u{signed}?"));
            }
        }
    }
    output
}

fn is_rtf_destination(word: &str) -> bool {
    matches!(
        word,
        "fonttbl"
            | "colortbl"
            | "stylesheet"
            | "info"
            | "pict"
            | "object"
            | "header"
            | "footer"
            | "headerl"
            | "headerr"
            | "footerl"
            | "footerr"
            | "footnote"
            | "annotation"
            | "field"
            | "fldinst"
            | "datafield"
            | "themedata"
            | "colorschememapping"
            | "latentstyles"
            | "datastore"
            | "xmlnstbl"
            | "generator"
            | "listtable"
            | "listoverridetable"
            | "rsidtbl"
    )
}
