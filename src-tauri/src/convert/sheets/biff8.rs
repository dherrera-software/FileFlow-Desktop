//! Escritor de hojas de cálculo XLS en formato BIFF8 dentro de un contenedor OLE2.

use super::{CellData, SheetData};

pub fn write_xls(sheets: &[SheetData]) -> Result<Vec<u8>, String> {
    let _sheet = sheets.first().ok_or("No hay hojas disponibles.")?;
    let mut out = Vec::new();

    // Función auxiliar: escribe un registro BIFF8
    fn rec(out: &mut Vec<u8>, code: u16, data: &[u8]) {
        out.extend_from_slice(&code.to_le_bytes());
        out.extend_from_slice(&(data.len() as u16).to_le_bytes());
        out.extend_from_slice(data);
    }

    // Función auxiliar: codifica una cadena como BIFF8 Unicode (1-byte length + flag byte + UTF-16LE)
    fn biff_str(s: &str) -> Vec<u8> {
        let utf16: Vec<u16> = s.encode_utf16().collect();
        let len = utf16.len().min(0xFFFF) as u16;
        let mut v = Vec::new();
        v.extend_from_slice(&len.to_le_bytes());
        v.push(1u8); // fHighByte = 1 (Unicode)
        for c in &utf16[..len as usize] {
            v.extend_from_slice(&c.to_le_bytes());
        }
        v
    }

    let mut biff = Vec::new();

    // Registro BOF (BIFF8: code=0x0809, version=0x0600, type=0x0005 workbook)
    {
        let mut d = Vec::new();
        d.extend_from_slice(&0x0600u16.to_le_bytes());
        d.extend_from_slice(&0x0005u16.to_le_bytes());
        d.extend_from_slice(&0x0DBBu16.to_le_bytes());
        d.extend_from_slice(&0x07CCu16.to_le_bytes());
        d.extend_from_slice(&0x0041u32.to_le_bytes());
        rec(&mut biff, 0x0809, &d);
    }

    // CODEPAGE (UTF-16LE: 1200 = 0x04B0)
    rec(&mut biff, 0x0042, &0x04B0u16.to_le_bytes());

    // BOUNDSHEET para cada hoja
    let mut bound_offsets: Vec<usize> = Vec::new();
    for sheet_data in sheets {
        let pos = biff.len() + 4;
        bound_offsets.push(pos);
        let mut d = vec![0u8; 4]; // desplazamiento provisional
        d.push(0x00); // estado: visible
        d.push(0x00); // tipo: worksheet
        let name_utf16: Vec<u16> = sheet_data.name.encode_utf16().collect();
        let name_len = name_utf16.len().min(31) as u8;
        d.push(name_len);
        d.push(0x01); // fHighByte
        for c in &name_utf16[..name_len as usize] {
            d.extend_from_slice(&c.to_le_bytes());
        }
        rec(&mut biff, 0x0085, &d);
    }

    // EOF para variables globales del libro
    rec(&mut biff, 0x000A, &[]);

    // Escribir cada subflujo de hoja y parchar BOUNDSHEET
    for (idx, sheet_data) in sheets.iter().enumerate() {
        let sheet_start = biff.len();

        let off_pos = bound_offsets[idx];
        let off_bytes = (sheet_start as u32).to_le_bytes();
        biff[off_pos..off_pos + 4].copy_from_slice(&off_bytes);

        // BOF de hoja
        {
            let mut d = Vec::new();
            d.extend_from_slice(&0x0600u16.to_le_bytes());
            d.extend_from_slice(&0x0010u16.to_le_bytes()); // worksheet
            d.extend_from_slice(&0x0DBBu16.to_le_bytes());
            d.extend_from_slice(&0x07CCu16.to_le_bytes());
            d.extend_from_slice(&0x0041u32.to_le_bytes());
            rec(&mut biff, 0x0809, &d);
        }

        // Celdas
        for (row_idx, row) in sheet_data.rows.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                let r = row_idx as u16;
                let c = col_idx as u16;
                match cell {
                    CellData::Text(value) => {
                        let mut d = Vec::new();
                        d.extend_from_slice(&r.to_le_bytes());
                        d.extend_from_slice(&c.to_le_bytes());
                        d.extend_from_slice(&0x0000u16.to_le_bytes());
                        d.extend_from_slice(&biff_str(value));
                        rec(&mut biff, 0x0204, &d);
                    }
                    CellData::Number(value) => {
                        let mut d = Vec::new();
                        d.extend_from_slice(&r.to_le_bytes());
                        d.extend_from_slice(&c.to_le_bytes());
                        d.extend_from_slice(&0x0000u16.to_le_bytes());
                        d.extend_from_slice(&value.to_le_bytes());
                        rec(&mut biff, 0x0203, &d);
                    }
                    CellData::Bool(value) => {
                        let mut d = Vec::new();
                        d.extend_from_slice(&r.to_le_bytes());
                        d.extend_from_slice(&c.to_le_bytes());
                        d.extend_from_slice(&0x0000u16.to_le_bytes());
                        d.push(*value as u8);
                        d.push(0x00);
                        rec(&mut biff, 0x0205, &d);
                    }
                    CellData::Empty => {}
                }
            }
        }

        // EOF de hoja
        rec(&mut biff, 0x000A, &[]);
    }

    ole2_wrap(&biff, &mut out)?;
    Ok(out)
}

fn ole2_wrap(biff: &[u8], out: &mut Vec<u8>) -> Result<(), String> {
    const SECTOR_SIZE: usize = 512;

    let biff_sectors = biff.len().div_ceil(SECTOR_SIZE).max(8);
    let mut biff_padded = biff.to_vec();
    biff_padded.resize(biff_sectors * SECTOR_SIZE, 0);

    let biff_start_sector = 1usize;
    let dir_sector = biff_start_sector + biff_sectors;
    let total_sectors = dir_sector + 1;

    let mut fat = vec![0xFFFFFFFFu32; 128];
    fat[0] = 0xFFFFFFFD;
    for i in 0..biff_sectors {
        fat[biff_start_sector + i] = if i + 1 < biff_sectors {
            (biff_start_sector + i + 1) as u32
        } else {
            0xFFFFFFFE
        };
    }
    fat[dir_sector] = 0xFFFFFFFE;

    let mut fat_bytes = Vec::with_capacity(SECTOR_SIZE);
    for &v in &fat {
        fat_bytes.extend_from_slice(&v.to_le_bytes());
    }

    let mut dir_sector_bytes = Vec::with_capacity(SECTOR_SIZE);

    #[allow(clippy::too_many_arguments)]
    fn dir_entry(
        name: &str,
        obj_type: u8,
        color: u8,
        child: u32,
        sibling_l: u32,
        sibling_r: u32,
        start: u32,
        size: u32,
    ) -> Vec<u8> {
        let mut e = vec![0u8; 128];
        let name_utf16: Vec<u16> = name.encode_utf16().collect();
        let name_len = name_utf16.len().min(31);
        for (i, &c) in name_utf16[..name_len].iter().enumerate() {
            e[i * 2] = (c & 0xFF) as u8;
            e[i * 2 + 1] = (c >> 8) as u8;
        }
        let name_byte_len = ((name_len + 1) * 2) as u16;
        e[64..66].copy_from_slice(&name_byte_len.to_le_bytes());
        e[66] = obj_type;
        e[67] = color;
        e[68..72].copy_from_slice(&sibling_l.to_le_bytes());
        e[72..76].copy_from_slice(&sibling_r.to_le_bytes());
        e[76..80].copy_from_slice(&child.to_le_bytes());
        e[116..120].copy_from_slice(&start.to_le_bytes());
        e[120..124].copy_from_slice(&size.to_le_bytes());
        e
    }

    let no_sibling = 0xFFFFFFFFu32;
    let biff_size = (biff_sectors * SECTOR_SIZE) as u32;

    dir_sector_bytes.extend(dir_entry(
        "Root Entry",
        5,
        1,
        1,
        no_sibling,
        no_sibling,
        0xFFFFFFFE,
        0,
    ));
    dir_sector_bytes.extend(dir_entry(
        "Workbook",
        2,
        1,
        no_sibling,
        no_sibling,
        no_sibling,
        biff_start_sector as u32,
        biff_size,
    ));
    dir_sector_bytes.extend(dir_entry(
        "", 0, 1, no_sibling, no_sibling, no_sibling, 0xFFFFFFFE, 0,
    ));
    dir_sector_bytes.extend(dir_entry(
        "", 0, 1, no_sibling, no_sibling, no_sibling, 0xFFFFFFFE, 0,
    ));

    let mut header = vec![0u8; 512];
    header[0..8].copy_from_slice(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]);
    header[24..26].copy_from_slice(&0x003Eu16.to_le_bytes());
    header[26..28].copy_from_slice(&0x0003u16.to_le_bytes());
    header[28..30].copy_from_slice(&0xFFFEu16.to_le_bytes());
    header[30..32].copy_from_slice(&0x0009u16.to_le_bytes());
    header[32..34].copy_from_slice(&0x0006u16.to_le_bytes());
    header[44..48].copy_from_slice(&1u32.to_le_bytes());
    header[48..52].copy_from_slice(&(dir_sector as u32).to_le_bytes());
    header[56..60].copy_from_slice(&0x1000u32.to_le_bytes());
    header[60..64].copy_from_slice(&0xFFFFFFFEu32.to_le_bytes());
    header[64..68].copy_from_slice(&0u32.to_le_bytes());
    header[68..72].copy_from_slice(&0xFFFFFFFEu32.to_le_bytes());
    header[72..76].copy_from_slice(&0u32.to_le_bytes());
    header[76..80].copy_from_slice(&0u32.to_le_bytes());
    for i in 1..109usize {
        let off = 76 + i * 4;
        if off + 4 <= 512 {
            header[off..off + 4].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes());
        }
    }

    if total_sectors > 128 {
        return Err(
            "El archivo XLS generado es demasiado grande para el escritor mínimo.".to_string(),
        );
    }

    out.extend_from_slice(&header);
    out.extend_from_slice(&fat_bytes);
    out.extend_from_slice(&biff_padded);
    out.extend_from_slice(&dir_sector_bytes);

    Ok(())
}
