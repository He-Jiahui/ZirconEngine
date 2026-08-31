pub(super) fn write_bytes<const N: usize>(output: &mut [u8], cursor: &mut usize, value: &[u8; N]) {
    output[*cursor..*cursor + N].copy_from_slice(value);
    *cursor += N;
}

pub(super) fn write_u32(output: &mut [u8], cursor: &mut usize, value: u32) {
    write_bytes(output, cursor, &value.to_le_bytes());
}

pub(super) fn write_u64(output: &mut [u8], cursor: &mut usize, value: u64) {
    write_bytes(output, cursor, &value.to_le_bytes());
}

pub(super) fn read_bytes<const N: usize>(input: &[u8], cursor: &mut usize) -> [u8; N] {
    let mut value = [0; N];
    value.copy_from_slice(&input[*cursor..*cursor + N]);
    *cursor += N;
    value
}

pub(super) fn read_u32(input: &[u8], cursor: &mut usize) -> u32 {
    u32::from_le_bytes(read_bytes(input, cursor))
}

pub(super) fn read_u64(input: &[u8], cursor: &mut usize) -> u64 {
    u64::from_le_bytes(read_bytes(input, cursor))
}
