//! Registro central de formatos: extensión, MIME, categoría y destinos.
//!
//! Las matrices viven aquí para que la UI y las pruebas usen exactamente la
//! misma fuente de verdad. Los destinos que dependen de un motor externo
//! (FFmpeg, LibreOffice, WinRAR, Ghostscript o la voz de Windows) se filtran
//! en `mod.rs` antes de exponerse a la interfaz.

pub mod formats;
#[cfg(test)]
pub mod matrix;
pub mod targets;

#[cfg(test)]
mod tests;

pub use formats::{Category, FORMATS, Format, detect, format, format_count, is_audio_target};
pub use targets::targets_for;
