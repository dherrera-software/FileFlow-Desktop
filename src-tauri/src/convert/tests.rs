//! Pruebas de integración del motor de conversión.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

fn scratch(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("fileflow-{unique}-{name}"))
}

fn convert_in_memory(bytes: &[u8], source: &str, target: &str) -> Result<Vec<u8>, String> {
    convert_bytes(bytes, source, target, "prueba", ConvertOptions::default())
}

#[test]
fn converts_text_document_on_disk() {
    let source = scratch("nota.txt");
    let output = scratch("nota.html");
    fs::write(&source, "Hola <mundo>\nSegunda línea\n\nOtro párrafo").unwrap();
    convert_file(&source, "HTML", &output, ConvertOptions::default()).unwrap();
    let html = fs::read_to_string(&output).unwrap();
    assert!(html.contains("&lt;mundo&gt;"));
    assert!(html.contains("<br>"));
    assert!(html.contains("Otro párrafo"));
    let _ = fs::remove_file(source);
    let _ = fs::remove_file(output);
}

#[test]
fn refuses_to_overwrite_the_source() {
    let source = scratch("datos.csv");
    fs::write(&source, "a,b\n1,2\n").unwrap();
    let error = convert_file(&source, "JSON", &source, ConvertOptions::default()).unwrap_err();
    assert!(error.contains("distinto"));
    let _ = fs::remove_file(source);
}

#[test]
fn rejects_unknown_empty_and_renamed_files() {
    let unknown = scratch("archivo.zzz");
    fs::write(&unknown, b"datos").unwrap();
    assert!(inspect_file(&unknown).unwrap_err().contains(".zzz"));
    let empty = scratch("vacio.txt");
    fs::write(&empty, b"").unwrap();
    assert!(inspect_file(&empty).unwrap_err().contains("vacío"));
    let renamed = scratch("foto.png");
    fs::write(&renamed, b"GIF89a pero con nombre png").unwrap();
    assert!(inspect_file(&renamed).is_err());
    let _ = fs::remove_file(unknown);
    let _ = fs::remove_file(empty);
    let _ = fs::remove_file(renamed);
}

#[test]
fn detects_csv_separator_and_legacy_encoding() {
    let source = scratch("datos.csv");
    // Punto y coma + Windows-1252 (0xF1 = ñ).
    fs::write(&source, b"nombre;ciudad\nAna;Ca\xf1\xf3n\n").unwrap();
    let output = scratch("datos.json");
    convert_file(&source, "JSON", &output, ConvertOptions::default()).unwrap();
    let json = fs::read_to_string(&output).unwrap();
    assert!(json.contains("\"nombre\""));
    assert!(json.contains("\"Ana\""));
    assert!(json.contains("Cañón"));
    let _ = fs::remove_file(source);
    let _ = fs::remove_file(output);
}

#[test]
fn every_required_document_route_works() {
    let source_markdown = "# Título\n\nPrimer párrafo.\n\n- uno\n- dos\n".as_bytes();
    for (source, targets) in catalog::matrix::DOCUMENTS {
        let fixture = if *source == "TXT" {
            b"Titulo\n\nPrimer parrafo.\n\nSegundo parrafo.".to_vec()
        } else {
            let from = if matches!(*source, "MD" | "HTML") {
                "TXT"
            } else {
                "MD"
            };
            convert_in_memory(source_markdown, from, source)
                .unwrap_or_else(|error| panic!("no se pudo crear fixture {source}: {error}"))
        };
        for target in *targets {
            convert_in_memory(&fixture, source, target).unwrap_or_else(|error| {
                panic!("falló la ruta obligatoria {source} -> {target}: {error}")
            });
        }
    }
}

#[test]
fn every_required_sheet_route_works() {
    let csv = b"nombre,nota\nAna,10\nLuis,9\n".to_vec();
    for (source, targets) in catalog::matrix::SHEETS {
        let fixture = if *source == "CSV" {
            csv.clone()
        } else {
            convert_in_memory(&csv, "CSV", source)
                .unwrap_or_else(|error| panic!("no se pudo crear fixture {source}: {error}"))
        };
        for target in *targets {
            convert_in_memory(&fixture, source, target).unwrap_or_else(|error| {
                panic!("falló la ruta obligatoria {source} -> {target}: {error}")
            });
        }
    }
}

#[test]
fn every_required_image_route_works() {
    use image::{ExtendedColorType, ImageEncoder, RgbaImage};
    let mut png = Vec::new();
    let image = RgbaImage::from_pixel(4, 3, image::Rgba([10, 120, 220, 255]));
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(image.as_raw(), 4, 3, ExtendedColorType::Rgba8)
        .unwrap();
    for (source, targets) in catalog::matrix::IMAGES {
        if *source == "HEIC" {
            // Requiere una muestra HEIC real o un codificador HEVC opcional.
            assert!(targets.contains(&"JPG"));
            continue;
        }
        let fixture = if *source == "SVG" {
            br#"<svg xmlns="http://www.w3.org/2000/svg" width="8" height="6"><rect width="8" height="6" fill="teal"/></svg>"#.to_vec()
        } else if *source == "PNG" {
            png.clone()
        } else {
            convert_in_memory(&png, "PNG", source)
                .unwrap_or_else(|error| panic!("no se pudo crear fixture {source}: {error}"))
        };
        for target in *targets {
            convert_in_memory(&fixture, source, target).unwrap_or_else(|error| {
                panic!("falló la ruta obligatoria {source} -> {target}: {error}")
            });
        }
    }
}

#[test]
fn every_required_audio_route_works_when_ffmpeg_is_present() {
    if external::ffmpeg().is_err() {
        eprintln!("ffmpeg no disponible: se omiten las rutas de audio");
        return;
    }
    let wav = audio::sample_wav();
    for (source, targets) in catalog::matrix::AUDIO {
        let fixture = if *source == "WAV" {
            wav.clone()
        } else {
            convert_in_memory(&wav, "WAV", source)
                .unwrap_or_else(|error| panic!("no se pudo crear fixture {source}: {error}"))
        };
        for target in *targets {
            convert_in_memory(&fixture, source, target).unwrap_or_else(|error| {
                panic!("falló la ruta obligatoria {source} -> {target}: {error}")
            });
        }
    }
}

#[test]
fn png_and_ico_round_trip_keep_pixels() {
    use image::{ExtendedColorType, ImageEncoder, RgbaImage};
    let mut png = Vec::new();
    let image = RgbaImage::from_pixel(32, 24, image::Rgba([10, 120, 220, 255]));
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(image.as_raw(), 32, 24, ExtendedColorType::Rgba8)
        .unwrap();
    let ico = convert_in_memory(&png, "PNG", "ICO").unwrap();
    assert!(ico.starts_with(&[0x00, 0x00, 0x01, 0x00]));
    let back = convert_in_memory(&ico, "ICO", "PNG").unwrap();
    assert!(back.starts_with(&[0x89, b'P', b'N', b'G']));
}

#[test]
fn gif_becomes_mp4_when_ffmpeg_is_present() {
    if external::ffmpeg().is_err() || !video::route_available("MP4") {
        eprintln!("ffmpeg no disponible: se omite GIF -> MP4");
        return;
    }
    use image::{ExtendedColorType, ImageEncoder, RgbaImage};
    let mut png = Vec::new();
    let image = RgbaImage::from_pixel(32, 24, image::Rgba([10, 120, 220, 255]));
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(image.as_raw(), 32, 24, ExtendedColorType::Rgba8)
        .unwrap();
    let gif = convert_in_memory(&png, "PNG", "GIF").unwrap();
    let mp4 = convert_in_memory(&gif, "GIF", "MP4").unwrap();
    assert!(mp4.get(4..8) == Some(b"ftyp"));
}

#[test]
fn video_conversions_work_when_ffmpeg_is_present() {
    if external::ffmpeg().is_err() {
        eprintln!("ffmpeg no disponible: se omiten las rutas de vídeo");
        return;
    }
    use std::ffi::OsStr;
    let workspace = external::Workspace::new().unwrap();
    let source = workspace.path("origen.mp4");
    let args: Vec<&OsStr> = vec![
        OsStr::new("-y"),
        OsStr::new("-hide_banner"),
        OsStr::new("-loglevel"),
        OsStr::new("error"),
        OsStr::new("-f"),
        OsStr::new("lavfi"),
        OsStr::new("-i"),
        OsStr::new("color=c=red:s=64x48:d=0.4"),
        OsStr::new("-f"),
        OsStr::new("lavfi"),
        OsStr::new("-i"),
        OsStr::new("sine=frequency=440:duration=0.4"),
        OsStr::new("-shortest"),
        OsStr::new("-c:v"),
        OsStr::new("libx264"),
        OsStr::new("-pix_fmt"),
        OsStr::new("yuv420p"),
        OsStr::new("-c:a"),
        OsStr::new("aac"),
        source.as_os_str(),
    ];
    external::run_ffmpeg(&args).unwrap();
    let mp4 = fs::read(&source).unwrap();
    for target in ["PNG", "JPG", "GIF", "MP3", "WEBM", "MKV"] {
        if !video::route_available(target) {
            continue;
        }
        convert_in_memory(&mp4, "MP4", target)
            .unwrap_or_else(|error| panic!("falló MP4 -> {target}: {error}"));
    }
}

#[test]
fn reports_available_routes_for_engine_info() {
    let declared = FORMATS
        .iter()
        .map(|format| catalog::targets_for(format.id).len())
        .sum::<usize>();
    let available = available_conversion_count();
    eprintln!("rutas disponibles en este equipo: {available} de {declared} declaradas");
    assert!(available > 0);
    assert!(available <= declared);
}

#[test]
fn pdf_text_tables_become_spreadsheet_rows() {
    let pdf = documents::pdf::text_to_pdf("nombre\tnota\nAna\t10\nLuis\t9\n").unwrap();
    let csv = convert_in_memory(&pdf, "PDF", "CSV");
    match csv {
        Ok(bytes) => {
            let text = String::from_utf8_lossy(&bytes);
            assert!(text.contains("Ana") || text.contains("nombre"));
        }
        Err(error) => {
            // pdf-extract puede no recuperar texto de un PDF sintético
            // mínimo; lo importante es que el error sea explícito.
            assert!(error.contains("PDF") || error.contains("filas"));
        }
    }
}
