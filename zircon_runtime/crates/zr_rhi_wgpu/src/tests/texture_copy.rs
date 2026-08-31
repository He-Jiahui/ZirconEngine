use crate::DeterministicRhiContractDevice;
use zr_rhi::{
    BufferDesc, BufferUsage, CommandList, RenderDevice, RenderQueueClass, TextureCopyAspect,
    TextureCopyRegion, TextureDesc, TextureDimension, TextureFormat, TextureUsage,
};

#[test]
fn deterministic_rhi_contract_copy_buffer_to_texture_preserves_bytes() {
    let device = DeterministicRhiContractDevice::new_headless();
    let upload = device
        .create_buffer(&BufferDesc::new(
            "texture-upload",
            16,
            BufferUsage::COPY_DST | BufferUsage::COPY_SRC,
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
    let ticket = device.submit(command_list).unwrap();
    assert_eq!(
        device.submission_status(ticket).unwrap(),
        zr_rhi::SubmissionStatus::Completed
    );

    assert_eq!(device.read_texture(texture).unwrap(), pixels);
}

#[test]
fn deterministic_rhi_contract_copy_texture_to_buffer_preserves_bytes() {
    let device = DeterministicRhiContractDevice::new_headless();
    let upload = device
        .create_buffer(&BufferDesc::new(
            "texture-upload",
            16,
            BufferUsage::COPY_DST | BufferUsage::COPY_SRC,
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
    let ticket = device.submit(command_list).unwrap();
    assert_eq!(
        device.submission_status(ticket).unwrap(),
        zr_rhi::SubmissionStatus::Completed
    );

    assert_eq!(
        device.read_buffer(readback, 0, 24).unwrap(),
        vec![0, 0, 0, 0, 1, 2, 3, 4, 10, 20, 30, 40, 5, 6, 7, 8, 50, 60, 70, 80, 0, 0, 0, 0]
    );
}

#[test]
fn deterministic_rhi_contract_reads_depth32_with_an_explicit_depth_aspect() {
    let device = DeterministicRhiContractDevice::new_headless();
    let source = device
        .create_texture(&TextureDesc::new(
            "depth32-copy-source",
            1,
            1,
            TextureFormat::Depth32Float,
            TextureUsage::COPY_SRC | TextureUsage::RENDER_ATTACHMENT,
        ))
        .unwrap();
    let destination = device
        .create_buffer(&BufferDesc::new(
            "depth32-copy-destination",
            256,
            BufferUsage::COPY_DST | BufferUsage::STAGING_READ,
        ))
        .unwrap();
    let region = TextureCopyRegion::new(1, 1).with_aspect(TextureCopyAspect::DepthOnly);

    let mut copy = device
        .create_command_list(RenderQueueClass::Copy, "depth32-to-buffer")
        .unwrap();
    copy.copy_texture_to_buffer(source, destination, 0, 256, region);
    device.submit(copy).unwrap();

    assert_eq!(
        device.read_buffer(destination, 0, 256).unwrap(),
        vec![0; 256]
    );

    let mut wrong_aspect = device
        .create_command_list(RenderQueueClass::Copy, "depth32-wrong-aspect")
        .unwrap();
    wrong_aspect.copy_texture_to_buffer(source, destination, 0, 256, TextureCopyRegion::new(1, 1));
    assert!(matches!(
        device.submit(wrong_aspect),
        Err(zr_rhi::RhiError::InvalidCopy { .. })
    ));
}

#[test]
fn deterministic_rhi_contract_texture_copy_region_targets_mip_and_array_layer() {
    let device = DeterministicRhiContractDevice::new_headless();
    let upload = device
        .create_buffer(&BufferDesc::new(
            "array-mip-upload",
            16,
            BufferUsage::COPY_DST | BufferUsage::COPY_SRC,
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
fn deterministic_rhi_contract_texture_copy_region_targets_cube_face() {
    let device = DeterministicRhiContractDevice::new_headless();
    let upload = device
        .create_buffer(&BufferDesc::new(
            "cube-face-upload",
            16,
            BufferUsage::COPY_DST | BufferUsage::COPY_SRC,
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
fn deterministic_rhi_contract_texture_copy_region_copies_contiguous_array_layers() {
    let device = DeterministicRhiContractDevice::new_headless();
    let upload = device
        .create_buffer(&BufferDesc::new(
            "array-layers-upload",
            32,
            BufferUsage::COPY_DST | BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let texture = device
        .create_texture(
            &TextureDesc::new(
                "array-layers-texture",
                2,
                2,
                TextureFormat::Rgba8Unorm,
                TextureUsage::COPY_DST | TextureUsage::COPY_SRC,
            )
            .with_dimension(TextureDimension::D2Array)
            .with_array_layers(3),
        )
        .unwrap();
    let readback = device
        .create_buffer(&BufferDesc::new(
            "array-layers-readback",
            32,
            BufferUsage::COPY_DST | BufferUsage::STAGING_READ,
        ))
        .unwrap();
    let pixels = (1_u8..=32).collect::<Vec<_>>();
    let region = TextureCopyRegion::new(2, 2)
        .with_origin(0, 0, 1)
        .with_depth_or_array_layers(2);

    device.write_buffer(upload, 0, &pixels).unwrap();

    let mut command_list = device
        .create_command_list(RenderQueueClass::Copy, "array-layers-roundtrip")
        .unwrap();
    command_list.copy_buffer_to_texture(upload, texture, 0, 8, region);
    command_list.copy_texture_to_buffer(texture, readback, 0, 8, region);
    device.submit(command_list).unwrap();

    let texture_bytes = device.read_texture(texture).unwrap();
    assert_eq!(&texture_bytes[16..48], pixels.as_slice());
    assert_eq!(device.read_buffer(readback, 0, 32).unwrap(), pixels);
}

#[test]
fn deterministic_rhi_contract_copy_buffer_to_texture_validates_usage_and_range() {
    let device = DeterministicRhiContractDevice::new_headless();
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
        zr_rhi::RhiError::InvalidBufferUsage {
            buffer: invalid_source.diagnostic_id(),
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
        zr_rhi::RhiError::InvalidTextureUsage {
            texture: invalid_texture.diagnostic_id(),
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
        zr_rhi::RhiError::BufferToTextureCopyOutOfRange {
            source_buffer: small_source.diagnostic_id(),
            destination_texture: valid_texture.diagnostic_id(),
            source_offset: 0,
            bytes_per_row: 8,
            mip_level: 0,
            origin_x: 0,
            origin_y: 0,
            origin_z: 0,
            width: 2,
            height: 2,
            depth_or_array_layers: 1,
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
        zr_rhi::RhiError::BufferToTextureCopyOutOfRange {
            source_buffer: valid_source.diagnostic_id(),
            destination_texture: valid_texture.diagnostic_id(),
            source_offset: 0,
            bytes_per_row: 8,
            mip_level: 0,
            origin_x: 1,
            origin_y: 0,
            origin_z: 0,
            width: 2,
            height: 2,
            depth_or_array_layers: 1,
        }
    );
}

#[test]
fn deterministic_rhi_contract_copy_texture_to_buffer_validates_usage_and_range() {
    let device = DeterministicRhiContractDevice::new_headless();
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
        zr_rhi::RhiError::InvalidTextureUsage {
            texture: invalid_source.diagnostic_id(),
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
        zr_rhi::RhiError::InvalidBufferUsage {
            buffer: invalid_destination.diagnostic_id(),
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
        zr_rhi::RhiError::TextureToBufferCopyOutOfRange {
            source_texture: valid_source.diagnostic_id(),
            destination_buffer: small_destination.diagnostic_id(),
            destination_offset: 0,
            bytes_per_row: 8,
            mip_level: 0,
            origin_x: 0,
            origin_y: 0,
            origin_z: 0,
            width: 2,
            height: 2,
            depth_or_array_layers: 1,
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
        zr_rhi::RhiError::TextureToBufferCopyOutOfRange {
            source_texture: valid_source.diagnostic_id(),
            destination_buffer: valid_destination.diagnostic_id(),
            destination_offset: 0,
            bytes_per_row: 8,
            mip_level: 1,
            origin_x: 0,
            origin_y: 0,
            origin_z: 0,
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        }
    );
}

#[test]
fn deterministic_rhi_contract_copy_texture_to_texture_preserves_subresource_bytes() {
    let device = DeterministicRhiContractDevice::new_headless();
    let upload = device
        .create_buffer(&BufferDesc::new(
            "texture-to-texture-upload",
            64,
            BufferUsage::COPY_DST | BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let source = device
        .create_texture(&TextureDesc::new(
            "texture-to-texture-source",
            4,
            4,
            TextureFormat::Rgba8Unorm,
            TextureUsage::COPY_DST | TextureUsage::COPY_SRC,
        ))
        .unwrap();
    let destination = device
        .create_texture(&TextureDesc::new(
            "texture-to-texture-destination",
            4,
            4,
            TextureFormat::Rgba8Unorm,
            TextureUsage::COPY_DST | TextureUsage::COPY_SRC,
        ))
        .unwrap();
    let readback = device
        .create_buffer(&BufferDesc::new(
            "texture-to-texture-readback",
            16,
            BufferUsage::COPY_DST | BufferUsage::STAGING_READ,
        ))
        .unwrap();
    let pixels: Vec<u8> = (0_u8..16)
        .flat_map(|value| [value, value.wrapping_add(1), value.wrapping_add(2), 255])
        .collect();
    let source_region = TextureCopyRegion::new(2, 2).with_origin(1, 1, 0);
    let destination_region = TextureCopyRegion::new(2, 2).with_origin(0, 2, 0);

    device.write_buffer(upload, 0, &pixels).unwrap();

    let mut commands = device
        .create_command_list(RenderQueueClass::Copy, "texture-to-texture-copy")
        .unwrap();
    commands.copy_buffer_to_texture(upload, source, 0, 16, TextureCopyRegion::new(4, 4));
    commands.copy_texture_to_texture(source, destination, source_region, destination_region);
    commands.copy_texture_to_buffer(destination, readback, 0, 8, destination_region);
    device.submit(commands).unwrap();

    assert_eq!(
        device.read_buffer(readback, 0, 16).unwrap(),
        vec![5, 6, 7, 255, 6, 7, 8, 255, 9, 10, 11, 255, 10, 11, 12, 255,]
    );
}

#[test]
fn deterministic_rhi_contract_copy_texture_to_texture_rejects_incompatible_regions_and_formats() {
    let device = DeterministicRhiContractDevice::new_headless();
    let source = device
        .create_texture(&TextureDesc::new(
            "texture-to-texture-source",
            2,
            2,
            TextureFormat::Rgba8Unorm,
            TextureUsage::COPY_SRC,
        ))
        .unwrap();
    let incompatible_format = device
        .create_texture(&TextureDesc::new(
            "texture-to-texture-srgb-destination",
            2,
            2,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::COPY_DST,
        ))
        .unwrap();
    let compatible_format = device
        .create_texture(&TextureDesc::new(
            "texture-to-texture-destination",
            2,
            2,
            TextureFormat::Rgba8Unorm,
            TextureUsage::COPY_DST,
        ))
        .unwrap();

    let mut incompatible_format_copy = device
        .create_command_list(RenderQueueClass::Copy, "texture-to-texture-format")
        .unwrap();
    incompatible_format_copy.copy_texture_to_texture(
        source,
        incompatible_format,
        TextureCopyRegion::new(2, 2),
        TextureCopyRegion::new(2, 2),
    );
    assert!(matches!(
        device.submit(incompatible_format_copy),
        Err(zr_rhi::RhiError::InvalidCopy { .. })
    ));

    let mut incompatible_region_copy = device
        .create_command_list(RenderQueueClass::Copy, "texture-to-texture-region")
        .unwrap();
    incompatible_region_copy.copy_texture_to_texture(
        source,
        compatible_format,
        TextureCopyRegion::new(1, 2),
        TextureCopyRegion::new(2, 2),
    );
    assert!(matches!(
        device.submit(incompatible_region_copy),
        Err(zr_rhi::RhiError::InvalidCopy { .. })
    ));
}
