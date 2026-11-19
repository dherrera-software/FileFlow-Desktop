//! Conversión de imágenes. Usa `image` para mapas de bits, `resvg` para SVG y
//! ffmpeg como códec opcional para HEIC/AVIF.

use std::io::Cursor;

use ico::{IconDir, IconDirEntry, IconImage, ResourceType};
use image::codecs::bmp::BmpEncoder;
use image::codecs::gif::GifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::tiff::TiffEncoder;
use image::{DynamicImage, ExtendedColorType, ImageEncoder, ImageReader, Rgb, RgbImage, RgbaImage};

use crate::convert::documents::pdf;
use crate::convert::external;
use crate::convert::video;

pub fn convert(bytes: &[u8], source: &str, target: &str, _title: &str) -> Result<Vec<u8>, String> {
    if target == "PDF" {
        let encoded = if matches!(source, "JPG" | "PNG" | "WEBP" | "BMP" | "GIF" | "TIFF") {
            bytes.to_vec()
        } else {
            encode_png(&decode(bytes, source)?)?
        };
        return pdf::image_to_pdf(&encoded);
    }
    if target == "MP4" {
        return video::convert(bytes, source, target, Default::default());
    }
    let image = decode(bytes, source)?;
    match target {
        "JPG" => encode_jpeg(&image),
        "PNG" => encode_png(&image),
        "WEBP" => encode_webp(&image),
        "BMP" => encode_bmp(&image),
        "GIF" => encode_gif(&image),
        "TIFF" => encode_tiff(&image),
        "ICO" => encode_ico(&image),
        "AVIF" => {
            let png = encode_png(&image)?;
            external::ffmpeg_image(
                &png,
                "png",
                "avif",
                &[
                    "-c:v",
                    "libaom-av1",
                    "-still-picture",
                    "1",
                    "-crf",
                    "30",
                    "-pix_fmt",
                    "yuv444p",
                ],
            )
        }
        _ => Err(format!(
            "La conversión de {source} a {target} no está disponible."
        )),
    }
}

pub fn dimensions(bytes: &[u8], source: &str) -> Option<(u32, u32)> {
    match source {
        "SVG" => {
            let options = resvg::usvg::Options::default();
            let tree = resvg::usvg::Tree::from_data(bytes, &options).ok()?;
            let size = tree.size().to_int_size();
            Some((size.width(), size.height()))
        }
        "ICO" => {
            let directory = IconDir::read(Cursor::new(bytes)).ok()?;
            let entry = directory
                .entries()
                .iter()
                .max_by_key(|entry| entry.width() * entry.height())?;
            Some((entry.width(), entry.height()))
        }
        "HEIC" | "AVIF" => {
            let workspace = external::Workspace::new().ok()?;
            let extension = if source == "HEIC" { "heic" } else { "avif" };
            let input = workspace
                .write(&format!("entrada.{extension}"), bytes)
                .ok()?;
            external::ffprobe_dimensions(&input)
        }
        _ => {
            let reader = ImageReader::new(Cursor::new(bytes))
                .with_guessed_format()
                .ok()?;
            reader.into_dimensions().ok()
        }
    }
}

fn decode(bytes: &[u8], source: &str) -> Result<DynamicImage, String> {
    match source {
        "SVG" => render_svg(bytes),
        "ICO" => decode_ico(bytes),
        "HEIC" | "AVIF" => {
            let extension = if source == "HEIC" { "heic" } else { "avif" };
            let png = external::ffmpeg_image(bytes, extension, "png", &[])?;
            let image = image::load_from_memory(&png)
                .map_err(|_| "No se pudo interpretar la imagen decodificada.".to_string())?;
            Ok(cap(image))
        }
        "GIF" | "WEBP" => image::load_from_memory(bytes).map_err(|_| {
            "La imagen está dañada o usa una variante no compatible (por ejemplo, animación con códec extendido)."
                .to_string()
        }),
        _ => image::load_from_memory(bytes).map_err(|_| {
            "La imagen está dañada o su formato no coincide con la extensión.".to_string()
        }),
    }
}

fn decode_ico(bytes: &[u8]) -> Result<DynamicImage, String> {
    let directory = IconDir::read(Cursor::new(bytes))
        .map_err(|_| "El archivo ICO está dañado o no es un icono válido.".to_string())?;
    let entry = directory
        .entries()
        .iter()
        .max_by_key(|entry| entry.width() * entry.height())
        .ok_or_else(|| "El ICO no contiene imágenes.".to_string())?;
    let icon = entry
        .decode()
        .map_err(|_| "No se pudo decodificar la imagen del ICO.".to_string())?;
    let image = RgbaImage::from_raw(icon.width(), icon.height(), icon.rgba_data().to_vec())
        .ok_or_else(|| "El ICO contiene datos de imagen no válidos.".to_string())?;
    Ok(DynamicImage::ImageRgba8(image))
}

fn encode_ico(image: &DynamicImage) -> Result<Vec<u8>, String> {
    let mut directory = IconDir::new(ResourceType::Icon);
    for size in [16u32, 32, 48, 64, 128, 256] {
        let resized = image
            .resize_exact(size, size, image::imageops::FilterType::Triangle)
            .to_rgba8();
        let mut icon = IconImage::new(size, size);
        icon.rgba_data_mut().copy_from_slice(resized.as_raw());
        let entry = IconDirEntry::encode_as_png(&icon)
            .map_err(|_| "No se pudo generar una entrada del ICO.".to_string())?;
        directory.add_entry(entry);
    }
    let mut output = Vec::new();
    directory
        .write(&mut output)
        .map_err(|_| "No se pudo escribir el archivo ICO.".to_string())?;
    Ok(output)
}

fn render_svg(bytes: &[u8]) -> Result<DynamicImage, String> {
    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(bytes, &options)
        .map_err(|_| "El SVG no es válido o está dañado.".to_string())?;
    let size = tree.size().to_int_size();
    let mut width = size.width().max(1);
    let mut height = size.height().max(1);
    let scale = (4096.0 / width.max(height) as f32).min(1.0);
    if scale < 1.0 {
        width = ((width as f32) * scale).round().max(1.0) as u32;
        height = ((height as f32) * scale).round().max(1.0) as u32;
    }
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| "El SVG tiene un tamaño no válido.".to_string())?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(
            width as f32 / size.width().max(1) as f32,
            height as f32 / size.height().max(1) as f32,
        ),
        &mut pixmap.as_mut(),
    );
    let image = RgbaImage::from_raw(width, height, pixmap.data().to_vec())
        .ok_or_else(|| "No se pudo preparar el SVG como imagen.".to_string())?;
    Ok(DynamicImage::ImageRgba8(image))
}

fn cap(image: DynamicImage) -> DynamicImage {
    let (width, height) = (image.width(), image.height());
    if width.max(height) > 4096 {
        let scale = 4096.0 / width.max(height) as f32;
        image.resize(
            ((width as f32) * scale).round().max(1.0) as u32,
            ((height as f32) * scale).round().max(1.0) as u32,
            image::imageops::FilterType::Triangle,
        )
    } else {
        image
    }
}

fn flatten(image: &DynamicImage) -> RgbImage {
    if !image.color().has_alpha() {
        return image.to_rgb8();
    }
    let rgba = image.to_rgba8();
    let mut rgb = RgbImage::new(rgba.width(), rgba.height());
    for (x, y, pixel) in rgba.enumerate_pixels() {
        let alpha = pixel[3] as f32 / 255.0;
        let blend =
            |channel: u8| -> u8 { (channel as f32 * alpha + 255.0 * (1.0 - alpha)).round() as u8 };
        rgb.put_pixel(
            x,
            y,
            Rgb([blend(pixel[0]), blend(pixel[1]), blend(pixel[2])]),
        );
    }
    rgb
}

fn encode_jpeg(image: &DynamicImage) -> Result<Vec<u8>, String> {
    let rgb = flatten(image);
    let mut output = Vec::new();
    JpegEncoder::new_with_quality(&mut output, 90)
        .encode(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            ExtendedColorType::Rgb8,
        )
        .map_err(|_| "No se pudo generar el JPG.".to_string())?;
    Ok(output)
}

fn encode_png(image: &DynamicImage) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    if image.color().has_alpha() {
        let rgba = image.to_rgba8();
        PngEncoder::new(&mut output)
            .write_image(
                rgba.as_raw(),
                rgba.width(),
                rgba.height(),
                ExtendedColorType::Rgba8,
            )
            .map_err(|_| "No se pudo generar el PNG.".to_string())?;
    } else {
        let rgb = image.to_rgb8();
        PngEncoder::new(&mut output)
            .write_image(
                rgb.as_raw(),
                rgb.width(),
                rgb.height(),
                ExtendedColorType::Rgb8,
            )
            .map_err(|_| "No se pudo generar el PNG.".to_string())?;
    }
    Ok(output)
}

fn encode_webp(image: &DynamicImage) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    if image.color().has_alpha() {
        let rgba = image.to_rgba8();
        image_webp::WebPEncoder::new(&mut output)
            .encode(
                rgba.as_raw(),
                rgba.width(),
                rgba.height(),
                image_webp::ColorType::Rgba8,
            )
            .map_err(|_| "No se pudo generar el WEBP.".to_string())?;
    } else {
        let rgb = image.to_rgb8();
        image_webp::WebPEncoder::new(&mut output)
            .encode(
                rgb.as_raw(),
                rgb.width(),
                rgb.height(),
                image_webp::ColorType::Rgb8,
            )
            .map_err(|_| "No se pudo generar el WEBP.".to_string())?;
    }
    Ok(output)
}

fn encode_bmp(image: &DynamicImage) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    let rgba = image.to_rgba8();
    BmpEncoder::new(&mut output)
        .write_image(
            rgba.as_raw(),
            rgba.width(),
            rgba.height(),
            ExtendedColorType::Rgba8,
        )
        .map_err(|_| "No se pudo generar el BMP.".to_string())?;
    Ok(output)
}

fn encode_gif(image: &DynamicImage) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    let rgba = image.to_rgba8();
    GifEncoder::new(&mut output)
        .encode(
            rgba.as_raw(),
            rgba.width(),
            rgba.height(),
            ExtendedColorType::Rgba8,
        )
        .map_err(|_| "No se pudo generar el GIF.".to_string())?;
    Ok(output)
}

fn encode_tiff(image: &DynamicImage) -> Result<Vec<u8>, String> {
    let mut output = Cursor::new(Vec::new());
    let rgba = image.to_rgba8();
    TiffEncoder::new(&mut output)
        .write_image(
            rgba.as_raw(),
            rgba.width(),
            rgba.height(),
            ExtendedColorType::Rgba8,
        )
        .map_err(|_| "No se pudo generar el TIFF.".to_string())?;
    Ok(output.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_png() -> Vec<u8> {
        let image = RgbaImage::from_pixel(3, 2, image::Rgba([20, 120, 200, 255]));
        encode_png(&DynamicImage::ImageRgba8(image)).unwrap()
    }

    #[test]
    fn png_converts_to_every_raster_target() {
        let png = sample_png();
        for target in ["JPG", "WEBP", "BMP", "GIF", "TIFF"] {
            let bytes = convert(&png, "PNG", target, "imagen").unwrap();
            assert!(!bytes.is_empty(), "falló PNG -> {target}");
        }
    }

    #[test]
    fn png_converts_to_pdf() {
        let pdf = convert(&sample_png(), "PNG", "PDF", "imagen").unwrap();
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn svg_converts_to_png_and_pdf() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10" fill="red"/></svg>"#;
        let png = convert(svg, "SVG", "PNG", "vector").unwrap();
        assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
        let pdf = convert(svg, "SVG", "PDF", "vector").unwrap();
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn corrupt_image_is_rejected() {
        let error = convert(b"no es una imagen", "PNG", "JPG", "x").unwrap_err();
        assert!(error.contains("dañada") || error.contains("formato"));
    }

    #[test]
    fn dimensions_are_reported() {
        assert_eq!(dimensions(&sample_png(), "PNG"), Some((3, 2)));
    }
}
