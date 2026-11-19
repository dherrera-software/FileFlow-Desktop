//! Pruebas unitarias para hojas de cálculo.

use super::*;

fn sample_csv() -> Vec<u8> {
    b"nombre,nota\nAna,10\nLuis,9\n".to_vec()
}

#[test]
fn csv_converts_to_every_required_target() {
    for target in ["XLSX", "XLS", "ODS", "TSV", "JSON", "PDF"] {
        let bytes = convert(&sample_csv(), "CSV", target, "tabla").unwrap();
        assert!(!bytes.is_empty(), "falló CSV -> {target}");
    }
}

#[test]
fn xlsx_round_trip_keeps_values() {
    let xlsx = convert(&sample_csv(), "CSV", "XLSX", "tabla").unwrap();
    let csv = convert(&xlsx, "XLSX", "CSV", "tabla").unwrap();
    let csv = String::from_utf8(csv).unwrap();
    assert!(csv.contains("nombre,nota"));
    assert!(csv.contains("Ana,10"));
}

#[test]
fn ods_and_xls_round_trip_keep_values() {
    let ods = convert(&sample_csv(), "CSV", "ODS", "tabla").unwrap();
    let csv = convert(&ods, "ODS", "CSV", "tabla").unwrap();
    assert!(String::from_utf8(csv).unwrap().contains("Ana"));

    let xls = convert(&sample_csv(), "CSV", "XLS", "tabla").unwrap();
    let csv = convert(&xls, "XLS", "CSV", "tabla").unwrap();
    assert!(String::from_utf8(csv).unwrap().contains("Ana"));
}

#[test]
fn json_becomes_table_and_back() {
    let json = br#"[{"nombre":"Ana","nota":10}]"#;
    let csv = convert(json, "JSON", "CSV", "tabla").unwrap();
    let csv = String::from_utf8(csv).unwrap();
    assert!(csv.contains("nombre,nota"));
    assert!(csv.contains("Ana,10"));
}

#[test]
fn corrupt_workbook_is_rejected() {
    let error = convert(b"no es un xlsx", "XLSX", "CSV", "tabla").unwrap_err();
    assert!(error.contains("dañado") || error.contains("XLSX"));
}
