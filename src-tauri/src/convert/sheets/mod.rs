//! Conversión de hojas de cálculo. XLSX/XLS/ODS se leen con `calamine`; XLSX
//! se escribe con `rust_xlsxwriter`, ODS con ODF mínimo y XLS con BIFF8.
//!
//! Limitaciones documentadas: al exportar un libro con varias hojas a
//! CSV/TSV/JSON/XML/PDF se usa únicamente la primera hoja. Las fórmulas se
//! exportan como su último valor calculado.

pub mod biff8;
pub mod readers;
pub mod structured;
pub mod tabular;
pub mod writers;

#[cfg(test)]
mod tests;

use crate::convert::documents::pdf;

pub use biff8::write_xls;
pub use readers::read;
pub use writers::{write_ods, write_xlsx};

#[derive(Debug, Clone, PartialEq)]
pub enum CellData {
    Text(String),
    Number(f64),
    Bool(bool),
    Empty,
}

impl CellData {
    pub fn to_text(&self) -> String {
        match self {
            CellData::Text(value) => value.clone(),
            CellData::Number(value) => {
                if value.fract() == 0.0 && value.abs() < 1e15 {
                    format!("{}", *value as i64)
                } else {
                    let mut text = format!("{value}");
                    if text.contains('e') || text.contains('E') {
                        text = format!("{value:.10}");
                        while text.contains('.') && text.ends_with('0') {
                            text.pop();
                        }
                        if text.ends_with('.') {
                            text.pop();
                        }
                    }
                    text
                }
            }
            CellData::Bool(value) => value.to_string(),
            CellData::Empty => String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SheetData {
    pub name: String,
    pub rows: Vec<Vec<CellData>>,
}

impl SheetData {
    pub fn string_rows(&self) -> Vec<Vec<String>> {
        self.rows
            .iter()
            .map(|row| row.iter().map(CellData::to_text).collect())
            .collect()
    }
}

pub fn convert(bytes: &[u8], source: &str, target: &str, _title: &str) -> Result<Vec<u8>, String> {
    let sheets = read(bytes, source)?;
    if sheets.iter().all(|sheet| sheet.rows.is_empty()) {
        return Err("La hoja de cálculo no contiene datos que se puedan convertir.".to_string());
    }
    match target {
        "CSV" => tabular::to_csv(&sheets[0].string_rows()).map(String::into_bytes),
        "TSV" => tabular::to_tsv(&sheets[0].string_rows()).map(String::into_bytes),
        "JSON" => tabular::to_json(&sheets[0].string_rows()).map(String::into_bytes),
        "XML" => tabular::to_xml(&sheets[0].string_rows()).map(String::into_bytes),
        "YAML" => structured::to_yaml(&sheets[0].string_rows()).map(String::into_bytes),
        "SQL" => structured::to_sql(&sheets[0].string_rows()).map(String::into_bytes),
        "XLSX" => write_xlsx(&sheets),
        "ODS" => write_ods(&sheets),
        "XLS" => write_xls(&sheets),
        "PDF" => pdf::table_to_pdf(&sheets[0].string_rows()),
        _ => Err(format!(
            "La conversión de {source} a {target} no está disponible."
        )),
    }
}

/// Convierte filas ya extraídas (por ejemplo, de un PDF) a un destino tabular.
/// No admite JSON/XML/YAML/SQL porque esas salidas necesitan encabezados
/// fiables; el llamador decide si las filas los tienen.
pub fn convert_rows(rows: Vec<Vec<String>>, target: &str) -> Result<Vec<u8>, String> {
    if rows.is_empty() {
        return Err("No se detectaron filas de datos en el documento de origen.".to_string());
    }
    let sheets = vec![SheetData {
        name: "Tabla".to_string(),
        rows: rows
            .into_iter()
            .map(|row| row.into_iter().map(CellData::Text).collect())
            .collect(),
    }];
    match target {
        "CSV" => tabular::to_csv(&sheets[0].string_rows()).map(String::into_bytes),
        "TSV" => tabular::to_tsv(&sheets[0].string_rows()).map(String::into_bytes),
        "XLSX" => write_xlsx(&sheets),
        "ODS" => write_ods(&sheets),
        "XLS" => write_xls(&sheets),
        "PDF" => pdf::table_to_pdf(&sheets[0].string_rows()),
        _ => Err(format!(
            "La conversión de tabla extraída a {target} no está disponible."
        )),
    }
}
