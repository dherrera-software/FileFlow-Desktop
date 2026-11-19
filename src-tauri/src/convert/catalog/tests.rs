//! Pruebas del catálogo de formatos y destinos.

use super::*;

#[test]
fn detects_formats_by_extension_without_case_sensitivity() {
    assert_eq!(detect("Foto.JPEG").unwrap().id, "JPG");
    assert_eq!(detect("libro.EPUB").unwrap().id, "EPUB");
    assert_eq!(detect("audio.aif").unwrap().id, "AIFF");
    assert_eq!(detect("datos.YML").unwrap().id, "YAML");
    assert_eq!(detect("video.MP4").unwrap().id, "MP4");
    assert_eq!(detect("paquete.7z").unwrap().id, "7Z");
    assert!(detect("desconocido.xyz").is_none());
}

#[test]
fn declared_targets_cover_every_required_route() {
    for (source, required) in matrix::DOCUMENTS
        .iter()
        .chain(matrix::IMAGES)
        .chain(matrix::SHEETS)
        .chain(matrix::PRESENTATIONS)
        .chain(matrix::AUDIO)
        .chain(matrix::ARCHIVES)
    {
        let targets = targets_for(source);
        for target in *required {
            assert!(
                targets.contains(target),
                "falta la ruta obligatoria {source} -> {target}"
            );
        }
    }
}

#[test]
fn every_format_has_a_unique_id_and_no_duplicate_extension() {
    let mut ids = std::collections::HashSet::new();
    let mut extensions = std::collections::HashSet::new();
    for format in FORMATS {
        assert!(ids.insert(format.id), "id duplicado: {}", format.id);
        assert!(!format.extensions.is_empty());
        assert!(!format.mimes.is_empty());
        for extension in format.extensions {
            assert!(
                extensions.insert(*extension),
                "extensión duplicada: {extension}"
            );
        }
    }
    assert_eq!(ids.len(), FORMATS.len());
}

#[test]
fn every_declared_target_exists() {
    for format in FORMATS {
        for target in targets_for(format.id) {
            assert!(
                formats::format(target).is_some(),
                "{} declara un destino inexistente: {target}",
                format.id
            );
        }
    }
}
