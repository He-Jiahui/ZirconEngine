use super::{rgba8_upload_layout, validate_lut_view_kind};
use crate::core::framework::render::TextureViewKind;

#[test]
fn lut_resource_accepts_only_2d_strips_and_3d_volumes() {
    assert!(validate_lut_view_kind(TextureViewKind::D2).is_ok());
    assert!(validate_lut_view_kind(TextureViewKind::D3).is_ok());

    for unsupported in [
        TextureViewKind::D1,
        TextureViewKind::D2Array,
        TextureViewKind::Cube,
        TextureViewKind::CubeArray,
    ] {
        assert_eq!(
            validate_lut_view_kind(unsupported),
            Err("post-process LUT textures must be a 2d strip or 3d volume")
        );
    }
}

#[test]
fn rgba8_upload_layout_describes_one_contiguous_volume_copy() {
    let layout = rgba8_upload_layout(32, 32, 32).expect("valid LUT upload layout");

    assert_eq!(layout.bytes_per_row, 128);
    assert_eq!(layout.rows_per_image, 32);
    assert_eq!(layout.depth_or_array_layers, 32);
    assert_eq!(layout.byte_len, 131_072);
}

#[test]
fn rgba8_upload_layout_rejects_extent_overflow() {
    assert!(rgba8_upload_layout(u32::MAX, 2, 2).is_none());
}

#[test]
fn lut_resource_preparation_has_no_private_queue_write() {
    let production = include_str!("../post_process_lut_texture_resource.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("LUT resource test boundary");

    assert!(production.contains("let rgba: Arc<[u8]> = Arc::from("));
    assert!(production.contains("WgpuTextureUpload::new("));
    assert!(production.contains(".with_depth_or_array_layers("));
    assert!(production.contains("WgpuTextureUploadBatch::from(upload)"));
    assert!(!production.contains("queue.write_texture"));
    assert!(!production.contains("wgpu::Queue"));
}
