use super::{IblBakeKey, IBL_BAKE_ARTIFACT_HEADER_SIZE};

pub(super) fn write_ibl_bake_key(
    bytes: &mut [u8; IBL_BAKE_ARTIFACT_HEADER_SIZE],
    cursor: &mut usize,
    bake_key: IblBakeKey,
) {
    write_u32(bytes, cursor, bake_key.source_kind);
    write_u64(bytes, cursor, bake_key.source_revision);
    for value in bake_key.horizon_color {
        write_u32(bytes, cursor, value);
    }
    for value in bake_key.zenith_color {
        write_u32(bytes, cursor, value);
    }
    for value in bake_key.ground_color {
        write_u32(bytes, cursor, value);
    }
    for value in bake_key.source_hash {
        write_u32(bytes, cursor, value);
    }
}

pub(super) fn read_ibl_bake_key(bytes: &[u8], cursor: &mut usize) -> IblBakeKey {
    IblBakeKey {
        source_kind: read_u32(bytes, cursor),
        source_revision: read_u64(bytes, cursor),
        horizon_color: read_u32_array(bytes, cursor),
        zenith_color: read_u32_array(bytes, cursor),
        ground_color: read_u32_array(bytes, cursor),
        source_hash: read_u32_array(bytes, cursor),
    }
}

pub(super) fn write_bytes(
    bytes: &mut [u8; IBL_BAKE_ARTIFACT_HEADER_SIZE],
    cursor: &mut usize,
    value: &[u8],
) {
    let next = *cursor + value.len();
    bytes[*cursor..next].copy_from_slice(value);
    *cursor = next;
}

pub(super) fn write_u32(
    bytes: &mut [u8; IBL_BAKE_ARTIFACT_HEADER_SIZE],
    cursor: &mut usize,
    value: u32,
) {
    write_bytes(bytes, cursor, &value.to_le_bytes());
}

pub(super) fn write_u64(
    bytes: &mut [u8; IBL_BAKE_ARTIFACT_HEADER_SIZE],
    cursor: &mut usize,
    value: u64,
) {
    write_bytes(bytes, cursor, &value.to_le_bytes());
}

pub(super) fn read_bytes<const N: usize>(bytes: &[u8], cursor: &mut usize) -> [u8; N] {
    let mut value = [0; N];
    let next = *cursor + N;
    value.copy_from_slice(&bytes[*cursor..next]);
    *cursor = next;
    value
}

pub(super) fn read_u32(bytes: &[u8], cursor: &mut usize) -> u32 {
    u32::from_le_bytes(read_bytes(bytes, cursor))
}

pub(super) fn read_u64(bytes: &[u8], cursor: &mut usize) -> u64 {
    u64::from_le_bytes(read_bytes(bytes, cursor))
}

fn read_u32_array(bytes: &[u8], cursor: &mut usize) -> [u32; 4] {
    [
        read_u32(bytes, cursor),
        read_u32(bytes, cursor),
        read_u32(bytes, cursor),
        read_u32(bytes, cursor),
    ]
}
