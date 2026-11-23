//! Distribución, medición de glifos y renderizado de líneas en PDF.

use super::encoding::{pdf_escape, win_ansi_bytes};
use crate::convert::documents::{Alignment, BlockStyle, Run};

#[derive(Clone)]
pub struct Glyph {
    pub character: char,
    pub style: BlockStyle,
}

pub fn font_name(bold: bool, italic: bool) -> &'static str {
    match (bold, italic) {
        (true, true) => "F4",
        (true, false) => "F2",
        (false, true) => "F3",
        (false, false) => "F1",
    }
}

pub fn glyph_width(character: char, size: f32, bold: bool) -> f32 {
    let em = match character {
        ' ' => 0.28,
        '\t' => 1.12, // Tabulador equivale a 4 espacios
        'i' | 'l' | 'j' | 'I' | '.' | ',' | ';' | ':' | '\'' | '!' | '|' => 0.28,
        'f' | 't' | 'r' | '(' | ')' | '[' | ']' | '-' => 0.34,
        'm' | 'M' | 'w' | 'W' | '@' => 0.82,
        'A'..='Z' => 0.66,
        '0'..='9' => 0.56,
        _ => 0.53,
    };
    size * em * if bold { 1.03 } else { 1.0 }
}

pub fn glyphs_width(glyphs: &[Glyph]) -> f32 {
    glyphs
        .iter()
        .map(|glyph| {
            let size = glyph
                .style
                .font_size
                .map(|value| value as f32 / 2.0)
                .unwrap_or(11.0);
            glyph_width(glyph.character, size, glyph.style.bold)
        })
        .sum()
}

pub fn line_font_size(line: &[Glyph], fallback: f32) -> f32 {
    line.iter()
        .map(|glyph| {
            glyph
                .style
                .font_size
                .map(|value| value as f32 / 2.0)
                .unwrap_or(fallback)
        })
        .fold(fallback, f32::max)
}

pub fn layout_block(runs: &[Run], default_size: f32, max_width: f32) -> Vec<Vec<Glyph>> {
    let mut lines: Vec<Vec<Glyph>> = Vec::new();
    let mut current: Vec<Glyph> = Vec::new();
    let mut width = 0.0_f32;

    for run in runs {
        let size = run
            .style
            .font_size
            .map(|value| value as f32 / 2.0)
            .unwrap_or(default_size);
        for character in run.text.chars() {
            if character == '\r' {
                continue;
            }
            if character == '\n' {
                while current.last().is_some_and(|glyph| glyph.character == ' ') {
                    current.pop();
                }
                lines.push(std::mem::take(&mut current));
                width = 0.0;
                continue;
            }
            let character_width = glyph_width(character, size, run.style.bold);
            if character == ' ' {
                if current.is_empty() {
                    continue;
                }
                current.push(Glyph {
                    character,
                    style: run.style.clone(),
                });
                width += character_width;
                continue;
            }
            if width + character_width > max_width && !current.is_empty() {
                if let Some(space_index) = current.iter().rposition(|glyph| glyph.character == ' ')
                {
                    let remainder = current.split_off(space_index + 1);
                    while current.last().is_some_and(|glyph| glyph.character == ' ') {
                        current.pop();
                    }
                    lines.push(std::mem::take(&mut current));
                    current = remainder;
                    width = glyphs_width(&current);
                } else {
                    lines.push(std::mem::take(&mut current));
                    width = 0.0;
                }
            }
            current.push(Glyph {
                character,
                style: run.style.clone(),
            });
            width += character_width;
        }
    }

    while current.last().is_some_and(|glyph| glyph.character == ' ') {
        current.pop();
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

pub fn same_text_style(a: &BlockStyle, b: &BlockStyle) -> bool {
    a.bold == b.bold
        && a.italic == b.italic
        && a.underline == b.underline
        && a.font_size == b.font_size
        && a.color == b.color
        && a.font_family == b.font_family
}

pub struct LineLayout {
    pub align: Alignment,
    pub y: f32,
    pub available: f32,
    pub margin: f32,
    pub default_size: f32,
    pub force_bold: bool,
    pub last_line: bool,
}

pub fn render_line(output: &mut Vec<u8>, line: &[Glyph], layout: &LineLayout) {
    if line.is_empty() {
        return;
    }
    let width = glyphs_width(line);
    let spaces = line.iter().filter(|glyph| glyph.character == ' ').count();
    let max_extra_spacing = layout.default_size * 0.8; // Evita espacios grotescos en líneas justificadas
    let is_substantially_filled = width >= layout.available * 0.65;
    let justify = layout.align == Alignment::Justify
        && !layout.last_line
        && spaces > 0
        && is_substantially_filled;

    let extra = if justify {
        ((layout.available - width) / spaces as f32).clamp(0.0, max_extra_spacing)
    } else {
        0.0
    };
    let effective_width = width + extra * spaces as f32;
    let mut x = match layout.align {
        Alignment::Center => layout.margin + (layout.available - effective_width) / 2.0,
        Alignment::Right => layout.margin + layout.available - effective_width,
        Alignment::Left | Alignment::Justify => layout.margin,
    };

    let mut index = 0;
    while index < line.len() {
        let style = &line[index].style;
        let start = index;
        while index < line.len() && same_text_style(&line[index].style, style) {
            index += 1;
        }
        let segment = &line[start..index];
        let segment_text = segment
            .iter()
            .map(|glyph| glyph.character)
            .collect::<String>();
        let segment_width = glyphs_width(segment);
        let segment_spaces = segment
            .iter()
            .filter(|glyph| glyph.character == ' ')
            .count();
        let size = style
            .font_size
            .map(|value| value as f32 / 2.0)
            .unwrap_or(layout.default_size);
        let (red, green, blue) = style
            .color
            .as_deref()
            .and_then(parse_hex_color)
            .unwrap_or((0.0, 0.0, 0.0));

        output.extend(
            format!(
                "BT\n/{} {:.1} Tf\n{:.3} {:.3} {:.3} rg\n{:.2} Tw\n1 0 0 1 {:.2} {:.2} Tm\n",
                font_name(style.bold || layout.force_bold, style.italic),
                size,
                red,
                green,
                blue,
                extra,
                x,
                layout.y
            )
            .into_bytes(),
        );
        output.push(b'(');
        output.extend(pdf_escape(&win_ansi_bytes(&segment_text)));
        output.extend(b") Tj\nET\n");

        if style.underline {
            let thickness = (size * 0.06).max(0.6);
            let bottom = layout.y - size * 0.12;
            output.extend(
                format!(
                    "{:.3} {:.3} {:.3} rg\n{:.2} {:.2} {:.2} {:.2} re f\n",
                    red, green, blue, x, bottom, segment_width, thickness
                )
                .into_bytes(),
            );
        }

        x += segment_width + extra * segment_spaces as f32;
    }
}

pub fn parse_hex_color(color: &str) -> Option<(f32, f32, f32)> {
    let hex = color.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
    Some((r, g, b))
}
