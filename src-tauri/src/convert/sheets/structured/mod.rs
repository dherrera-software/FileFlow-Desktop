//! Datos estructurados adicionales: YAML y volcados SQL.
//!
//! YAML se parsea con `yaml-rust2` y se aplana a la misma matriz de celdas que
//! JSON/CSV/XML. SQL cubre el subconjunto habitual de un volcado:
//! `CREATE TABLE` (para conocer columnas) e `INSERT INTO ... VALUES (...)`.

pub mod sql;
pub mod sql_parser;
pub mod yaml;

#[cfg(test)]
mod tests;

pub use sql::{parse_sql, to_sql};
pub use yaml::{parse_yaml, to_yaml};

pub(crate) fn headers_of(row: &[String]) -> Vec<String> {
    row.iter()
        .enumerate()
        .map(|(index, header)| {
            let trimmed = header.trim();
            if trimmed.is_empty() {
                format!("col_{}", index + 1)
            } else {
                trimmed.to_string()
            }
        })
        .collect()
}
