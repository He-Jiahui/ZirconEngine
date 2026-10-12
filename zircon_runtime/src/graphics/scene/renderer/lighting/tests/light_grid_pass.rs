#[test]
fn light_grid_uses_one_packed_payload_and_no_direct_queue_write() {
    let source = include_str!("../light_grid_pass.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("light-grid production source");

    assert!(!production.contains("queue.write_buffer"));
    assert_eq!(production.matches("let payload: Arc<[u8]>").count(), 1);
    assert_eq!(production.matches("WgpuBufferUpload::new(").count(), 1);
    assert!(production.contains("light_grid_params_buffer"));
    assert!(production.contains("light_zbins_buffer"));
    assert!(production.contains("light_tile_masks_buffer"));
    assert!(production.contains("binding.offset"));
    assert!(production.contains("BufferBinding<'_>"));
}
