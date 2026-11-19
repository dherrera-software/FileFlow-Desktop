//! Lectura de hojas de cálculo (CSV, TSV, JSON, XML, YAML, SQL, XLSX, XLS, ODS).

use calamine::{Data, Ods, Reader, Xls, Xlsx};
use std::io::Cursor;

use super::structured;
use super::tabular;
use super::{CellData, SheetData};
use crate::convert::support;

pub fn read(bytes: &[u8], source: &str) -> Result<Vec<SheetData>, String> {
    match source {
        "CSV" => {
            let text = support::decode_text(bytes).text;
            let rows = tabular::parse_csv(&text)?;
            Ok(vec![SheetData {
                name: "Hoja1".to_string(),
                rows: rows
                    .into_iter()
                    .map(|row| row.into_iter().map(CellData::Text).collect())
                    .collect(),
            }])
        }
        "TSV" => {
            let text = support::decode_text(bytes).text;
            let rows = tabular::parse_tsv(&text)?;
            Ok(vec![SheetData {
                name: "Hoja1".to_string(),
                rows: rows
                    .into_iter()
                    .map(|row| row.into_iter().map(CellData::Text).collect())
                    .collect(),
            }])
        }
        "JSON" => {
            let text = support::decode_text(bytes).text;
            let rows = tabular::parse_json(&text)?;
            Ok(vec![SheetData {
                name: "Hoja1".to_string(),
                rows: rows
                    .into_iter()
                    .map(|row| row.into_iter().map(CellData::Text).collect())
                    .collect(),
            }])
        }
        "XML" => {
            let text = support::decode_text(bytes).text;
            let rows = tabular::parse_xml(&text)?;
            Ok(vec![SheetData {
                name: "Hoja1".to_string(),
                rows: rows
                    .into_iter()
                    .map(|row| row.into_iter().map(CellData::Text).collect())
                    .collect(),
            }])
        }
        "YAML" => {
            let text = support::decode_text(bytes).text;
            let rows = structured::parse_yaml(&text)?;
            Ok(vec![SheetData {
                name: "Hoja1".to_string(),
                rows: rows
                    .into_iter()
                    .map(|row| row.into_iter().map(CellData::Text).collect())
                    .collect(),
            }])
        }
        "SQL" => {
            let text = support::decode_text(bytes).text;
            let rows = structured::parse_sql(&text)?;
            Ok(vec![SheetData {
                name: "Hoja1".to_string(),
                rows: rows
                    .into_iter()
                    .map(|row| row.into_iter().map(CellData::Text).collect())
                    .collect(),
            }])
        }
        "XLSX" => read_calamine(bytes, "XLSX", Xlsx::new),
        "XLS" => read_calamine(bytes, "XLS", Xls::new),
        "ODS" => read_calamine(bytes, "ODS", Ods::new),
        _ => Err(format!("La conversión desde {source} no está disponible.")),
    }
}

pub fn read_calamine<'a, T, F>(
    bytes: &'a [u8],
    label: &str,
    factory: F,
) -> Result<Vec<SheetData>, String>
where
    T: Reader<Cursor<&'a [u8]>>,
    F: FnOnce(Cursor<&'a [u8]>) -> Result<T, T::Error>,
{
    let mut workbook = factory(Cursor::new(bytes)).map_err(|_| {
        format!("El archivo {label} está dañado o no es un libro de cálculo válido.")
    })?;
    let names = workbook.sheet_names();
    if names.is_empty() {
        return Err(format!("El archivo {label} no contiene hojas de cálculo."));
    }
    let mut sheets = Vec::new();
    for name in names {
        let range = workbook
            .worksheet_range(&name)
            .map_err(|_| format!("No se pudo leer la hoja «{name}» del archivo {label}."))?;
        let mut rows = Vec::new();
        for row in range.rows() {
            let mut cells: Vec<CellData> = row.iter().map(cell_data).collect();
            while cells.last().is_some_and(|cell| *cell == CellData::Empty) {
                cells.pop();
            }
            if !cells.is_empty() {
                rows.push(cells);
            }
        }
        sheets.push(SheetData { name, rows });
    }
    Ok(sheets)
}

fn cell_data(cell: &Data) -> CellData {
    match cell {
        Data::Empty => CellData::Empty,
        Data::Int(value) => CellData::Number(*value as f64),
        Data::Float(value) => CellData::Number(*value),
        Data::String(value) => CellData::Text(value.clone()),
        Data::Bool(value) => CellData::Bool(*value),
        Data::DateTime(value) => CellData::Number(value.as_f64()),
        Data::DateTimeIso(value) => CellData::Text(value.clone()),
        Data::DurationIso(value) => CellData::Text(value.clone()),
        Data::Error(value) => CellData::Text(format!("#ERROR({value:?})")),
    }
}
