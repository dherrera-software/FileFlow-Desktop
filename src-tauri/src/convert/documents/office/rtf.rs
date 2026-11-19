//! Conversión hacia y desde Rich Text Format (.rtf).

use super::super::{Block, BlockKind};
use super::zip_utils::push_block;

pub fn read_rtf(bytes: &[u8]) -> Result<Vec<Block>, String> {
    let mut text = String::new();
    let mut index = 0usize;
    let mut depth = 0usize;
    let mut skip_depth: Option<usize> = None;
    let mut paragraph_break = false;
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
                            text.push(bytes[index] as char);
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
                            text.push_str(&decoded);
                        }
                        index += if has_hex { 3 } else { 1 };
                    }
                    b'*' => index += 1,
                    b'\n' | b'\r' => {
                        if skip_depth.is_none() {
                            text.push('\n');
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
                                "par" | "pard" => paragraph_break = true,
                                "line" => text.push('\n'),
                                "tab" => text.push('\t'),
                                "u" => {
                                    if let Ok(code) = numeric.parse::<i32>() {
                                        let value =
                                            if code < 0 { code + 65536 } else { code } as u32;
                                        if let Some(character) = char::from_u32(value) {
                                            text.push(character);
                                        }
                                    }
                                    // \uc1: se descarta el carácter de reserva que sigue.
                                    if index < bytes.len()
                                        && !matches!(bytes[index], b'\\' | b'{' | b'}')
                                    {
                                        index += 1;
                                    }
                                }
                                "emdash" => text.push('—'),
                                "endash" => text.push('–'),
                                "lquote" => text.push('‘'),
                                "rquote" => text.push('’'),
                                "ldblquote" => text.push('“'),
                                "rdblquote" => text.push('”'),
                                "bullet" => text.push('•'),
                                _ => {}
                            }
                        }
                    }
                }
            }
            byte => {
                if skip_depth.is_none() {
                    if byte == b'\n' || byte == b'\r' {
                        // Los saltos de línea del propio RTF se ignoran.
                    } else {
                        text.push(byte as char);
                    }
                }
                index += 1;
            }
        }
    }
    if text.trim().is_empty() {
        return Err("El RTF no contiene texto legible.".to_string());
    }
    let mut blocks = Vec::new();
    for chunk in text.split("\n\n") {
        for line in chunk.split('\n') {
            if line.trim().is_empty() {
                continue;
            }
            push_block(&mut blocks, BlockKind::Paragraph, line);
        }
    }
    if paragraph_break && blocks.is_empty() {
        blocks.push(Block::paragraph(""));
    }
    Ok(blocks)
}

pub fn write_rtf(blocks: &[Block], _title: &str) -> Vec<u8> {
    let mut body = String::new();
    for block in blocks {
        let prefix = match block.kind {
            BlockKind::Heading(_) => "\\b ",
            BlockKind::Code => "\\f1 ",
            _ => "",
        };
        let text = if block.kind == BlockKind::ListItem {
            format!("• {}", block.text)
        } else {
            block.text.clone()
        };
        body.push_str(prefix);
        body.push_str(&rtf_escape(&text).replace('\n', "\\line "));
        body.push_str("\\par\n");
    }
    format!(
        "{{\\rtf1\\ansi\\deff0\\uc1\n{{\\fonttbl{{\\f0 Calibri;}}{{\\f1 Consolas;}}}}\n\\viewkind4\\pard\\f0\\fs22\n{body}}}"
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
