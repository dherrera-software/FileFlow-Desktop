//! Resolución de estilos para documentos DOCX.

use quick_xml::events::BytesStart;
use std::collections::HashMap;

use crate::convert::documents::office::zip_utils::attribute;
use crate::convert::documents::{Alignment, BlockStyle};

/// Cambios de estilo opcionales. `None` significa "sin cambio"; `Some(false)`
/// permite desactivar explícitamente negrita, cursiva o subrayado.
#[derive(Debug, Clone, Default)]
pub struct StylePatch {
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub font_size: Option<u8>,
    pub color: Option<String>,
    pub font_family: Option<String>,
    pub align: Option<Alignment>,
    pub outline: Option<u8>,
    pub numbered: bool,
}

impl StylePatch {
    pub fn overlay(&mut self, child: &StylePatch) {
        if let Some(value) = child.bold {
            self.bold = Some(value);
        }
        if let Some(value) = child.italic {
            self.italic = Some(value);
        }
        if let Some(value) = child.underline {
            self.underline = Some(value);
        }
        if let Some(value) = &child.font_size {
            self.font_size = Some(*value);
        }
        if let Some(value) = &child.color {
            self.color = Some(value.clone());
        }
        if let Some(value) = &child.font_family {
            self.font_family = Some(value.clone());
        }
        if let Some(value) = child.align {
            self.align = Some(value);
        }
        if let Some(value) = child.outline {
            self.outline = Some(value);
        }
        if child.numbered {
            self.numbered = true;
        }
    }

    pub fn apply(&self, style: &mut BlockStyle) {
        if let Some(value) = self.bold {
            style.bold = value;
        }
        if let Some(value) = self.italic {
            style.italic = value;
        }
        if let Some(value) = self.underline {
            style.underline = value;
        }
        if let Some(value) = self.font_size {
            style.font_size = Some(value);
        }
        if let Some(value) = &self.color {
            style.color = Some(value.clone());
        }
        if let Some(value) = &self.font_family {
            style.font_family = Some(value.clone());
        }
        if let Some(value) = self.align {
            style.align = value;
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StyleDef {
    pub based_on: Option<String>,
    pub name: Option<String>,
    pub patch: StylePatch,
}

#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    pub defaults: StylePatch,
    pub defs: HashMap<String, StyleDef>,
    pub default_paragraph: Option<String>,
}

pub struct ResolvedStyle {
    pub style: BlockStyle,
    pub heading: Option<u8>,
    pub numbered: bool,
}

impl Stylesheet {
    pub fn parse(xml: &str) -> Self {
        let mut sheet = Stylesheet::default();
        let mut reader = quick_xml::Reader::from_str(xml);
        reader.config_mut().trim_text(false);

        let mut current: Option<(String, StyleDef)> = None;
        let mut in_rpr = false;
        let mut in_ppr = false;
        let mut ignore_rpr = false;
        let mut in_doc_rpr = false;
        let mut in_doc_ppr = false;

        while let Ok(event) = reader.read_event() {
            if matches!(event, quick_xml::events::Event::Eof) {
                break;
            }
            match event {
                quick_xml::events::Event::Start(start) => {
                    let name = start.local_name();
                    match name.as_ref() {
                        "rPrDefault" => in_doc_rpr = true,
                        "pPrDefault" => in_doc_ppr = true,
                        "style" => {
                            if let Some(id) = attribute(&start, "styleId") {
                                let kind = attribute(&start, "type").unwrap_or_default();
                                if kind == "paragraph" || kind == "character" {
                                    let is_default =
                                        attribute(&start, "default").is_some_and(|value| {
                                            value == "1" || value.eq_ignore_ascii_case("true")
                                        });
                                    if kind == "paragraph" && is_default {
                                        sheet.default_paragraph = Some(id.clone());
                                    }
                                    current = Some((id, StyleDef::default()));
                                }
                            }
                        }
                        "rPr" => {
                            if current.is_some() || in_doc_rpr {
                                in_rpr = true;
                                ignore_rpr = in_ppr;
                            }
                        }
                        "pPr" => {
                            if current.is_some() || in_doc_ppr {
                                in_ppr = true;
                            }
                        }
                        "name" => {
                            if let Some((_, def)) = current.as_mut() {
                                def.name = attribute(&start, "val");
                            }
                        }
                        "basedOn" => {
                            if let Some((_, def)) = current.as_mut() {
                                def.based_on = attribute(&start, "val");
                            }
                        }
                        _ => {
                            if in_rpr && !ignore_rpr {
                                if in_doc_rpr {
                                    apply_run_element(name.as_ref(), &start, &mut sheet.defaults);
                                } else if let Some((_, def)) = current.as_mut() {
                                    apply_run_element(name.as_ref(), &start, &mut def.patch);
                                }
                            } else if in_ppr {
                                if in_doc_ppr {
                                    apply_paragraph_element(
                                        name.as_ref(),
                                        &start,
                                        &mut sheet.defaults,
                                    );
                                } else if let Some((_, def)) = current.as_mut() {
                                    apply_paragraph_element(name.as_ref(), &start, &mut def.patch);
                                }
                            }
                        }
                    }
                }
                quick_xml::events::Event::Empty(start) => {
                    let name = start.local_name();
                    match name.as_ref() {
                        "name" => {
                            if let Some((_, def)) = current.as_mut() {
                                def.name = attribute(&start, "val");
                            }
                        }
                        "basedOn" => {
                            if let Some((_, def)) = current.as_mut() {
                                def.based_on = attribute(&start, "val");
                            }
                        }
                        _ => {
                            if in_rpr && !ignore_rpr {
                                if in_doc_rpr {
                                    apply_run_element(name.as_ref(), &start, &mut sheet.defaults);
                                } else if let Some((_, def)) = current.as_mut() {
                                    apply_run_element(name.as_ref(), &start, &mut def.patch);
                                }
                            } else if in_ppr {
                                if in_doc_ppr {
                                    apply_paragraph_element(
                                        name.as_ref(),
                                        &start,
                                        &mut sheet.defaults,
                                    );
                                } else if let Some((_, def)) = current.as_mut() {
                                    apply_paragraph_element(name.as_ref(), &start, &mut def.patch);
                                }
                            }
                        }
                    }
                }
                quick_xml::events::Event::End(end) => match end.local_name().as_ref() {
                    "rPr" => {
                        in_rpr = false;
                        ignore_rpr = false;
                    }
                    "pPr" => in_ppr = false,
                    "rPrDefault" => in_doc_rpr = false,
                    "pPrDefault" => in_doc_ppr = false,
                    "style" => {
                        if let Some((id, def)) = current.take() {
                            sheet.defs.insert(id, def);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        sheet
    }

    pub fn chain(&self, id: &str) -> Vec<&StyleDef> {
        let mut output = Vec::new();
        let mut current = Some(id.to_string());
        let mut guard = 0;
        while let Some(id) = current {
            if guard >= 32 {
                break;
            }
            guard += 1;
            if let Some(def) = self.defs.get(&id) {
                output.push(def);
                current = def.based_on.clone();
            } else {
                break;
            }
        }
        output
    }

    pub fn resolved_patch(&self, id: &str) -> StylePatch {
        let chain = self.chain(id);
        let mut patch = StylePatch::default();
        for def in chain.iter().rev() {
            patch.overlay(&def.patch);
        }
        patch
    }

    pub fn style_for(&self, id: Option<&str>) -> ResolvedStyle {
        let mut patch = self.defaults.clone();
        let mut heading = None;
        let mut numbered = false;

        let target = if let Some(id) = id {
            Some(id.to_string())
        } else {
            self.default_paragraph.clone()
        };

        if let Some(id) = target {
            let chain = self.chain(&id);
            for def in chain.iter().rev() {
                patch.overlay(&def.patch);
            }
            for def in &chain {
                if heading.is_none() {
                    heading = heading_level(def);
                }
                if def.patch.numbered {
                    numbered = true;
                }
            }
        }

        let mut style = BlockStyle::default();
        patch.apply(&mut style);
        ResolvedStyle {
            style,
            heading,
            numbered,
        }
    }
}

pub fn heading_level(def: &StyleDef) -> Option<u8> {
    if let Some(level) = def.patch.outline {
        return Some((level + 1).clamp(1, 6));
    }
    let name = def.name.as_deref()?.to_ascii_lowercase();
    let digits = name.strip_prefix("heading")?.trim().parse::<u8>().ok()?;
    Some(digits.clamp(1, 6))
}

pub fn on_off(start: &BytesStart<'_>) -> bool {
    match attribute(start, "val") {
        Some(value) => !matches!(value.as_str(), "0" | "false" | "off" | "none"),
        None => true,
    }
}

pub fn apply_run_element(name: &str, start: &BytesStart<'_>, patch: &mut StylePatch) {
    match name {
        "b" | "bCs" => patch.bold = Some(on_off(start)),
        "i" | "iCs" => patch.italic = Some(on_off(start)),
        "u" => {
            patch.underline = Some(
                attribute(start, "val")
                    .map(|value| value != "none")
                    .unwrap_or(true),
            );
        }
        "sz" => {
            if let Some(size) = attribute(start, "val").and_then(|value| value.parse::<u8>().ok()) {
                patch.font_size = Some(size);
            }
        }
        "color" => {
            if let Some(value) = attribute(start, "val")
                && !value.eq_ignore_ascii_case("auto")
            {
                patch.color = Some(value);
            }
        }
        "rFonts" => {
            if let Some(family) = attribute(start, "ascii").or_else(|| attribute(start, "hAnsi")) {
                patch.font_family = Some(family);
            }
        }
        _ => {}
    }
}

pub fn apply_paragraph_element(name: &str, start: &BytesStart<'_>, patch: &mut StylePatch) {
    match name {
        "jc" => {
            if let Some(value) = attribute(start, "val") {
                patch.align = Some(alignment_from(&value));
            }
        }
        "outlineLvl" => {
            if let Some(level) = attribute(start, "val").and_then(|value| value.parse::<u8>().ok())
            {
                patch.outline = Some(level);
            }
        }
        "numPr" => patch.numbered = true,
        _ => {}
    }
}

pub fn alignment_from(value: &str) -> Alignment {
    match value {
        "center" => Alignment::Center,
        "right" | "end" => Alignment::Right,
        "both" | "justify" | "distribute" => Alignment::Justify,
        _ => Alignment::Left,
    }
}
