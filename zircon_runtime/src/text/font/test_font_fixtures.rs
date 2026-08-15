use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FONT_FIXTURE_ID: AtomicU64 = AtomicU64::new(1);
const TEXT_FONT_FIXTURE_WORK_DIRECTORY: &str = ".runtime_text_font_fixture_work";

pub(super) fn write_weight_fixture(label: &str, weight: u16) -> PathBuf {
    let mut bytes = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf"),
    )
    .unwrap();
    patch_os2_weight(&mut bytes, weight);
    let path = unique_font_fixture_path(&format!("font-weight-{label}-{weight}"), "ttf");
    std::fs::write(&path, bytes).unwrap();
    path
}

pub(super) fn write_ttc_fixture() -> PathBuf {
    let regular = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf"),
    )
    .unwrap();
    let mut bold = regular.clone();
    patch_os2_weight(&mut bold, 700);
    let collection = ttc_from_fonts(&[regular.as_slice(), bold.as_slice()]);
    let path = unique_font_fixture_path("font-collection", "ttc");
    std::fs::write(&path, collection).unwrap();
    path
}

pub(super) fn unique_font_fixture_path(label: &str, extension: &str) -> PathBuf {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime manifest must have a workspace parent");
    let root = workspace_root
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("text")
        .join(TEXT_FONT_FIXTURE_WORK_DIRECTORY);
    std::fs::create_dir_all(&root).expect("workspace text font fixture directory should exist");
    let fixture_id = NEXT_FONT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    root.join(format!(
        "zircon-runtime-text-{label}-{}-{fixture_id}.{extension}",
        std::process::id()
    ))
}

fn ttc_from_fonts(fonts: &[&[u8]]) -> Vec<u8> {
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

fn patch_os2_weight(bytes: &mut [u8], weight: u16) {
    let offset = sfnt_table_offset(bytes, b"OS/2").expect("font fixture must contain OS/2 table");
    bytes[offset + 4..offset + 6].copy_from_slice(&weight.to_be_bytes());
}

fn sfnt_table_offset(bytes: &[u8], tag: &[u8; 4]) -> Option<usize> {
    let table_count = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
    (0..table_count).find_map(|index| {
        let record_offset = 12 + index * 16;
        (&bytes[record_offset..record_offset + 4] == tag)
            .then(|| read_u32(bytes, record_offset + 8) as usize)
    })
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn pad_to_four(bytes: &mut Vec<u8>) {
    while bytes.len() % 4 != 0 {
        bytes.push(0);
    }
}

#[cfg(test)]
mod tests {
    use super::{unique_font_fixture_path, TEXT_FONT_FIXTURE_WORK_DIRECTORY};

    #[test]
    fn font_fixture_paths_stay_under_workspace_text_artifacts() {
        let path = unique_font_fixture_path("fixture-root", "ttf");
        let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("zircon_runtime manifest must have a workspace parent");

        assert!(path.starts_with(
            workspace_root
                .join("docs")
                .join("tests")
                .join("runtime")
                .join("text")
                .join(TEXT_FONT_FIXTURE_WORK_DIRECTORY)
        ));
    }
}
