//! Utilidades tabulares: CSV/TSV (con detección de separador), JSON y XML.
//! Todas trabajan sobre una matriz de celdas de texto.

use serde_json::{Map, Value};

use crate::convert::support;

pub fn parse_csv(text: &str) -> Result<Vec<Vec<String>>, String> {
    let delimiter = support::detect_separator(text) as u8;
    parse_delimited(text, delimiter)
}

pub fn parse_tsv(text: &str) -> Result<Vec<Vec<String>>, String> {
    parse_delimited(text, b'\t')
}

fn parse_delimited(text: &str, delimiter: u8) -> Result<Vec<Vec<String>>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());
    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|_| {
            "No se pudo leer la tabla. Revisa las comillas y los separadores.".to_string()
        })?;
        rows.push(record.iter().map(str::to_string).collect::<Vec<_>>());
    }
    while rows
        .last()
        .is_some_and(|row| row.len() == 1 && row[0].trim().is_empty())
    {
        rows.pop();
    }
    Ok(rows)
}

pub fn parse_json(text: &str) -> Result<Vec<Vec<String>>, String> {
    let data: Value = serde_json::from_str(text).map_err(|_| {
        "El archivo JSON no es válido. Revisa la sintaxis e inténtalo de nuevo.".to_string()
    })?;
    parse_json_value(data)
}

/// Convierte un valor JSON ya parseado (también usado por YAML) en filas.
pub fn parse_json_value(data: Value) -> Result<Vec<Vec<String>>, String> {
    let items = match data {
        Value::Array(items) => items,
        other => vec![other],
    };
    if items.is_empty() {
        return Ok(Vec::new());
    }
    // Array de arrays: se conserva tal cual.
    if items.iter().all(Value::is_array) {
        return Ok(items
            .into_iter()
            .map(|item| {
                item.as_array()
                    .map(|row| row.iter().map(json_cell).collect())
                    .unwrap_or_default()
            })
            .collect());
    }
    let objects = items
        .into_iter()
        .map(|item| match item {
            Value::Object(map) => map,
            other => {
                let mut map = Map::new();
                map.insert("value".to_string(), other);
                map
            }
        })
        .collect::<Vec<_>>();
    let mut headers = Vec::new();
    for object in &objects {
        for key in object.keys() {
            if !headers.contains(key) {
                headers.push(key.clone());
            }
        }
    }
    if headers.is_empty() {
        headers.push("value".to_string());
    }
    let mut rows = vec![headers.clone()];
    for object in objects {
        rows.push(
            headers
                .iter()
                .map(|header| match object.get(header) {
                    None | Some(Value::Null) => String::new(),
                    Some(value) => json_cell(value),
                })
                .collect(),
        );
    }
    Ok(rows)
}

fn json_cell(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(value) => value.clone(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

pub fn parse_xml(text: &str) -> Result<Vec<Vec<String>>, String> {
    let document = roxmltree::Document::parse(text).map_err(|_| {
        "El archivo XML no es válido. Revisa la sintaxis e inténtalo de nuevo.".to_string()
    })?;
    let root = document.root_element();
    let mut nodes: Vec<roxmltree::Node<'_, '_>> =
        root.children().filter(|node| node.is_element()).collect();
    if nodes.is_empty() {
        return Err("El XML no contiene datos para convertir.".to_string());
    }
    if nodes.iter().all(|row| element_children(row).is_empty()) {
        nodes = vec![root];
    }
    let mut columns = Vec::new();
    for row in &nodes {
        for child in element_children(row) {
            let name = child.tag_name().name().to_string();
            if !columns.contains(&name) {
                columns.push(name);
            }
        }
    }
    if columns.is_empty() {
        let mut matrix = vec![vec!["value".to_string()]];
        matrix.extend(nodes.into_iter().map(|row| vec![node_text(&row)]));
        return Ok(matrix);
    }
    let mut matrix = vec![columns.clone()];
    for row in nodes {
        let children = element_children(&row);
        matrix.push(
            columns
                .iter()
                .map(|column| {
                    children
                        .iter()
                        .find(|child| child.tag_name().name() == column)
                        .map(node_text)
                        .unwrap_or_default()
                })
                .collect(),
        );
    }
    Ok(matrix)
}

fn element_children<'a, 'input>(
    node: &roxmltree::Node<'a, 'input>,
) -> Vec<roxmltree::Node<'a, 'input>> {
    node.children().filter(|child| child.is_element()).collect()
}

fn node_text(node: &roxmltree::Node<'_, '_>) -> String {
    node.text()
        .map(str::to_string)
        .unwrap_or_else(|| {
            node.descendants()
                .filter_map(|descendant| descendant.text())
                .collect::<String>()
        })
        .trim()
        .to_string()
}

pub fn to_csv(rows: &[Vec<String>]) -> Result<String, String> {
    write_delimited(rows, b',')
}

pub fn to_tsv(rows: &[Vec<String>]) -> Result<String, String> {
    write_delimited(rows, b'\t')
}

fn write_delimited(rows: &[Vec<String>], delimiter: u8) -> Result<String, String> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(Vec::new());
    for row in rows {
        writer
            .write_record(row)
            .map_err(|_| "No se pudo escribir la tabla delimitada.".to_string())?;
    }
    let bytes = writer
        .into_inner()
        .map_err(|_| "No se pudo cerrar la tabla delimitada.".to_string())?;
    String::from_utf8(bytes)
        .map_err(|_| "La tabla contiene caracteres no válidos en UTF-8.".to_string())
}

/// Convierte la primera fila en encabezados y el resto en objetos JSON.
pub fn to_json(rows: &[Vec<String>]) -> Result<String, String> {
    if rows.is_empty() {
        return Ok("[]\n".to_string());
    }
    let headers = headers_of(&rows[0]);
    let objects = rows
        .iter()
        .skip(1)
        .map(|row| {
            let mut object = Map::new();
            for (index, header) in headers.iter().enumerate() {
                object.insert(
                    header.clone(),
                    Value::String(row.get(index).cloned().unwrap_or_default()),
                );
            }
            Value::Object(object)
        })
        .collect::<Vec<_>>();
    Ok(format!(
        "{}\n",
        serde_json::to_string_pretty(&objects).map_err(|_| "No se pudo generar el JSON.")?
    ))
}

pub fn to_xml(rows: &[Vec<String>]) -> Result<String, String> {
    if rows.is_empty() {
        return Ok("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rows></rows>\n".to_string());
    }
    let headers = xml_names(&rows[0]);
    let body = rows
        .iter()
        .skip(1)
        .map(|row| {
            let fields = headers
                .iter()
                .enumerate()
                .map(|(index, header)| {
                    let value = xml_escape(row.get(index).map(String::as_str).unwrap_or(""));
                    format!("    <{header}>{value}</{header}>")
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("  <row>\n{fields}\n  </row>")
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rows>\n{body}\n</rows>\n"
    ))
}

fn headers_of(row: &[String]) -> Vec<String> {
    row.iter()
        .enumerate()
        .map(|(index, header)| {
            let trimmed = header.trim();
            if trimmed.is_empty() {
                format!("column{}", index + 1)
            } else {
                trimmed.to_string()
            }
        })
        .collect()
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn xml_names(headers: &[String]) -> Vec<String> {
    let mut used = std::collections::HashMap::<String, usize>::new();
    headers
        .iter()
        .enumerate()
        .map(|(index, header)| {
            let mut name: String = header
                .trim()
                .chars()
                .map(|character| {
                    if character.is_ascii_alphanumeric()
                        || character == '_'
                        || character == '.'
                        || character == '-'
                    {
                        character
                    } else {
                        '_'
                    }
                })
                .collect();
            if name.is_empty() {
                name = format!("col{}", index + 1);
            }
            if !name
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
            {
                name = format!("col_{name}");
            }
            let count = used.entry(name.clone()).or_insert(0);
            *count += 1;
            if *count == 1 {
                name
            } else {
                format!("{name}_{}", *count)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_detects_separator_and_quotes() {
        let rows = parse_csv("nombre;nota\nAna;10\n\"Luis; Jr\";9\n").unwrap();
        assert_eq!(rows[1], vec!["Ana", "10"]);
        assert_eq!(rows[2], vec!["Luis; Jr", "9"]);
    }

    #[test]
    fn json_and_xml_become_rows() {
        let rows = parse_json(r#"[{"id":1,"meta":{"ok":true}}]"#).unwrap();
        assert_eq!(rows[0], vec!["id", "meta"]);
        assert!(rows[1][1].contains("\"ok\":true"));
        let rows = parse_xml("<rows><row><nombre>Ana</nombre></row></rows>").unwrap();
        assert_eq!(rows[0], vec!["nombre"]);
        assert_eq!(rows[1], vec!["Ana"]);
    }

    #[test]
    fn writes_csv_tsv_json_and_xml() {
        let rows = vec![
            vec!["nombre".to_string(), "nota".to_string()],
            vec!["Ana".to_string(), "10".to_string()],
        ];
        assert_eq!(to_csv(&rows).unwrap(), "nombre,nota\nAna,10\n");
        assert_eq!(to_tsv(&rows).unwrap(), "nombre\tnota\nAna\t10\n");
        assert!(to_json(&rows).unwrap().contains("\"nombre\": \"Ana\""));
        assert!(to_xml(&rows).unwrap().contains("<nombre>Ana</nombre>"));
    }

    #[test]
    fn invalid_json_and_xml_fail() {
        assert!(parse_json("{").unwrap_err().contains("JSON"));
        assert!(parse_xml("<rows>").unwrap_err().contains("XML"));
    }
}
