use crate::rhi::{
    BufferDesc, BufferUsage, CommandList, RenderDevice, RenderQueueClass, TextureCopyRegion,
    TextureDesc, TextureDimension, TextureFormat, TextureUsage,
};
use crate::rhi_wgpu::WgpuRenderDevice;

#[test]
fn wgpu_rhi_copy_buffer_to_texture_preserves_bytes() {
    let device = WgpuRenderDevice::new_headless();
    let upload = device
        .create_buffer(&BufferDesc::new(
            "texture-upload",
            16,
            BufferUsage::STAGING_WRITE | BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let texture = device
        .create_texture(&TextureDesc::new(
            "albedo",
            2,
            2,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::COPY_DST | TextureUsage::COPY_SRC,
        ))
        .unwrap();
    let pixels = vec![1, 2, 3, 4, 10, 20, 30, 40, 5, 6, 7, 8, 50, 60, 70, 80];

    device.write_buffer(upload, 0, &pixels).unwrap();

    let mut command_list = device
        .create_command_list(RenderQueueClass::Copy, "texture-upload")
        .unwrap();
    command_list.copy_buffer_to_texture(upload, texture, 0, 8, TextureCopyRegion::new(2, 2));
    let fence = device.submit(command_list).unwrap();
    assert!(device.is_fence_complete(fence).unwrap());

    assert_eq!(device.read_texture(texture).unwrap(), pixels);
}

#[test]
fn wgpu_rhi_copy_texture_to_buffer_preserves_bytes() {
    let device = WgpuRenderDevice::new_headless();
    let upload = device
        .create_buffer(&BufferDesc::new(
            "texture-upload",
            16,
            BufferUsage::STAGING_WRITE | BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let texture = device
        .create_texture(&TextureDesc::new(
            "albedo",
            2,
            2,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::COPY_DST | TextureUsage::COPY_SRC,
        ))
        .unwrap();
    let readback = device
        .create_buffer(&BufferDesc::new(
            "texture-readback",
            24,
            BufferUsage::COPY_DST | BufferUsage::STAGING_READ,
        ))
        .unwrap();
    let pixels = vec![1, 2, 3, 4, 10, 20, 30, 40, 5, 6, 7, 8, 50, 60, 70, 80];

    device.write_buffer(upload, 0, &pixels).unwrap();

    let mut command_list = device
        .create_command_list(RenderQueueClass::Copy, "texture-roundtrip")
        .unwrap();
    command_list.copy_buffer_to_texture(upload, texture, 0, 8, TextureCopyRegion::new(2, 2));
    command_list.copy_texture_to_buffer(texture, readback, 4, 8, TextureCopyRegion::new(2, 2));
    let fence = device.submit(command_list).unwrap();
    assert!(device.is_fence_complete(fence).unwrap());

    assert_eq!(
        device.read_buffer(readback, 0, 24).unwrap(),
        vec![0, 0, 0, 0, 1, 2, 3, 4, 10, 20, 30, 40, 5, 6, 7, 8, 50, 60, 70, 80, 0, 0, 0, 0]
    );
}

#[test]
fn wgpu_rhi_texture_copy_region_targets_mip_and_array_layer() {
    let device = WgpuRenderDevice::new_headless();
    let upload = device
        .create_buffer(&BufferDesc::new(
            "array-mip-upload",
            16,
            BufferUsage::STAGING_WRITE | BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let texture = device
        .create_texture(
            &TextureDesc::new(
                "array-mip-texture",
                4,
                4,
                TextureFormat::Rgba8Unorm,
                TextureUsage::COPY_DST | TextureUsage::COPY_SRC,
            )
            .with_dimension(TextureDimension::D2Array)
            .with_array_layers(2)
            .with_mip_levels(2),
        )
        .unwrap();
    let readback = device
        .create_buffer(&BufferDesc::new(
            "array-mip-readback",
            16,
            BufferUsage::COPY_DST | BufferUsage::STAGING_READ,
        ))
        .unwrap();
    let pixels = vec![1, 2, 3, 4, 10, 20, 30, 40, 5, 6, 7, 8, 50, 60, 70, 80];
    let region = TextureCopyRegion::new(2, 2)
        .with_mip_level(1)
        .with_origin(0, 0, 1);

    device.write_buffer(upload, 0, &pixels).unwrap();

    let mut command_list = device
        .create_command_list(RenderQueueClass::Copy, "array-mip-copy")
        .unwrap();
    command_list.copy_buffer_to_texture(upload, texture, 0, 8, region);
    command_list.copy_texture_to_buffer(texture, readback, 0, 8, region);
    device.submit(command_list).unwrap();

    let texture_bytes = device.read_texture(texture).unwrap();
    assert_eq!(texture_bytes.len(), 160);
    assert_eq!(&texture_bytes[144..160], pixels.as_slice());
    assert_eq!(device.read_buffer(readback, 0, 16).unwrap(), pixels);
}

#[test]
fn wgpu_rhi_texture_copy_region_targets_cube_face() {
    let device = WgpuRenderDevice::new_headless();
    let upload = device
        .create_buffer(&BufferDesc::new(
            "cube-face-upload",
            16,
            BufferUsage::STAGING_WRITE | BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let texture = device
        .create_texture(
            &TextureDesc::new(
                "skybox-cube",
                2,
                2,
                TextureFormat::Rgba8UnormSrgb,
                TextureUsage::COPY_DST | TextureUsage::COPY_SRC,
            )
            .with_dimension(TextureDimension::Cube)
            .with_array_layers(6),
        )
        .unwrap();
    let readback = device
        .create_buffer(&BufferDesc::new(
            "cube-face-readback",
            16,
            BufferUsage::COPY_DST | BufferUsage::STAGING_READ,
        ))
        .unwrap();
    let pixels = vec![3, 6, 9, 12, 15, 18, 21, 24, 30, 33, 36, 39, 42, 45, 48, 51];
    let region = TextureCopyRegion::new(2, 2).with_origin(0, 0, 4);

    device.write_buffer(upload, 0, &pixels).unwrap();

    let mut command_list = device
        .create_command_list(RenderQueueClass::Copy, "cube-face-copy")
        .unwrap();
    command_list.copy_buffer_to_texture(upload, texture, 0, 8, region);
    command_list.copy_texture_to_buffer(texture, readback, 0, 8, region);
    device.submit(command_list).unwrap();

    let texture_bytes = device.read_texture(texture).unwrap();
    assert_eq!(texture_bytes.len(), 96);
    assert_eq!(&texture_bytes[64..80], pixels.as_slice());
    assert_eq!(device.read_buffer(readback, 0, 16).unwrap(), pixels);
}

#[test]
fn wgpu_rhi_copy_buffer_to_texture_validates_usage_and_range() {
    let device = WgpuRenderDevice::new_headless();
    let invalid_source = device
        .create_buffer(&BufferDesc::new(
            "not-copy-source",
            16,
            BufferUsage::UNIFORM,
        ))
        .unwrap();
    let valid_texture = device
        .create_texture(&TextureDesc::new(
            "copy-destination",
            2,
            2,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::COPY_DST | TextureUsage::COPY_SRC,
        ))
        .unwrap();
    let mut invalid_source_commands = device
        .create_command_list(RenderQueueClass::Copy, "invalid-texture-source")
        .unwrap();
    invalid_source_commands.copy_buffer_to_texture(
        invalid_source,
        valid_texture,
        0,
        8,
        TextureCopyRegion::new(2, 2),
    );

    assert_eq!(
        device.submit(invalid_source_commands).unwrap_err(),
        crate::rhi::RhiError::InvalidBufferUsage {
            buffer: invalid_source.raw(),
            required: BufferUsage::COPY_SRC,
            actual: BufferUsage::UNIFORM,
        }
    );

    let valid_source = device
        .create_buffer(&BufferDesc::new("copy-source", 16, BufferUsage::COPY_SRC))
        .unwrap();
    let invalid_texture = device
        .create_texture(&TextureDesc::new(
            "not-copy-destination",
            2,
            2,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::SAMPLED,
        ))
        .unwrap();
    let mut invalid_texture_commands = device
        .create_command_list(RenderQueueClass::Copy, "invalid-texture-destination")
        .unwrap();
    invalid_texture_commands.copy_buffer_to_texture(
        valid_source,
        invalid_texture,
        0,
        8,
        TextureCopyRegion::new(2, 2),
    );

    assert_eq!(
        device.submit(invalid_texture_commands).unwrap_err(),
        crate::rhi::RhiError::InvalidTextureUsage {
            texture: invalid_texture.raw(),
            required: TextureUsage::COPY_DST,
            actual: TextureUsage::SAMPLED,
        }
    );

    let small_source = device
        .create_buffer(&BufferDesc::new("small-source", 8, BufferUsage::COPY_SRC))
        .unwrap();
    let mut out_of_range_commands = device
        .create_command_list(RenderQueueClass::Copy, "texture-copy-out-of-range")
        .unwrap();
    out_of_range_commands.copy_buffer_to_texture(
        small_source,
        valid_texture,
        0,
        8,
        TextureCopyRegion::new(2, 2),
    );

    assert_eq!(
        device.submit(out_of_range_commands).unwrap_err(),
        crate::rhi::RhiError::BufferToTextureCopyOutOfRange {
            source_buffer: small_source.raw(),
            destination_texture: valid_texture.raw(),
            source_offset: 0,
            bytes_per_row: 8,
            mip_level: 0,
            origin_x: 0,
            origin_y: 0,
            origin_z: 0,
            width: 2,
            height: 2,
        }
    );

    let mut bad_region_commands = device
        .create_command_list(RenderQueueClass::Copy, "texture-copy-bad-region")
        .unwrap();
    bad_region_commands.copy_buffer_to_texture(
        valid_source,
        valid_texture,
        0,
        8,
        TextureCopyRegion::new(2, 2).with_origin(1, 0, 0),
    );
    assert_eq!(
        device.submit(bad_region_commands).unwrap_err(),
        crate::rhi::RhiError::BufferToTextureCopyOutOfRange {
            source_buffer: valid_source.raw(),
            destination_texture: valid_texture.raw(),
            source_offset: 0,
            bytes_per_row: 8,
            mip_level: 0,
            origin_x: 1,
            origin_y: 0,
            origin_z: 0,
            width: 2,
            height: 2,
        }
    );
}

#[test]
fn wgpu_rhi_copy_texture_to_buffer_validates_usage_and_range() {
    let device = WgpuRenderDevice::new_headless();
    let invalid_source = device
        .create_texture(&TextureDesc::new(
            "not-copy-source",
            2,
            2,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::COPY_DST,
        ))
        .unwrap();
    let valid_destination = device
        .create_buffer(&BufferDesc::new(
            "copy-destination",
            16,
            BufferUsage::COPY_DST | BufferUsage::STAGING_READ,
        ))
        .unwrap();
    let mut invalid_source_commands = device
        .create_command_list(RenderQueueClass::Copy, "invalid-texture-source")
        .unwrap();
    invalid_source_commands.copy_texture_to_buffer(
        invalid_source,
        valid_destination,
        0,
        8,
        TextureCopyRegion::new(2, 2),
    );

    assert_eq!(
        device.submit(invalid_source_commands).unwrap_err(),
        crate::rhi::RhiError::InvalidTextureUsage {
            texture: invalid_source.raw(),
            required: TextureUsage::COPY_SRC,
            actual: TextureUsage::COPY_DST,
        }
    );

    let valid_source = device
        .create_texture(&TextureDesc::new(
            "copy-source",
            2,
            2,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::COPY_SRC,
        ))
        .unwrap();
    let invalid_destination = device
        .create_buffer(&BufferDesc::new(
            "not-copy-destination",
            16,
            BufferUsage::STAGING_READ,
        ))
        .unwrap();
    let mut invalid_destination_commands = device
        .create_command_list(RenderQueueClass::Copy, "invalid-buffer-destination")
        .unwrap();
    invalid_destination_commands.copy_texture_to_buffer(
        valid_source,
        invalid_destination,
        0,
        8,
        TextureCopyRegion::new(2, 2),
    );

    assert_eq!(
        device.submit(invalid_destination_commands).unwrap_err(),
        crate::rhi::RhiError::InvalidBufferUsage {
            buffer: invalid_destination.raw(),
            required: BufferUsage::COPY_DST,
            actual: BufferUsage::STAGING_READ,
        }
    );

    let small_destination = device
        .create_buffer(&BufferDesc::new(
            "small-destination",
            8,
            BufferUsage::COPY_DST,
        ))
        .unwrap();
    let mut out_of_range_commands = device
        .create_command_list(RenderQueueClass::Copy, "texture-copy-out-of-range")
        .unwrap();
    out_of_range_commands.copy_texture_to_buffer(
        valid_source,
        small_destination,
        0,
        8,
        TextureCopyRegion::new(2, 2),
    );

    assert_eq!(
        device.submit(out_of_range_commands).unwrap_err(),
        crate::rhi::RhiError::TextureToBufferCopyOutOfRange {
            source_texture: valid_source.raw(),
            destination_buffer: small_destination.raw(),
            destination_offset: 0,
            bytes_per_row: 8,
            mip_level: 0,
            origin_x: 0,
            origin_y: 0,
            origin_z: 0,
            width: 2,
            height: 2,
        }
    );

    let mut bad_region_commands = device
        .create_command_list(RenderQueueClass::Copy, "texture-readback-bad-region")
        .unwrap();
    bad_region_commands.copy_texture_to_buffer(
        valid_source,
        valid_destination,
        0,
        8,
        TextureCopyRegion::new(1, 1).with_mip_level(1),
    );
    assert_eq!(
        device.submit(bad_region_commands).unwrap_err(),
        crate::rhi::RhiError::TextureToBufferCopyOutOfRange {
            source_texture: valid_source.raw(),
            destination_buffer: valid_destination.raw(),
            destination_offset: 0,
            bytes_per_row: 8,
            mip_level: 1,
            origin_x: 0,
            origin_y: 0,
            origin_z: 0,
            width: 1,
            height: 1,
        }
    );
}
