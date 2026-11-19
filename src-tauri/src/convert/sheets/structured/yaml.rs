//! Conversión hacia y desde YAML para hojas de cálculo.

use serde_json::{Map, Number, Value};
use yaml_rust2::{Yaml, YamlEmitter, YamlLoader};

use super::headers_of;
use crate::convert::sheets::tabular;

pub fn parse_yaml(text: &str) -> Result<Vec<Vec<String>>, String> {
    let documents = YamlLoader::load_from_str(text).map_err(|error| {
        format!("El archivo YAML no es válido: {error}. Revisa la indentación y la sintaxis.")
    })?;
    let document = documents
        .into_iter()
        .next()
        .ok_or_else(|| "El archivo YAML está vacío.".to_string())?;
    tabular::parse_json_value(yaml_to_value(&document))
}

fn yaml_to_value(node: &Yaml) -> Value {
    match node {
        Yaml::Null => Value::Null,
        Yaml::Boolean(value) => Value::Bool(*value),
        Yaml::Integer(value) => Value::Number(Number::from(*value)),
        Yaml::Real(value) => value
            .parse::<f64>()
            .ok()
            .and_then(Number::from_f64)
            .map(Value::Number)
            .unwrap_or_else(|| Value::String(value.clone())),
        Yaml::String(value) => Value::String(value.clone()),
        Yaml::Array(items) => Value::Array(items.iter().map(yaml_to_value).collect()),
        Yaml::Hash(pairs) => {
            let mut object = Map::new();
            for (key, value) in pairs {
                object.insert(yaml_key(key), yaml_to_value(value));
            }
            Value::Object(object)
        }
        _ => Value::Null,
    }
}

fn yaml_key(node: &Yaml) -> String {
    match node {
        Yaml::String(value) => value.clone(),
        Yaml::Integer(value) => value.to_string(),
        Yaml::Boolean(value) => value.to_string(),
        Yaml::Real(value) => value.clone(),
        _ => "value".to_string(),
    }
}

pub fn to_yaml(rows: &[Vec<String>]) -> Result<String, String> {
    if rows.is_empty() {
        return Ok("[]\n".to_string());
    }
    let headers = headers_of(&rows[0]);
    let mut items = Vec::new();
    for row in rows.iter().skip(1) {
        let mut map: yaml_rust2::yaml::Hash = Default::default();
        for (index, header) in headers.iter().enumerate() {
            map.insert(
                Yaml::String(header.clone()),
                Yaml::String(row.get(index).cloned().unwrap_or_default()),
            );
        }
        items.push(Yaml::Hash(map));
    }
    let mut output = String::new();
    let mut emitter = YamlEmitter::new(&mut output);
    emitter
        .dump(&Yaml::Array(items))
        .map_err(|_| "No se pudo generar el YAML.".to_string())?;
    Ok(format!(
        "{}\n",
        output.trim_start_matches("---\n").trim_end()
    ))
}
