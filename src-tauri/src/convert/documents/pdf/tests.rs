//! Pruebas unitarias para generación de PDF.

use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, Rgb, RgbImage};

use super::writer::{image_dimensions, image_to_pdf, text_to_pdf};

#[test]
fn text_pdf_stores_spanish_as_winansi_bytes() {
    let pdf = text_to_pdf("cañón (sur)").unwrap();
    assert!(pdf.starts_with(b"%PDF"));
    assert!(pdf.windows(6).any(|window| window == b"ca\xf1\xf3n "));
    assert!(pdf.windows(7).any(|window| window == br"\(sur\)"));
}

#[test]
fn png_becomes_a_pdf() {
    let mut bytes = Vec::new();
    let image = RgbImage::from_pixel(2, 2, Rgb([12, 80, 140]));
    PngEncoder::new(&mut bytes)
        .write_image(image.as_raw(), 2, 2, ExtendedColorType::Rgb8)
        .unwrap();
    let pdf = image_to_pdf(&bytes).unwrap();
    assert!(pdf.starts_with(b"%PDF"));
    assert_eq!(image_dimensions(&bytes).unwrap(), (2, 2));
}
