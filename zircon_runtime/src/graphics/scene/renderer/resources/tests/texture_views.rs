use crate::rhi::{TextureDesc, TextureDimension, TextureFormat, TextureUsage};

use super::validate_texture_view_descriptor;

#[test]
fn d3_texture_view_rejects_z_slice_encoded_as_array_layer() {
    let desc = TextureDesc::new(
        "volume",
        16,
        16,
        TextureFormat::Rgba16Float,
        TextureUsage::SAMPLED,
    )
    .with_dimension(TextureDimension::D3)
    .with_depth(8);
    let view = wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D3),
        base_array_layer: 1,
        array_layer_count: Some(1),
        ..Default::default()
    };

    let error = validate_texture_view_descriptor("volume", &desc, &view)
        .expect_err("D3 Z slices are not WGPU array layers");

    assert!(error.contains("addressable array layers 1"), "{error}");
}
