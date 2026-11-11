use serde_json::{Map, Value};

pub fn csv_to_json(text: &str) -> Result<String, String> {
    let rows = parse_delimited(text, b',')?;
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
                object.insert(header.clone(), Value::String(row.get(index).cloned().unwrap_or_default()));
            }
            Value::Object(object)
        })
        .collect::<Vec<_>>();
    Ok(format!("{}\n", serde_json::to_string_pretty(&objects).map_err(|_| "No se pudo generar el JSON.")?))
}

pub fn csv_to_xml(text: &str) -> Result<String, String> {
    let rows = parse_delimited(text, b',')?;
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
    Ok(format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rows>\n{body}\n</rows>\n"))
}

pub fn tsv_to_csv(text: &str) -> Result<String, String> {
    Ok(to_csv(&parse_delimited(text, b'\t')?))
}

pub fn json_to_csv(text: &str) -> Result<String, String> {
    let data: Value = serde_json::from_str(text)
        .map_err(|_| "El archivo JSON no es válido. Revisa la sintaxis e inténtalo de nuevo.".to_string())?;
    let source = match data {
        Value::Array(items) => items,
        other => vec![other],
    };
    if source.is_empty() {
        return Ok("\n".to_string());
    }
    let objects = source
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
                    Some(Value::String(value)) => value.clone(),
                    Some(Value::Bool(value)) => value.to_string(),
                    Some(Value::Number(value)) => value.to_string(),
                    Some(other) => serde_json::to_string(other).unwrap_or_default(),
                })
                .collect(),
        );
    }
    Ok(to_csv(&rows))
}

pub fn xml_to_csv(text: &str) -> Result<String, String> {
    let document = roxmltree::Document::parse(text)
        .map_err(|_| "El archivo XML no es válido. Revisa la sintaxis e inténtalo de nuevo.".to_string())?;
    let root = document.root_element();
    let mut rows: Vec<roxmltree::Node<'_, '_>> = root.children().filter(|node| node.is_element()).collect();
    if rows.is_empty() {
        return Err("El XML no contiene datos para convertir.".to_string());
    }
    if rows.iter().all(|row| element_children(row).is_empty()) {
        rows = vec![root];
    }
    let mut columns = Vec::new();
    for row in &rows {
        for child in element_children(row) {
            let name = child.tag_name().name().to_string();
            if !columns.contains(&name) {
                columns.push(name);
            }
        }
    }
    if columns.is_empty() {
        let mut matrix = vec![vec!["value".to_string()]];
        matrix.extend(rows.into_iter().map(|row| vec![node_text(&row)]));
        return Ok(to_csv(&matrix));
    }
    let mut matrix = vec![columns.clone()];
    for row in rows {
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
    Ok(to_csv(&matrix))
}

fn parse_delimited(text: &str, delimiter: u8) -> Result<Vec<Vec<String>>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());
    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|_| "No se pudo leer la tabla. Revisa las comillas y los separadores.".to_string())?;
        rows.push(record.iter().map(str::to_string).collect::<Vec<_>>());
    }
    while rows.last().is_some_and(|row| row.len() == 1 && row[0].is_empty()) {
        rows.pop();
    }
    Ok(rows)
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

fn to_csv(rows: &[Vec<String>]) -> String {
    let mut writer = csv::WriterBuilder::new().from_writer(Vec::new());
    for row in rows {
        writer.write_record(row).expect("la memoria acepta la fila");
    }
    let bytes = writer.into_inner().expect("el csv cabe en memoria");
    String::from_utf8(bytes).expect("el csv es utf-8")
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
                .map(|character| if character.is_ascii_alphanumeric() || character == '_' || character == '.' || character == '-' { character } else { '_' })
                .collect();
            if name.is_empty() {
                name = format!("col{}", index + 1);
            }
            if !name.chars().next().is_some_and(|character| character.is_ascii_alphabetic() || character == '_') {
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

fn element_children<'a, 'input>(node: &roxmltree::Node<'a, 'input>) -> Vec<roxmltree::Node<'a, 'input>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_round_trips_through_json_and_xml() {
        let json = csv_to_json("nombre,nota\nAna,10\n\"Luis, Jr\",9\n").unwrap();
        assert!(json.contains("\"nombre\": \"Ana\""));
        assert!(json.contains("Luis, Jr"));
        let xml = csv_to_xml("nombre,nota\nAna,10\n").unwrap();
        assert!(xml.contains("<nombre>Ana</nombre>"));
        assert!(xml.contains("<nota>10</nota>"));
        let csv = json_to_csv(&json).unwrap();
        assert!(csv.lines().next().unwrap().contains("nombre"));
        assert!(csv.contains("Ana"));
    }

    #[test]
    fn tsv_becomes_csv() {
        let csv = tsv_to_csv("a\tb\n1\t2\n").unwrap();
        assert_eq!(csv, "a,b\n1,2\n");
    }

    #[test]
    fn xml_becomes_csv_and_invalid_inputs_fail() {
        let csv = xml_to_csv("<rows><row><nombre>Ana</nombre><nota>10</nota></row></rows>").unwrap();
        assert!(csv.contains("nombre,nota"));
        assert!(csv.contains("Ana,10"));
        assert!(json_to_csv("{").unwrap_err().contains("JSON"));
        assert!(xml_to_csv("<rows>").unwrap_err().contains("XML"));
    }

    #[test]
    fn nested_json_is_kept_as_text() {
        let csv = json_to_csv(r#"[{"id":1,"meta":{"ok":true}}]"#).unwrap();
        assert!(csv.contains("id,meta"));
        assert!(csv.contains("\"\"ok\"\":true"));
    }
}
