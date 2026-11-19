//! Matrices usadas por las pruebas de cobertura. Sólo incluyen rutas que no
//! dependen de herramientas externas opcionales.

pub const DOCUMENTS: &[(&str, &[&str])] = &[
    ("DOCX", &["ODT", "RTF", "TXT", "MD", "HTML", "PDF", "EPUB"]),
    ("ODT", &["DOCX", "PDF"]),
    ("RTF", &["DOCX", "PDF"]),
    ("TXT", &["DOCX", "PDF"]),
    ("MD", &["DOCX", "HTML", "PDF"]),
    ("HTML", &["MD", "PDF"]),
    ("PDF", &["DOCX", "TXT", "MD", "HTML", "XLSX", "CSV"]),
    ("EPUB", &["TXT", "MD", "HTML", "PDF"]),
];

pub const IMAGES: &[(&str, &[&str])] = &[
    ("JPG", &["PNG", "WEBP", "BMP", "GIF", "TIFF", "PDF", "ICO"]),
    ("PNG", &["JPG", "WEBP", "BMP", "GIF", "TIFF", "PDF", "ICO"]),
    ("WEBP", &["JPG", "PNG", "BMP", "GIF", "TIFF", "PDF", "ICO"]),
    ("BMP", &["JPG", "PNG", "WEBP", "GIF", "TIFF", "PDF", "ICO"]),
    ("GIF", &["JPG", "PNG", "WEBP", "BMP", "TIFF", "PDF", "ICO"]),
    ("TIFF", &["JPG", "PNG", "WEBP", "BMP", "GIF", "PDF", "ICO"]),
    ("HEIC", &["JPG", "PNG", "WEBP", "PDF"]),
    ("SVG", &["PNG", "PDF", "ICO"]),
    ("ICO", &["PNG", "PDF", "JPG"]),
];

pub const SHEETS: &[(&str, &[&str])] = &[
    (
        "XLSX",
        &[
            "XLS", "ODS", "CSV", "TSV", "JSON", "XML", "YAML", "SQL", "PDF",
        ],
    ),
    ("XLS", &["XLSX", "ODS", "CSV", "TSV", "PDF"]),
    ("ODS", &["XLSX", "XLS", "CSV", "TSV", "PDF"]),
    ("CSV", &["XLSX", "TSV", "JSON", "XML", "YAML", "SQL", "PDF"]),
    ("TSV", &["XLSX", "CSV", "JSON", "YAML", "SQL", "PDF"]),
    ("JSON", &["XLSX", "CSV", "TSV", "YAML", "SQL", "PDF"]),
    ("YAML", &["XLSX", "CSV", "JSON", "SQL", "PDF"]),
    ("SQL", &["XLSX", "CSV", "JSON", "YAML", "PDF"]),
];

pub const PRESENTATIONS: &[(&str, &[&str])] = &[
    ("PPTX", &["ODP", "PDF", "DOCX", "TXT", "MD", "HTML"]),
    ("ODP", &["PPTX", "PDF", "DOCX", "TXT", "MD", "HTML"]),
];

pub const AUDIO: &[(&str, &[&str])] = &[
    ("MP3", &["WAV", "FLAC", "AAC", "OGG", "M4A"]),
    ("WAV", &["MP3", "FLAC", "AAC", "OGG", "M4A"]),
    ("FLAC", &["MP3", "WAV", "AAC", "OGG", "M4A"]),
    ("AAC", &["MP3", "WAV", "FLAC", "OGG", "M4A"]),
    ("OGG", &["MP3", "WAV", "FLAC", "AAC", "M4A"]),
    ("M4A", &["MP3", "WAV", "FLAC", "AAC", "OGG"]),
];

pub const ARCHIVES: &[(&str, &[&str])] = &[("ZIP", &["ZIP", "7Z"]), ("7Z", &["ZIP"])];
