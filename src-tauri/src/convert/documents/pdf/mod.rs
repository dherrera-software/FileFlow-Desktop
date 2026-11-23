//! Módulo de generación y rasterización de PDF.

pub mod render;
pub mod writer;

#[cfg(test)]
mod tests;

pub use render::render_first_page;
pub use writer::{blocks_to_pdf, image_to_pdf, table_to_pdf, text_to_pdf};
