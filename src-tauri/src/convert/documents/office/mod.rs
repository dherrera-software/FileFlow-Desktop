//! Módulo de documentos de oficina (DOCX, ODT, RTF, EPUB).

pub mod docx;
pub mod epub;
pub mod odt;
pub mod rtf;
pub mod zip_utils;

#[cfg(test)]
mod tests;

pub use docx::{read_docx, write_docx};
pub use epub::{read_epub, write_epub};
pub use odt::{read_odt, write_odt};
pub use rtf::{read_rtf, write_rtf};
pub use zip_utils::{build_zip_with_mimetype, named_entity, xml_escape};
