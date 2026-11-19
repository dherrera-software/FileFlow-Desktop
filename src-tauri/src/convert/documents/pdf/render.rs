//! Rasterización de PDF a imágenes usando Ghostscript.

use image::ExtendedColorType;
use image::codecs::jpeg::JpegEncoder;
use std::ffi::OsStr;
use std::fs;

use crate::convert::external;

/// Rasteriza la primera página con Ghostscript y la devuelve como PNG o JPG.
pub fn render_first_page(bytes: &[u8], target: &str) -> Result<Vec<u8>, String> {
    let program = external::ghostscript().ok_or_else(|| {
        "Convertir PDF a imagen requiere Ghostscript instalado (o define FILEFLOW_GS).".to_string()
    })?;
    let workspace = external::Workspace::new()?;
    let input = workspace.write("entrada.pdf", bytes)?;
    let rendered = workspace.path("pagina.png");
    let output = format!("-sOutputFile={}", rendered.display());
    let args: Vec<&OsStr> = vec![
        OsStr::new("-dSAFER"),
        OsStr::new("-dBATCH"),
        OsStr::new("-dNOPAUSE"),
        OsStr::new("-dFirstPage=1"),
        OsStr::new("-dLastPage=1"),
        OsStr::new("-sDEVICE=png16m"),
        OsStr::new("-r150"),
        OsStr::new(&output),
        input.as_os_str(),
    ];
    external::run_tool(&program, &args, None)?;
    let png = fs::read(&rendered)
        .map_err(|_| "Ghostscript no generó la imagen de la primera página.".to_string())?;
    if target == "PNG" {
        return Ok(png);
    }
    let image = image::load_from_memory(&png)
        .map_err(|_| "No se pudo interpretar la página rasterizada.".to_string())?;
    let rgb = image.to_rgb8();
    let mut output = Vec::new();
    JpegEncoder::new_with_quality(&mut output, 90)
        .encode(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            ExtendedColorType::Rgb8,
        )
        .map_err(|_| "No se pudo generar el JPG de la página.".to_string())?;
    Ok(output)
}
