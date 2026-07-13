pub fn zrpack_hash(bytes: &[u8]) -> [u8; 32] {
    zircon_runtime::asset::pack::zrpack_content_hash(bytes)
}
