//! Tipos semánticos para el modelo de documentos de FileFlow.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

/// Estilo básico aplicable a un bloque o a un fragmento de texto (`Run`).
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct BlockStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub font_size: Option<u8>,
    pub color: Option<String>,
    pub font_family: Option<String>,
    pub align: Alignment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockKind {
    Paragraph,
    Heading(u8),
    ListItem,
    Quote,
    Code,
    PageBreak,
    Table(Vec<Vec<String>>),
}

/// Fragmento de texto con estilo propio dentro de un bloque.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub text: String,
    pub style: BlockStyle,
}

impl Run {
    pub fn new(text: impl Into<String>, style: BlockStyle) -> Self {
        Run {
            text: text.into(),
            style,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub kind: BlockKind,
    pub text: String,
    pub style: BlockStyle,
    /// Estilos mixtos del bloque. Si está vacío, `text` y `style` describen el bloque completo.
    pub runs: Vec<Run>,
}

impl Block {
    pub fn paragraph(text: impl Into<String>) -> Self {
        Block {
            kind: BlockKind::Paragraph,
            text: text.into(),
            style: BlockStyle::default(),
            runs: Vec::new(),
        }
    }

    pub fn page_break() -> Self {
        Block {
            kind: BlockKind::PageBreak,
            text: String::new(),
            style: BlockStyle::default(),
            runs: Vec::new(),
        }
    }

    pub fn table(rows: Vec<Vec<String>>) -> Self {
        Block {
            kind: BlockKind::Table(rows),
            text: String::new(),
            style: BlockStyle::default(),
            runs: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_style(mut self, style: BlockStyle) -> Self {
        self.style = style;
        self
    }

    #[cfg(test)]
    pub fn heading(level: u8, text: impl Into<String>) -> Self {
        Block {
            kind: BlockKind::Heading(level.clamp(1, 6)),
            text: text.into(),
            style: BlockStyle::default(),
            runs: Vec::new(),
        }
    }

    /// Construye un bloque a partir de runs, recortando solo los extremos del
    /// texto y descartando runs vacíos para que `text` sea la concatenación
    /// exacta de los estilos.
    pub fn from_runs(kind: BlockKind, mut runs: Vec<Run>, style: BlockStyle) -> Self {
        trim_run_edges(&mut runs);
        let text = runs.iter().map(|run| run.text.as_str()).collect::<String>();
        Block {
            kind,
            text,
            style,
            runs,
        }
    }

    /// Runs efectivos: si el bloque no trae estilos mixtos, devuelve un único
    /// run con el estilo del bloque.
    pub fn effective_runs(&self) -> Vec<Run> {
        if self.runs.is_empty() {
            vec![Run::new(self.text.clone(), self.style.clone())]
        } else {
            self.runs.clone()
        }
    }
}

fn trim_run_edges(runs: &mut Vec<Run>) {
    for run in runs.iter_mut() {
        run.text = run.text.trim_start().to_string();
        if !run.text.is_empty() {
            break;
        }
    }
    runs.retain(|run| !run.text.is_empty());
    for run in runs.iter_mut().rev() {
        run.text = run.text.trim_end().to_string();
        if !run.text.is_empty() {
            break;
        }
    }
    runs.retain(|run| !run.text.is_empty());
}
