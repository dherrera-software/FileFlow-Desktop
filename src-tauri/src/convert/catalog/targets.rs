//! Matriz de rutas de conversión soportadas.

/// Destinos declarados por el contrato de formatos.
pub fn targets_for(format: &str) -> Vec<&'static str> {
    match format {
        // Documentos
        "DOCX" => vec!["ODT", "RTF", "TXT", "MD", "HTML", "PDF", "EPUB"],
        "DOC" => vec!["DOCX", "PDF", "TXT", "MD", "HTML", "RTF", "ODT", "EPUB"],
        "ODT" => vec!["DOCX", "RTF", "TXT", "MD", "HTML", "PDF", "EPUB"],
        "RTF" => vec!["DOCX", "ODT", "TXT", "MD", "HTML", "PDF", "EPUB"],
        "TXT" => vec!["DOCX", "ODT", "RTF", "MD", "HTML", "PDF", "EPUB", "MP3"],
        "MD" => vec!["DOCX", "ODT", "RTF", "TXT", "HTML", "PDF", "EPUB"],
        "HTML" => vec!["DOCX", "ODT", "RTF", "TXT", "MD", "PDF", "EPUB"],
        // PDF: extracción de texto, hojas y primera página.
        "PDF" => vec![
            "DOCX", "ODT", "RTF", "TXT", "MD", "HTML", "EPUB", "XLSX", "CSV", "JPG", "PNG",
        ],
        "EPUB" => vec!["TXT", "MD", "HTML", "DOCX", "ODT", "RTF", "PDF"],
        // Imágenes
        "JPG" => vec!["PNG", "WEBP", "BMP", "GIF", "TIFF", "AVIF", "ICO", "PDF"],
        "PNG" => vec!["JPG", "WEBP", "BMP", "GIF", "TIFF", "AVIF", "ICO", "PDF"],
        "WEBP" => vec!["JPG", "PNG", "BMP", "GIF", "TIFF", "AVIF", "ICO", "PDF"],
        "BMP" => vec!["JPG", "PNG", "WEBP", "GIF", "TIFF", "AVIF", "ICO", "PDF"],
        "GIF" => vec![
            "JPG", "PNG", "WEBP", "BMP", "TIFF", "AVIF", "ICO", "PDF", "MP4",
        ],
        "TIFF" => vec!["JPG", "PNG", "WEBP", "BMP", "GIF", "AVIF", "ICO", "PDF"],
        "HEIC" => vec![
            "JPG", "PNG", "WEBP", "BMP", "GIF", "TIFF", "AVIF", "ICO", "PDF",
        ],
        "SVG" => vec!["PNG", "JPG", "WEBP", "ICO", "PDF"],
        "AVIF" => vec!["JPG", "PNG", "WEBP", "BMP", "GIF", "TIFF", "ICO", "PDF"],
        "ICO" => vec!["PNG", "JPG", "WEBP", "BMP", "GIF", "TIFF", "AVIF", "PDF"],
        // Hojas de cálculo y datos
        "XLSX" => vec![
            "XLS", "ODS", "CSV", "TSV", "JSON", "XML", "YAML", "SQL", "PDF",
        ],
        "XLS" => vec![
            "XLSX", "ODS", "CSV", "TSV", "JSON", "XML", "YAML", "SQL", "PDF",
        ],
        "ODS" => vec![
            "XLSX", "XLS", "CSV", "TSV", "JSON", "XML", "YAML", "SQL", "PDF",
        ],
        "CSV" => vec![
            "XLSX", "XLS", "ODS", "TSV", "JSON", "XML", "YAML", "SQL", "PDF",
        ],
        "TSV" => vec![
            "XLSX", "XLS", "ODS", "CSV", "JSON", "XML", "YAML", "SQL", "PDF",
        ],
        "JSON" => vec![
            "XLSX", "XLS", "ODS", "CSV", "TSV", "XML", "YAML", "SQL", "PDF",
        ],
        "XML" => vec![
            "XLSX", "XLS", "ODS", "CSV", "TSV", "JSON", "YAML", "SQL", "PDF",
        ],
        "YAML" => vec![
            "XLSX", "XLS", "ODS", "CSV", "TSV", "JSON", "XML", "SQL", "PDF",
        ],
        "SQL" => vec![
            "XLSX", "XLS", "ODS", "CSV", "TSV", "JSON", "XML", "YAML", "PDF",
        ],
        // Presentaciones
        "PPTX" => vec![
            "ODP", "PDF", "DOCX", "ODT", "RTF", "TXT", "MD", "HTML", "EPUB", "JPG", "PNG",
        ],
        "PPT" => vec![
            "PPTX", "PDF", "DOCX", "ODT", "RTF", "TXT", "MD", "HTML", "EPUB", "JPG", "PNG",
        ],
        "ODP" => vec![
            "PPTX", "PDF", "DOCX", "ODT", "RTF", "TXT", "MD", "HTML", "EPUB", "JPG", "PNG",
        ],
        // Audio (ffmpeg como códec local)
        "MP3" => vec!["WAV", "FLAC", "AAC", "OGG", "M4A", "WMA", "OPUS", "AIFF"],
        "WAV" => vec!["MP3", "FLAC", "AAC", "OGG", "M4A", "WMA", "OPUS", "AIFF"],
        "FLAC" => vec!["MP3", "WAV", "AAC", "OGG", "M4A", "WMA", "OPUS", "AIFF"],
        "AAC" => vec!["MP3", "WAV", "FLAC", "OGG", "M4A", "WMA", "OPUS", "AIFF"],
        "OGG" => vec!["MP3", "WAV", "FLAC", "AAC", "M4A", "WMA", "OPUS", "AIFF"],
        "M4A" => vec!["MP3", "WAV", "FLAC", "AAC", "OGG", "WMA", "OPUS", "AIFF"],
        "WMA" => vec!["MP3", "WAV", "FLAC", "AAC", "OGG", "M4A", "OPUS", "AIFF"],
        "OPUS" => vec!["MP3", "WAV", "FLAC", "AAC", "OGG", "M4A", "WMA", "AIFF"],
        "AIFF" => vec!["MP3", "WAV", "FLAC", "AAC", "OGG", "M4A", "WMA", "OPUS"],
        // Vídeo (ffmpeg): contenedores, GIF, audio y fotograma.
        "MP4" => video_targets(),
        "MOV" => video_targets(),
        "AVI" => video_targets(),
        "MKV" => video_targets(),
        "WEBM" => video_targets(),
        "WMV" => video_targets(),
        "FLV" => video_targets(),
        // Archivos comprimidos. ZIP como destino también representa "extraer".
        "ZIP" => vec!["ZIP", "RAR", "7Z"],
        "RAR" => vec!["ZIP", "RAR", "7Z"],
        "7Z" => vec!["ZIP", "RAR", "7Z"],
        _ => Vec::new(),
    }
}

fn video_targets() -> Vec<&'static str> {
    vec![
        "MP4", "MOV", "AVI", "MKV", "WEBM", "WMV", "FLV", "GIF", "MP3", "WAV", "FLAC", "AAC",
        "OGG", "M4A", "WMA", "OPUS", "AIFF", "JPG", "PNG",
    ]
}
