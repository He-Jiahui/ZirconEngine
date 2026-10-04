const SOURCE: &str = include_str!("../create_sprite.rs");

fn production_source() -> &'static str {
    SOURCE
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("viewport icon sprite source should retain a test-module boundary")
}

#[test]
fn viewport_icon_sprite_prepares_owned_copy_upload_without_raw_queue_writes() {
    let source = production_source();

    assert!(!source.contains("wgpu::Queue"));
    assert!(!source.contains("write_texture"));
    assert!(source.contains("WgpuTextureUpload::from_owned_bytes("));
    assert!(source.contains("checked_mul(RGBA8_BYTES_PER_TEXEL)"));
    assert!(source.contains("checked_mul(height)"));
}
