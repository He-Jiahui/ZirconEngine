pub(super) const fn shader_source_content_hash(source: &str) -> u64 {
    let bytes = source.as_bytes();
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    let mut index = 0;
    while index < bytes.len() {
        hash ^= bytes[index] as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        index += 1;
    }
    hash
}

pub(super) const fn shader_source_pair_content_identity(first: &str, second: &str) -> [u32; 4] {
    let first_hash = shader_source_content_hash(first);
    let second_hash = shader_source_content_hash(second);
    [
        first_hash as u32,
        (first_hash >> 32) as u32,
        second_hash as u32,
        (second_hash >> 32) as u32,
    ]
}
