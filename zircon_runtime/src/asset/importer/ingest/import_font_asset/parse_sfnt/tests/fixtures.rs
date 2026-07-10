use std::path::Path;

pub(super) fn fira_regular() -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("fonts")
            .join("FiraSans-Regular.ttf"),
    )
    .unwrap()
}

pub(super) fn patch_os2_weight(bytes: &mut [u8], weight: u16) {
    let offset = sfnt_table_offset(bytes, b"OS/2").unwrap() + 4;
    bytes[offset..offset + 2].copy_from_slice(&weight.to_be_bytes());
}

pub(super) fn variable_font() -> Vec<u8> {
    let mut tables = sfnt_tables(&fira_regular());
    tables.retain(|table| table.tag != *b"fvar");
    tables.push(SfntTable {
        tag: *b"fvar",
        data: fvar_table(),
    });
    build_sfnt(tables)
}

pub(super) fn ttc_from_fonts(fonts: &[&[u8]]) -> Vec<u8> {
    let header_len = 12 + fonts.len() * 4;
    let mut output = vec![0; header_len];
    output[0..4].copy_from_slice(b"ttcf");
    output[4..8].copy_from_slice(&0x0001_0000_u32.to_be_bytes());
    output[8..12].copy_from_slice(&(fonts.len() as u32).to_be_bytes());

    for (font_index, font) in fonts.iter().enumerate() {
        pad_to_four(&mut output);
        let directory_offset = output.len();
        let offset_slot = 12 + font_index * 4;
        output[offset_slot..offset_slot + 4]
            .copy_from_slice(&(directory_offset as u32).to_be_bytes());

        let table_count = u16::from_be_bytes([font[4], font[5]]) as usize;
        let directory_len = 12 + table_count * 16;
        output.extend_from_slice(&font[..directory_len]);
        for table_index in 0..table_count {
            let record_offset = 12 + table_index * 16;
            let source_offset = read_u32(font, record_offset + 8) as usize;
            let source_len = read_u32(font, record_offset + 12) as usize;
            pad_to_four(&mut output);
            let target_offset = output.len();
            output.extend_from_slice(&font[source_offset..source_offset + source_len]);
            output[directory_offset + record_offset + 8..directory_offset + record_offset + 12]
                .copy_from_slice(&(target_offset as u32).to_be_bytes());
        }
    }
    output
}

fn sfnt_table_offset(bytes: &[u8], tag: &[u8; 4]) -> Option<usize> {
    let table_count = u16::from_be_bytes([*bytes.get(4)?, *bytes.get(5)?]) as usize;
    for table_index in 0..table_count {
        let record_offset = 12 + table_index * 16;
        if bytes.get(record_offset..record_offset + 4)? != &tag[..] {
            continue;
        }
        return Some(read_u32(bytes, record_offset + 8) as usize);
    }
    None
}

#[derive(Clone)]
struct SfntTable {
    tag: [u8; 4],
    data: Vec<u8>,
}

fn sfnt_tables(bytes: &[u8]) -> Vec<SfntTable> {
    let table_count = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
    (0..table_count)
        .map(|index| {
            let record = 12 + index * 16;
            let offset = read_u32(bytes, record + 8) as usize;
            let len = read_u32(bytes, record + 12) as usize;
            SfntTable {
                tag: bytes[record..record + 4].try_into().unwrap(),
                data: bytes[offset..offset + len].to_vec(),
            }
        })
        .collect()
}

fn build_sfnt(mut tables: Vec<SfntTable>) -> Vec<u8> {
    tables.sort_by_key(|table| table.tag);
    let table_count = tables.len();
    let mut output = vec![0; 12 + table_count * 16];
    output[0..4].copy_from_slice(&0x0001_0000_u32.to_be_bytes());
    output[4..6].copy_from_slice(&(table_count as u16).to_be_bytes());
    let entry_selector = usize::BITS as usize - 1 - table_count.leading_zeros() as usize;
    let search_range = (1usize << entry_selector) * 16;
    output[6..8].copy_from_slice(&(search_range as u16).to_be_bytes());
    output[8..10].copy_from_slice(&(entry_selector as u16).to_be_bytes());
    output[10..12].copy_from_slice(&((table_count * 16 - search_range) as u16).to_be_bytes());

    for (index, table) in tables.iter().enumerate() {
        pad_to_four(&mut output);
        let offset = output.len();
        output.extend_from_slice(&table.data);
        let record = 12 + index * 16;
        output[record..record + 4].copy_from_slice(&table.tag);
        output[record + 4..record + 8].copy_from_slice(&table_checksum(&table.data).to_be_bytes());
        output[record + 8..record + 12].copy_from_slice(&(offset as u32).to_be_bytes());
        output[record + 12..record + 16].copy_from_slice(&(table.data.len() as u32).to_be_bytes());
    }
    output
}

fn fvar_table() -> Vec<u8> {
    let mut table = Vec::with_capacity(46);
    table.extend_from_slice(&1u16.to_be_bytes());
    table.extend_from_slice(&0u16.to_be_bytes());
    table.extend_from_slice(&16u16.to_be_bytes());
    table.extend_from_slice(&2u16.to_be_bytes());
    table.extend_from_slice(&1u16.to_be_bytes());
    table.extend_from_slice(&20u16.to_be_bytes());
    table.extend_from_slice(&1u16.to_be_bytes());
    table.extend_from_slice(&10u16.to_be_bytes());
    table.extend_from_slice(b"wght");
    table.extend_from_slice(&fixed(100.0).to_be_bytes());
    table.extend_from_slice(&fixed(400.0).to_be_bytes());
    table.extend_from_slice(&fixed(900.0).to_be_bytes());
    table.extend_from_slice(&0u16.to_be_bytes());
    table.extend_from_slice(&256u16.to_be_bytes());
    table.extend_from_slice(&257u16.to_be_bytes());
    table.extend_from_slice(&0u16.to_be_bytes());
    table.extend_from_slice(&fixed(650.0).to_be_bytes());
    table.extend_from_slice(&258u16.to_be_bytes());
    table
}

fn fixed(value: f32) -> i32 {
    (value * 65_536.0).round() as i32
}

fn table_checksum(data: &[u8]) -> u32 {
    data.chunks(4).fold(0u32, |checksum, chunk| {
        let mut word = [0u8; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        checksum.wrapping_add(u32::from_be_bytes(word))
    })
}

fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(data[offset..offset + 4].try_into().unwrap())
}

fn pad_to_four(data: &mut Vec<u8>) {
    while data.len() % 4 != 0 {
        data.push(0);
    }
}
