//! Módulo de escritura y generación de documentos PDF.

pub mod basic;
pub mod blocks;
pub mod encoding;
pub mod layout;

pub use basic::{image_to_pdf, table_to_pdf, text_to_pdf};
pub use blocks::blocks_to_pdf;

#[cfg(test)]
pub use basic::image_dimensions;
