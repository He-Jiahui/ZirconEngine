use super::*;
use zr_rhi::{DeviceGeneration, DeviceId, RenderResourceHandleAllocator, TextureFormat};

#[test]
fn diagnostic_boundary_texture_row_narrowing_returns_invalid_copy() {
    let handles =
        RenderResourceHandleAllocator::new(DeviceId::new(17), DeviceGeneration::initial());
    let texture = handles.allocate_texture().unwrap();
    let desc = TextureDesc::new(
        "diagnostic row narrowing",
        u32::MAX,
        1,
        TextureFormat::Rgba8Unorm,
        TextureUsage::COPY_SRC,
    );

    assert!(matches!(
        ensure_diagnostic_texture_readback_region(
            texture,
            &desc,
            TextureCopyRegion::new(u32::MAX, 1),
        ),
        Err(RhiError::InvalidCopy { reason }) if reason.contains("staging layout overflowed")
    ));
}

#[test]
fn diagnostic_boundary_depth_direct_copy_returns_invalid_copy() {
    let handles =
        RenderResourceHandleAllocator::new(DeviceId::new(17), DeviceGeneration::initial());
    let texture = handles.allocate_texture().unwrap();
    let desc = TextureDesc::new(
        "diagnostic depth",
        1,
        1,
        TextureFormat::Depth32Float,
        TextureUsage::COPY_SRC,
    );

    assert!(matches!(
        ensure_diagnostic_texture_readback_region(texture, &desc, TextureCopyRegion::new(1, 1)),
        Err(RhiError::InvalidCopy { reason }) if reason.contains("depth/stencil conversion")
    ));
}

#[test]
fn diagnostic_boundary_buffer_invalid_ranges_return_typed_errors() {
    let handles =
        RenderResourceHandleAllocator::new(DeviceId::new(17), DeviceGeneration::initial());
    let buffer = handles.allocate_buffer().unwrap();
    let desc = BufferDesc::new("diagnostic buffer", 16, BufferUsage::COPY_SRC);

    assert!(matches!(
        ensure_diagnostic_readback_range(buffer, &desc, 0, 0),
        Err(RhiError::InvalidCopy { .. })
    ));
    assert!(matches!(
        ensure_diagnostic_readback_range(buffer, &desc, 12, 8),
        Err(RhiError::ReadbackOutOfRange { .. })
    ));
}
