//! Pruebas unitarias para formatos estructurados (YAML y SQL).

use super::sql::{parse_sql, to_sql};
use super::yaml::{parse_yaml, to_yaml};

#[test]
fn yaml_round_trip_keeps_headers_and_values() {
    let yaml = "- nombre: Ana\n  nota: 10\n- nombre: Luis\n  nota: 9\n";
    let rows = parse_yaml(yaml).unwrap();
    assert_eq!(rows[0], vec!["nombre", "nota"]);
    assert_eq!(rows[1], vec!["Ana", "10"]);
    let generated = to_yaml(&rows).unwrap();
    let again = parse_yaml(&generated).unwrap();
    assert_eq!(again, rows);
}

#[test]
fn invalid_yaml_is_rejected() {
    let error = parse_yaml("- nombre: [sin cerrar\n").unwrap_err();
    assert!(error.contains("YAML"));
}

#[test]
fn sql_dump_becomes_rows() {
    let sql = r#"
        -- comentario
        CREATE TABLE personas (id INTEGER PRIMARY KEY, nombre TEXT, nota REAL);
        INSERT INTO personas (id, nombre, nota) VALUES (1, 'Ana', 9.5), (2, 'Luis''s', NULL);
    "#;
    let rows = parse_sql(sql).unwrap();
    assert_eq!(rows[0], vec!["id", "nombre", "nota"]);
    assert_eq!(rows[1], vec!["1", "Ana", "9.5"]);
    assert_eq!(rows[2], vec!["2", "Luis's", ""]);
}

#[test]
fn rows_become_valid_sql() {
    let rows = vec![
        vec!["id".to_string(), "nombre".to_string()],
        vec!["1".to_string(), "Ana".to_string()],
    ];
    let sql = to_sql(&rows).unwrap();
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS \"data\""));
    assert!(sql.contains("\"id\" INTEGER"));
    assert!(sql.contains("VALUES (1, 'Ana')"));
    let parsed = parse_sql(&sql).unwrap();
    assert_eq!(parsed[0], rows[0]);
    assert_eq!(parsed[1], rows[1]);
}

#[test]
fn sql_without_inserts_is_rejected() {
    let error = parse_sql("CREATE TABLE vacia (id INTEGER);").unwrap_err();
    assert!(error.contains("INSERT"));
}
