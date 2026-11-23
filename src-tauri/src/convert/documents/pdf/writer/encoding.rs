//! Codificación WinAnsi y mapeo tipográfico para PDFs nativos.

pub fn win_ansi_bytes(value: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    for character in value.chars() {
        let code = character as u32;
        if code == 9 || code < 0x20 {
            bytes.push(b' ');
        } else if code <= 0xff {
            bytes.push(code as u8);
        } else if let Some(extra) = win_ansi_extra(code) {
            bytes.push(extra);
        } else if let Some(mapped) = map_unicode_character(character) {
            bytes.extend_from_slice(mapped);
        } else {
            bytes.push(b'?');
        }
    }
    bytes
}

fn win_ansi_extra(code: u32) -> Option<u8> {
    Some(match code {
        0x20ac => 0x80, // €
        0x201a => 0x82,
        0x0192 => 0x83,
        0x201e => 0x84,
        0x2026 => 0x85, // …
        0x2020 => 0x86, // †
        0x2021 => 0x87, // ‡
        0x02c6 => 0x88,
        0x2030 => 0x89,
        0x0160 => 0x8a,
        0x2039 => 0x8b,
        0x0152 => 0x8c,
        0x017d => 0x8e,
        0x2018 => 0x91, // ‘
        0x2019 => 0x92, // ’
        0x201c => 0x93, // “
        0x201d => 0x94, // ”
        0x2022 => 0x95, // •
        0x2013 => 0x96, // –
        0x2014 => 0x97, // —
        0x02dc => 0x98,
        0x2122 => 0x99, // ™
        0x0161 => 0x9a,
        0x203a => 0x9b,
        0x0153 => 0x9c,
        0x017e => 0x9e,
        0x0178 => 0x9f,
        _ => return None,
    })
}

fn map_unicode_character(c: char) -> Option<&'static [u8]> {
    match c {
        // Viñetas y símbolos de lista
        '\u{25CF}' | '\u{25CB}' | '\u{25A0}' | '\u{25AA}' | '\u{25E6}' | '\u{2043}'
        | '\u{2219}' | '\u{25B8}' => Some(b"\x95"), // viñeta WinAnsi
        // Guiones y signos menos
        '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2015}' | '\u{2212}' => Some(b"-"),
        // Espacios especiales
        '\u{2000}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' => Some(b" "),
        '\u{200B}' | '\u{FEFF}' => Some(b""), // zero-width
        // Comillas y primas
        '\u{201E}' | '\u{201F}' => Some(b"\x94"),
        '\u{2032}' => Some(b"'"),
        '\u{2033}' => Some(b"\""),
        '\u{2025}' => Some(b".."),
        '\u{2044}' => Some(b"/"),
        // Flechas y símbolos matemáticos comunes
        '\u{2192}' | '\u{2794}' | '\u{27A4}' => Some(b"->"),
        '\u{2190}' => Some(b"<-"),
        '\u{2194}' => Some(b"<->"),
        '\u{2264}' => Some(b"<="),
        '\u{2265}' => Some(b">="),
        '\u{2260}' => Some(b"!="),
        '\u{2248}' => Some(b"~"),
        '\u{2713}' | '\u{2714}' => Some(b"v"),
        // Transliteraciones para acentos y caracteres latinos extendidos frecuentes
        'ą' | 'ă' | 'ā' => Some(b"a"),
        'Ą' | 'Ă' | 'Ā' => Some(b"A"),
        'ć' | 'č' | 'ċ' => Some(b"c"),
        'Ć' | 'Č' | 'Ċ' => Some(b"C"),
        'ď' | 'đ' => Some(b"d"),
        'Ď' | 'Đ' => Some(b"D"),
        'ę' | 'ě' | 'ē' => Some(b"e"),
        'Ę' | 'Ě' | 'Ē' => Some(b"E"),
        'ğ' | 'ġ' => Some(b"g"),
        'Ğ' | 'Ġ' => Some(b"G"),
        'ħ' => Some(b"h"),
        'Ħ' => Some(b"H"),
        'ī' | 'ı' | 'į' => Some(b"i"),
        'Ī' | 'İ' | 'Į' => Some(b"I"),
        'ł' | 'ľ' => Some(b"l"),
        'Ł' | 'Ľ' => Some(b"L"),
        'ń' | 'ň' => Some(b"n"),
        'Ń' | 'Ň' => Some(b"N"),
        'ő' | 'ō' => Some(b"o"),
        'Ő' | 'Ō' => Some(b"O"),
        'ř' | 'ŕ' => Some(b"r"),
        'Ř' | 'Ŕ' => Some(b"R"),
        'ś' | 'ș' | 'ş' => Some(b"s"),
        'Ś' | 'Ș' | 'Ş' => Some(b"S"),
        'ť' | 'ț' | 'ţ' => Some(b"t"),
        'Ť' | 'Ț' | 'Ţ' => Some(b"T"),
        'ű' | 'ū' => Some(b"u"),
        'Ű' | 'Ū' => Some(b"U"),
        'ź' | 'ż' => Some(b"z"),
        'Ź' | 'Ż' => Some(b"Z"),
        _ => None,
    }
}

pub fn pdf_escape(bytes: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    for byte in bytes {
        if matches!(byte, b'\\' | b'(' | b')') {
            output.push(b'\\');
        }
        output.push(*byte);
    }
    output
}
