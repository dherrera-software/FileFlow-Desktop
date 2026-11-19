//! Registro de formatos y metadatos.

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Document,
    Image,
    Sheet,
    Presentation,
    Audio,
    Video,
    Archive,
}

impl Category {
    pub fn key(self) -> &'static str {
        match self {
            Category::Document => "document",
            Category::Image => "image",
            Category::Sheet => "sheet",
            Category::Presentation => "presentation",
            Category::Audio => "audio",
            Category::Video => "video",
            Category::Archive => "archive",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Category::Document => "Documentos",
            Category::Image => "Imágenes",
            Category::Sheet => "Hojas de cálculo",
            Category::Presentation => "Presentaciones",
            Category::Audio => "Audio",
            Category::Video => "Vídeo",
            Category::Archive => "Comprimidos",
        }
    }
}

pub struct Format {
    pub id: &'static str,
    pub category: Category,
    pub extensions: &'static [&'static str],
    pub mimes: &'static [&'static str],
    /// Extensión canónica al guardar.
    pub output_extension: &'static str,
}

impl Format {
    pub fn primary_mime(&self) -> &'static str {
        self.mimes
            .first()
            .copied()
            .unwrap_or("application/octet-stream")
    }
}

pub static FORMATS: &[Format] = &[
    // Documentos
    Format {
        id: "DOCX",
        category: Category::Document,
        extensions: &["docx"],
        mimes: &["application/vnd.openxmlformats-officedocument.wordprocessingml.document"],
        output_extension: "docx",
    },
    Format {
        id: "DOC",
        category: Category::Document,
        extensions: &["doc"],
        mimes: &["application/msword"],
        output_extension: "docx",
    },
    Format {
        id: "ODT",
        category: Category::Document,
        extensions: &["odt"],
        mimes: &["application/vnd.oasis.opendocument.text"],
        output_extension: "odt",
    },
    Format {
        id: "RTF",
        category: Category::Document,
        extensions: &["rtf"],
        mimes: &["application/rtf", "text/rtf"],
        output_extension: "rtf",
    },
    Format {
        id: "TXT",
        category: Category::Document,
        extensions: &["txt"],
        mimes: &["text/plain"],
        output_extension: "txt",
    },
    Format {
        id: "MD",
        category: Category::Document,
        extensions: &["md", "markdown"],
        mimes: &["text/markdown"],
        output_extension: "md",
    },
    Format {
        id: "HTML",
        category: Category::Document,
        extensions: &["html", "htm"],
        mimes: &["text/html"],
        output_extension: "html",
    },
    Format {
        id: "PDF",
        category: Category::Document,
        extensions: &["pdf"],
        mimes: &["application/pdf"],
        output_extension: "pdf",
    },
    Format {
        id: "EPUB",
        category: Category::Document,
        extensions: &["epub"],
        mimes: &["application/epub+zip"],
        output_extension: "epub",
    },
    // Imágenes
    Format {
        id: "JPG",
        category: Category::Image,
        extensions: &["jpg", "jpeg"],
        mimes: &["image/jpeg"],
        output_extension: "jpg",
    },
    Format {
        id: "PNG",
        category: Category::Image,
        extensions: &["png"],
        mimes: &["image/png"],
        output_extension: "png",
    },
    Format {
        id: "WEBP",
        category: Category::Image,
        extensions: &["webp"],
        mimes: &["image/webp"],
        output_extension: "webp",
    },
    Format {
        id: "BMP",
        category: Category::Image,
        extensions: &["bmp"],
        mimes: &["image/bmp", "image/x-ms-bmp"],
        output_extension: "bmp",
    },
    Format {
        id: "GIF",
        category: Category::Image,
        extensions: &["gif"],
        mimes: &["image/gif"],
        output_extension: "gif",
    },
    Format {
        id: "TIFF",
        category: Category::Image,
        extensions: &["tiff", "tif"],
        mimes: &["image/tiff"],
        output_extension: "tiff",
    },
    Format {
        id: "HEIC",
        category: Category::Image,
        extensions: &["heic", "heif"],
        mimes: &["image/heic", "image/heif"],
        output_extension: "jpg",
    },
    Format {
        id: "SVG",
        category: Category::Image,
        extensions: &["svg"],
        mimes: &["image/svg+xml"],
        output_extension: "svg",
    },
    Format {
        id: "AVIF",
        category: Category::Image,
        extensions: &["avif"],
        mimes: &["image/avif"],
        output_extension: "avif",
    },
    Format {
        id: "ICO",
        category: Category::Image,
        extensions: &["ico"],
        mimes: &["image/x-icon", "image/vnd.microsoft.icon"],
        output_extension: "ico",
    },
    // Hojas de cálculo y datos
    Format {
        id: "XLSX",
        category: Category::Sheet,
        extensions: &["xlsx"],
        mimes: &["application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"],
        output_extension: "xlsx",
    },
    Format {
        id: "XLS",
        category: Category::Sheet,
        extensions: &["xls"],
        mimes: &["application/vnd.ms-excel"],
        output_extension: "xlsx",
    },
    Format {
        id: "ODS",
        category: Category::Sheet,
        extensions: &["ods"],
        mimes: &["application/vnd.oasis.opendocument.spreadsheet"],
        output_extension: "ods",
    },
    Format {
        id: "CSV",
        category: Category::Sheet,
        extensions: &["csv"],
        mimes: &["text/csv"],
        output_extension: "csv",
    },
    Format {
        id: "TSV",
        category: Category::Sheet,
        extensions: &["tsv"],
        mimes: &["text/tab-separated-values"],
        output_extension: "tsv",
    },
    Format {
        id: "JSON",
        category: Category::Sheet,
        extensions: &["json"],
        mimes: &["application/json"],
        output_extension: "json",
    },
    Format {
        id: "XML",
        category: Category::Sheet,
        extensions: &["xml"],
        mimes: &["application/xml", "text/xml"],
        output_extension: "xml",
    },
    Format {
        id: "YAML",
        category: Category::Sheet,
        extensions: &["yaml", "yml"],
        mimes: &["application/yaml", "text/yaml", "application/x-yaml"],
        output_extension: "yaml",
    },
    Format {
        id: "SQL",
        category: Category::Sheet,
        extensions: &["sql"],
        mimes: &["application/sql", "text/x-sql"],
        output_extension: "sql",
    },
    // Presentaciones
    Format {
        id: "PPTX",
        category: Category::Presentation,
        extensions: &["pptx"],
        mimes: &["application/vnd.openxmlformats-officedocument.presentationml.presentation"],
        output_extension: "pptx",
    },
    Format {
        id: "PPT",
        category: Category::Presentation,
        extensions: &["ppt"],
        mimes: &["application/vnd.ms-powerpoint"],
        output_extension: "pptx",
    },
    Format {
        id: "ODP",
        category: Category::Presentation,
        extensions: &["odp"],
        mimes: &["application/vnd.oasis.opendocument.presentation"],
        output_extension: "odp",
    },
    // Audio
    Format {
        id: "MP3",
        category: Category::Audio,
        extensions: &["mp3"],
        mimes: &["audio/mpeg"],
        output_extension: "mp3",
    },
    Format {
        id: "WAV",
        category: Category::Audio,
        extensions: &["wav"],
        mimes: &["audio/wav", "audio/x-wav"],
        output_extension: "wav",
    },
    Format {
        id: "FLAC",
        category: Category::Audio,
        extensions: &["flac"],
        mimes: &["audio/flac", "audio/x-flac"],
        output_extension: "flac",
    },
    Format {
        id: "AAC",
        category: Category::Audio,
        extensions: &["aac"],
        mimes: &["audio/aac"],
        output_extension: "aac",
    },
    Format {
        id: "OGG",
        category: Category::Audio,
        extensions: &["ogg", "oga"],
        mimes: &["audio/ogg"],
        output_extension: "ogg",
    },
    Format {
        id: "M4A",
        category: Category::Audio,
        extensions: &["m4a"],
        mimes: &["audio/mp4", "audio/x-m4a"],
        output_extension: "m4a",
    },
    Format {
        id: "WMA",
        category: Category::Audio,
        extensions: &["wma"],
        mimes: &["audio/x-ms-wma"],
        output_extension: "wma",
    },
    Format {
        id: "OPUS",
        category: Category::Audio,
        extensions: &["opus"],
        mimes: &["audio/opus"],
        output_extension: "opus",
    },
    Format {
        id: "AIFF",
        category: Category::Audio,
        extensions: &["aiff", "aif"],
        mimes: &["audio/aiff", "audio/x-aiff"],
        output_extension: "aiff",
    },
    // Vídeo
    Format {
        id: "MP4",
        category: Category::Video,
        extensions: &["mp4"],
        mimes: &["video/mp4"],
        output_extension: "mp4",
    },
    Format {
        id: "MOV",
        category: Category::Video,
        extensions: &["mov"],
        mimes: &["video/quicktime"],
        output_extension: "mov",
    },
    Format {
        id: "AVI",
        category: Category::Video,
        extensions: &["avi"],
        mimes: &["video/x-msvideo"],
        output_extension: "avi",
    },
    Format {
        id: "MKV",
        category: Category::Video,
        extensions: &["mkv"],
        mimes: &["video/x-matroska"],
        output_extension: "mkv",
    },
    Format {
        id: "WEBM",
        category: Category::Video,
        extensions: &["webm"],
        mimes: &["video/webm"],
        output_extension: "webm",
    },
    Format {
        id: "WMV",
        category: Category::Video,
        extensions: &["wmv"],
        mimes: &["video/x-ms-wmv"],
        output_extension: "wmv",
    },
    Format {
        id: "FLV",
        category: Category::Video,
        extensions: &["flv"],
        mimes: &["video/x-flv"],
        output_extension: "flv",
    },
    // Archivos comprimidos
    Format {
        id: "ZIP",
        category: Category::Archive,
        extensions: &["zip"],
        mimes: &["application/zip"],
        output_extension: "zip",
    },
    Format {
        id: "RAR",
        category: Category::Archive,
        extensions: &["rar"],
        mimes: &["application/vnd.rar", "application/x-rar-compressed"],
        output_extension: "rar",
    },
    Format {
        id: "7Z",
        category: Category::Archive,
        extensions: &["7z"],
        mimes: &["application/x-7z-compressed"],
        output_extension: "7z",
    },
];

pub fn format(id: &str) -> Option<&'static Format> {
    FORMATS.iter().find(|format| format.id == id)
}

pub fn detect(file_name: &str) -> Option<&'static Format> {
    let extension = Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if extension.is_empty() {
        return None;
    }
    FORMATS
        .iter()
        .find(|format| format.extensions.contains(&extension.as_str()))
}

pub fn format_count() -> usize {
    FORMATS.len()
}

pub fn is_audio_target(id: &str) -> bool {
    matches!(
        id,
        "MP3" | "WAV" | "FLAC" | "AAC" | "OGG" | "M4A" | "WMA" | "OPUS" | "AIFF"
    )
}
