//! Conversión hacia y desde volcados SQL (CREATE TABLE + INSERT INTO).

use std::collections::HashMap;

use super::headers_of;
use super::sql_parser::{
    parse_create_table, parse_insert, push_unique, split_statements, strip_sql_comments,
};

pub fn parse_sql(text: &str) -> Result<Vec<Vec<String>>, String> {
    let cleaned = strip_sql_comments(text);
    let mut tables: HashMap<String, Vec<String>> = HashMap::new();
    let mut inserted: Vec<(Vec<String>, Vec<String>)> = Vec::new();

    for statement in split_statements(&cleaned) {
        let upper = statement.trim_start().to_ascii_uppercase();
        if upper.starts_with("CREATE TABLE")
            && let Some((table, columns)) = parse_create_table(&statement)
        {
            tables.insert(table.to_ascii_lowercase(), columns);
        } else if upper.starts_with("INSERT INTO")
            && let Some((table, columns, tuples)) = parse_insert(&statement)
        {
            let columns = if columns.is_empty() {
                tables
                    .get(&table.to_ascii_lowercase())
                    .cloned()
                    .unwrap_or_default()
            } else {
                columns
            };
            for tuple in tuples {
                inserted.push((columns.clone(), tuple));
            }
        }
    }

    if inserted.is_empty() {
        return Err(
            "No se encontraron sentencias INSERT INTO ... VALUES en el volcado SQL.".to_string(),
        );
    }

    let mut headers: Vec<String> = Vec::new();
    for (columns, values) in &inserted {
        if columns.is_empty() {
            for index in 0..values.len() {
                push_unique(&mut headers, format!("col_{}", index + 1));
            }
        } else {
            for column in columns {
                push_unique(&mut headers, column.clone());
            }
        }
    }

    let mut matrix = vec![headers.clone()];
    for (columns, values) in inserted {
        let mut row = vec![String::new(); headers.len()];
        for (index, value) in values.into_iter().enumerate() {
            let header = if columns.is_empty() {
                format!("col_{}", index + 1)
            } else {
                columns.get(index).cloned().unwrap_or_default()
            };
            if let Some(position) = headers.iter().position(|item| *item == header) {
                row[position] = value;
            }
        }
        matrix.push(row);
    }
    Ok(matrix)
}

pub fn to_sql(rows: &[Vec<String>]) -> Result<String, String> {
    if rows.is_empty() {
        return Ok(String::new());
    }
    let headers = headers_of(&rows[0]);
    let mut columns = Vec::new();
    for (index, header) in headers.iter().enumerate() {
        columns.push(sql_identifier(header, index));
    }
    let mut definitions = Vec::new();
    let mut types = Vec::new();
    for (index, column) in columns.iter().enumerate() {
        let kind = infer_sql_type(rows.iter().skip(1).map(|row| row.get(index)));
        types.push(kind);
        definitions.push(format!("  \"{column}\" {kind}"));
    }
    let column_list = columns
        .iter()
        .map(|column| format!("\"{}\"", column.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(", ");
    let mut output = format!(
        "CREATE TABLE IF NOT EXISTS \"data\" (\n{}\n);\n\n",
        definitions.join(",\n")
    );
    for row in rows.iter().skip(1) {
        let values = columns
            .iter()
            .enumerate()
            .map(|(index, column)| {
                let value = row.get(index).map(String::as_str).unwrap_or("").trim();
                if value.is_empty() {
                    return "NULL".to_string();
                }
                if types[index] != "TEXT" && is_sql_number(column, value) {
                    value.to_string()
                } else {
                    sql_text(value)
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!(
            "INSERT INTO \"data\" ({column_list}) VALUES ({values});\n"
        ));
    }
    Ok(output)
}

fn infer_sql_type<'a>(values: impl Iterator<Item = Option<&'a String>>) -> &'static str {
    let values = values
        .flatten()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if values.is_empty() {
        return "TEXT";
    }
    if values.iter().all(|value| value.parse::<i64>().is_ok()) {
        "INTEGER"
    } else if values.iter().all(|value| value.parse::<f64>().is_ok()) {
        "REAL"
    } else {
        "TEXT"
    }
}

fn is_sql_number(column: &str, value: &str) -> bool {
    if column.is_empty() {
        return false;
    }
    value.parse::<i64>().is_ok() || value.parse::<f64>().is_ok()
}

fn sql_text(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn sql_identifier(value: &str, index: usize) -> String {
    let mut cleaned = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    if cleaned.is_empty() {
        cleaned = format!("col_{}", index + 1);
    }
    if cleaned
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit())
    {
        cleaned = format!("_{cleaned}");
    }
    cleaned
}
