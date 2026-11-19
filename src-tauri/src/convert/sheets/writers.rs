//! Escritores para formatos XLSX y ODS.

use rust_xlsxwriter::Workbook as XlsxWriter;

use super::{CellData, SheetData};
use crate::convert::documents::office;

pub fn write_xlsx(sheets: &[SheetData]) -> Result<Vec<u8>, String> {
    let mut workbook = XlsxWriter::new();
    for sheet in sheets {
        let worksheet = workbook.add_worksheet();
        worksheet
            .set_name(&sheet.name)
            .map_err(|_| "No se pudo nombrar la hoja de salida.".to_string())?;
        for (row_index, row) in sheet.rows.iter().enumerate() {
            for (column_index, cell) in row.iter().enumerate() {
                let row_index = row_index as u32;
                let column_index = column_index as u16;
                match cell {
                    CellData::Text(value) => {
                        worksheet
                            .write_string(row_index, column_index, value)
                            .map_err(|_| "No se pudo escribir una celda XLSX.".to_string())?;
                    }
                    CellData::Number(value) => {
                        worksheet
                            .write_number(row_index, column_index, *value)
                            .map_err(|_| "No se pudo escribir una celda XLSX.".to_string())?;
                    }
                    CellData::Bool(value) => {
                        worksheet
                            .write_boolean(row_index, column_index, *value)
                            .map_err(|_| "No se pudo escribir una celda XLSX.".to_string())?;
                    }
                    CellData::Empty => {}
                }
            }
        }
    }
    workbook
        .save_to_buffer()
        .map_err(|_| "No se pudo generar el archivo XLSX.".to_string())
}

pub fn write_ods(sheets: &[SheetData]) -> Result<Vec<u8>, String> {
    let mut body = String::new();
    for sheet in sheets {
        body.push_str(&format!(
            "<table:table table:name=\"{}\">",
            office::xml_escape(&sheet.name)
        ));
        let columns = sheet.rows.iter().map(Vec::len).max().unwrap_or(1).max(1);
        body.push_str(&format!(
            "<table:table-column table:number-columns-repeated=\"{columns}\"/>"
        ));
        for row in &sheet.rows {
            body.push_str("<table:table-row>");
            for cell in row {
                match cell {
                    CellData::Text(value) => body.push_str(&format!(
                        "<table:table-cell office:value-type=\"string\"><text:p>{}</text:p></table:table-cell>",
                        office::xml_escape(value)
                    )),
                    CellData::Number(value) => {
                        let text = CellData::Number(*value).to_text();
                        body.push_str(&format!(
                            "<table:table-cell office:value-type=\"float\" office:value=\"{}\"><text:p>{}</text:p></table:table-cell>",
                            office::xml_escape(&text),
                            office::xml_escape(&text)
                        ));
                    }
                    CellData::Bool(value) => body.push_str(&format!(
                        "<table:table-cell office:value-type=\"boolean\" office:boolean-value=\"{}\"><text:p>{}</text:p></table:table-cell>",
                        value, value
                    )),
                    CellData::Empty => body.push_str("<table:table-cell/>"),
                }
            }
            body.push_str("</table:table-row>");
        }
        body.push_str("</table:table>");
    }
    let content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" office:version="1.2">
<office:body><office:spreadsheet>{body}</office:spreadsheet></office:body>
</office:document-content>"#
    );
    let manifest = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.2">
<manifest:file-entry manifest:full-path="/" manifest:media-type="application/vnd.oasis.opendocument.spreadsheet"/>
<manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
</manifest:manifest>"#;
    office::build_zip_with_mimetype(
        "application/vnd.oasis.opendocument.spreadsheet",
        &[
            ("content.xml", content.into_bytes()),
            ("META-INF/manifest.xml", manifest.as_bytes().to_vec()),
        ],
    )
}
