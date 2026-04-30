use crate::rhi::{
    BufferDesc, BufferHandle, BufferUsage, CommandList, FenceValue, PipelineDesc, PipelineHandle,
    PipelineKind, RenderDevice, RenderQueueClass, SamplerDesc, SamplerHandle, ShaderModuleDesc,
    ShaderModuleHandle, ShaderStage, TextureDesc, TextureFormat, TextureHandle, TextureUsage,
};
use crate::rhi_wgpu::{WgpuCommandList, WgpuRenderDevice};
use std::path::Path;

#[test]
fn rhi_handles_are_stable_raw_identifiers() {
    assert_eq!(BufferHandle::new(11).raw(), 11);
    assert_eq!(TextureHandle::new(12).raw(), 12);
    assert_eq!(SamplerHandle::new(13).raw(), 13);
    assert_eq!(ShaderModuleHandle::new(14).raw(), 14);
    assert_eq!(PipelineHandle::new(15).raw(), 15);
}

#[test]
fn buffer_and_texture_usage_flags_are_composable() {
    let buffer_usage = BufferUsage::UNIFORM | BufferUsage::STORAGE | BufferUsage::COPY_DST;
    assert!(buffer_usage.contains(BufferUsage::UNIFORM));
    assert!(buffer_usage.contains(BufferUsage::STORAGE));
    assert!(buffer_usage.contains(BufferUsage::COPY_DST));
    assert!(!buffer_usage.contains(BufferUsage::INDEX));

    let texture_usage =
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_SRC;
    assert!(texture_usage.contains(TextureUsage::RENDER_ATTACHMENT));
    assert!(texture_usage.contains(TextureUsage::SAMPLED));
    assert!(texture_usage.contains(TextureUsage::COPY_SRC));
    assert!(!texture_usage.contains(TextureUsage::PRESENT));
}

#[test]
fn wgpu_rhi_device_allocates_stable_resource_handles_and_fences() {
    let device = WgpuRenderDevice::new_headless();

    let buffer = device
        .create_buffer(&BufferDesc::new(
            "frame-uniform",
            256,
            BufferUsage::UNIFORM | BufferUsage::COPY_DST | BufferUsage::STAGING_READ,
        ))
        .unwrap();
    let texture = device
        .create_texture(&TextureDesc::new(
            "scene-color",
            64,
            64,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
        ))
        .unwrap();
    let sampler = device
        .create_sampler(&SamplerDesc::linear("scene-linear"))
        .unwrap();
    let shader = device
        .create_shader_module(&ShaderModuleDesc {
            label: Some("fullscreen".to_string()),
            source: "@compute @workgroup_size(1) fn main() {}".to_string(),
            stage: ShaderStage::Compute,
            entry_point: "main".to_string(),
        })
        .unwrap();
    let pipeline = device
        .create_pipeline(&PipelineDesc::new("compute", PipelineKind::Compute))
        .unwrap();

    assert_ne!(buffer.raw(), texture.raw());
    assert_ne!(sampler.raw(), shader.raw());
    assert_ne!(pipeline.raw(), 0);

    let command_list = device
        .create_command_list(RenderQueueClass::Copy, "copy-upload")
        .unwrap();
    assert_eq!(command_list.queue_class(), RenderQueueClass::Copy);
    assert_eq!(command_list.label(), Some("copy-upload"));

    let fence = device.submit(command_list).unwrap();
    assert_eq!(fence, FenceValue(1));
    assert!(device.is_fence_complete(fence).unwrap());

    let bytes = device.read_buffer(buffer, 0, 16).unwrap();
    assert_eq!(bytes.len(), 16);

    device.destroy_pipeline(pipeline).unwrap();
    device.destroy_shader_module(shader).unwrap();
    device.destroy_sampler(sampler).unwrap();
    device.destroy_texture(texture).unwrap();
    device.destroy_buffer(buffer).unwrap();
}

#[test]
fn wgpu_rhi_device_roundtrips_resource_descriptors_by_handle() {
    let device = WgpuRenderDevice::new_headless();
    let buffer_desc = BufferDesc::new(
        "frame-uniform",
        256,
        BufferUsage::UNIFORM | BufferUsage::COPY_DST,
    );
    let texture_desc = TextureDesc::new(
        "scene-color",
        64,
        64,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
    );
    let sampler_desc = SamplerDesc::linear("scene-linear");
    let shader_desc = ShaderModuleDesc {
        label: Some("fullscreen".to_string()),
        source: "@compute @workgroup_size(1) fn main() {}".to_string(),
        stage: ShaderStage::Compute,
        entry_point: "main".to_string(),
    };
    let pipeline_desc = PipelineDesc::new("compute", PipelineKind::Compute);

    let buffer = device.create_buffer(&buffer_desc).unwrap();
    let texture = device.create_texture(&texture_desc).unwrap();
    let sampler = device.create_sampler(&sampler_desc).unwrap();
    let shader = device.create_shader_module(&shader_desc).unwrap();
    let pipeline = device.create_pipeline(&pipeline_desc).unwrap();

    assert_eq!(device.buffer_desc(buffer).unwrap(), buffer_desc);
    assert_eq!(device.texture_desc(texture).unwrap(), texture_desc);
    assert_eq!(device.sampler_desc(sampler).unwrap(), sampler_desc);
    assert_eq!(device.shader_module_desc(shader).unwrap(), shader_desc);
    assert_eq!(device.pipeline_desc(pipeline).unwrap(), pipeline_desc);

    device.destroy_buffer(buffer).unwrap();
    assert_eq!(
        device.buffer_desc(buffer).unwrap_err(),
        crate::rhi::RhiError::UnknownBuffer(buffer.raw())
    );
}

#[test]
fn wgpu_rhi_rejects_invalid_resource_descriptors() {
    let device = WgpuRenderDevice::new_headless();

    assert_eq!(
        device
            .create_buffer(&BufferDesc::new("empty-buffer", 0, BufferUsage::COPY_SRC))
            .unwrap_err(),
        crate::rhi::RhiError::InvalidBufferDescriptor {
            label: Some("empty-buffer".to_string()),
            reason: "size_bytes must be greater than zero".to_string(),
        }
    );
    assert_eq!(
        device
            .create_buffer(&BufferDesc::new("no-buffer-usage", 16, BufferUsage::NONE))
            .unwrap_err(),
        crate::rhi::RhiError::InvalidBufferDescriptor {
            label: Some("no-buffer-usage".to_string()),
            reason: "usage must not be empty".to_string(),
        }
    );

    let zero_extent = TextureDesc::new(
        "zero-extent",
        0,
        2,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::COPY_SRC,
    );
    assert_eq!(
        device.create_texture(&zero_extent).unwrap_err(),
        crate::rhi::RhiError::InvalidTextureDescriptor {
            label: Some("zero-extent".to_string()),
            reason: "width, height, and depth must be greater than zero".to_string(),
        }
    );

    let mut no_mips = TextureDesc::new(
        "no-mips",
        2,
        2,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::COPY_SRC,
    );
    no_mips.mip_levels = 0;
    assert_eq!(
        device.create_texture(&no_mips).unwrap_err(),
        crate::rhi::RhiError::InvalidTextureDescriptor {
            label: Some("no-mips".to_string()),
            reason: "mip_levels must be greater than zero".to_string(),
        }
    );

    let no_usage = TextureDesc::new(
        "no-texture-usage",
        2,
        2,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::NONE,
    );
    assert_eq!(
        device.create_texture(&no_usage).unwrap_err(),
        crate::rhi::RhiError::InvalidTextureDescriptor {
            label: Some("no-texture-usage".to_string()),
            reason: "usage must not be empty".to_string(),
        }
    );

    let mut overflowing_storage = TextureDesc::new(
        "overflowing-texture",
        u32::MAX,
        u32::MAX,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::COPY_SRC,
    );
    overflowing_storage.depth = u32::MAX;
    assert_eq!(
        device.create_texture(&overflowing_storage).unwrap_err(),
        crate::rhi::RhiError::InvalidTextureDescriptor {
            label: Some("overflowing-texture".to_string()),
            reason: "storage size overflows u64".to_string(),
        }
    );
}

#[test]
fn wgpu_rhi_fence_queries_reject_unissued_fence_values() {
    let device = WgpuRenderDevice::new_headless();

    assert_eq!(
        device.is_fence_complete(FenceValue(0)).unwrap_err(),
        crate::rhi::RhiError::UnknownFence(0)
    );
    assert_eq!(
        device.is_fence_complete(FenceValue(7)).unwrap_err(),
        crate::rhi::RhiError::UnknownFence(7)
    );

    let command_list = device
        .create_command_list(RenderQueueClass::Copy, "empty-copy")
        .unwrap();
    let fence = device.submit(command_list).unwrap();
    assert!(device.is_fence_complete(fence).unwrap());
    assert_eq!(
        device
            .is_fence_complete(FenceValue(fence.0 + 1))
            .unwrap_err(),
        crate::rhi::RhiError::UnknownFence(fence.0 + 1)
    );
}

#[test]
fn command_list_keeps_queue_class_and_label() {
    let command_list = WgpuCommandList::new(RenderQueueClass::Graphics, "main");
    assert_eq!(command_list.queue_class(), RenderQueueClass::Graphics);
    assert_eq!(command_list.label(), Some("main"));
}

#[test]
fn command_list_records_buffer_copy_commands_and_submit_validates_resources() {
    let device = WgpuRenderDevice::new_headless();
    let source = device
        .create_buffer(&BufferDesc::new("copy-source", 32, BufferUsage::COPY_SRC))
        .unwrap();
    let destination = device
        .create_buffer(&BufferDesc::new(
            "copy-destination",
            16,
            BufferUsage::COPY_DST,
        ))
        .unwrap();

    let mut command_list = device
        .create_command_list(RenderQueueClass::Copy, "copy-valid")
        .unwrap();
    command_list.push_debug_marker("upload source");
    command_list.copy_buffer_to_buffer(source, destination, 4, 8, 8);

    assert_eq!(command_list.recorded_command_count(), 2);
    let fence = device.submit(command_list).unwrap();
    assert!(device.is_fence_complete(fence).unwrap());

    let mut unknown_destination = device
        .create_command_list(RenderQueueClass::Copy, "copy-unknown-destination")
        .unwrap();
    unknown_destination.copy_buffer_to_buffer(source, BufferHandle::new(9_999), 0, 0, 4);

    assert_eq!(
        device.submit(unknown_destination).unwrap_err(),
        crate::rhi::RhiError::UnknownBuffer(9_999)
    );

    let mut out_of_range = device
        .create_command_list(RenderQueueClass::Copy, "copy-out-of-range")
        .unwrap();
    out_of_range.copy_buffer_to_buffer(source, destination, 0, 12, 8);

    assert_eq!(
        device.submit(out_of_range).unwrap_err(),
        crate::rhi::RhiError::BufferCopyOutOfRange {
            source_buffer: source.raw(),
            destination_buffer: destination.raw(),
            source_offset: 0,
            destination_offset: 12,
            size: 8,
        }
    );
}

#[test]
fn command_list_buffer_copy_submit_validates_usage_flags() {
    let device = WgpuRenderDevice::new_headless();
    let invalid_source = device
        .create_buffer(&BufferDesc::new(
            "not-copy-source",
            16,
            BufferUsage::UNIFORM,
        ))
        .unwrap();
    let valid_destination = device
        .create_buffer(&BufferDesc::new(
            "copy-destination",
            16,
            BufferUsage::COPY_DST,
        ))
        .unwrap();

    let mut source_command_list = device
        .create_command_list(RenderQueueClass::Copy, "invalid-source-copy")
        .unwrap();
    source_command_list.copy_buffer_to_buffer(invalid_source, valid_destination, 0, 0, 4);

    assert_eq!(
        device.submit(source_command_list).unwrap_err(),
        crate::rhi::RhiError::InvalidBufferUsage {
            buffer: invalid_source.raw(),
            required: BufferUsage::COPY_SRC,
            actual: BufferUsage::UNIFORM,
        }
    );

    let valid_source = device
        .create_buffer(&BufferDesc::new("copy-source", 16, BufferUsage::COPY_SRC))
        .unwrap();
    let invalid_destination = device
        .create_buffer(&BufferDesc::new(
            "not-copy-destination",
            16,
            BufferUsage::STORAGE,
        ))
        .unwrap();
    let mut destination_command_list = device
        .create_command_list(RenderQueueClass::Copy, "invalid-destination-copy")
        .unwrap();
    destination_command_list.copy_buffer_to_buffer(valid_source, invalid_destination, 0, 0, 4);

    assert_eq!(
        device.submit(destination_command_list).unwrap_err(),
        crate::rhi::RhiError::InvalidBufferUsage {
            buffer: invalid_destination.raw(),
            required: BufferUsage::COPY_DST,
            actual: BufferUsage::STORAGE,
        }
    );
}

#[test]
fn wgpu_rhi_write_copy_and_read_buffer_preserves_bytes() {
    let device = WgpuRenderDevice::new_headless();
    let upload = device
        .create_buffer(&BufferDesc::new(
            "upload",
            16,
            BufferUsage::STAGING_WRITE | BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let gpu_buffer = device
        .create_buffer(&BufferDesc::new(
            "gpu-buffer",
            16,
            BufferUsage::COPY_DST | BufferUsage::STAGING_READ,
        ))
        .unwrap();

    device
        .write_buffer(upload, 4, &[10, 20, 30, 40, 50, 60])
        .unwrap();

    let mut command_list = device
        .create_command_list(RenderQueueClass::Copy, "upload-copy")
        .unwrap();
    command_list.copy_buffer_to_buffer(upload, gpu_buffer, 4, 2, 6);
    let fence = device.submit(command_list).unwrap();
    assert!(device.is_fence_complete(fence).unwrap());

    assert_eq!(
        device.read_buffer(gpu_buffer, 0, 10).unwrap(),
        vec![0, 0, 10, 20, 30, 40, 50, 60, 0, 0]
    );
}

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
    command_list.copy_buffer_to_texture(upload, texture, 0, 8, 2, 2);
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
    command_list.copy_buffer_to_texture(upload, texture, 0, 8, 2, 2);
    command_list.copy_texture_to_buffer(texture, readback, 4, 8, 2, 2);
    let fence = device.submit(command_list).unwrap();
    assert!(device.is_fence_complete(fence).unwrap());

    assert_eq!(
        device.read_buffer(readback, 0, 24).unwrap(),
        vec![0, 0, 0, 0, 1, 2, 3, 4, 10, 20, 30, 40, 5, 6, 7, 8, 50, 60, 70, 80, 0, 0, 0, 0]
    );
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
    invalid_source_commands.copy_buffer_to_texture(invalid_source, valid_texture, 0, 8, 2, 2);

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
    invalid_texture_commands.copy_buffer_to_texture(valid_source, invalid_texture, 0, 8, 2, 2);

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
    out_of_range_commands.copy_buffer_to_texture(small_source, valid_texture, 0, 8, 2, 2);

    assert_eq!(
        device.submit(out_of_range_commands).unwrap_err(),
        crate::rhi::RhiError::BufferToTextureCopyOutOfRange {
            source_buffer: small_source.raw(),
            destination_texture: valid_texture.raw(),
            source_offset: 0,
            bytes_per_row: 8,
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
    invalid_source_commands.copy_texture_to_buffer(invalid_source, valid_destination, 0, 8, 2, 2);

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
        2,
        2,
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
    out_of_range_commands.copy_texture_to_buffer(valid_source, small_destination, 0, 8, 2, 2);

    assert_eq!(
        device.submit(out_of_range_commands).unwrap_err(),
        crate::rhi::RhiError::TextureToBufferCopyOutOfRange {
            source_texture: valid_source.raw(),
            destination_buffer: small_destination.raw(),
            destination_offset: 0,
            bytes_per_row: 8,
            width: 2,
            height: 2,
        }
    );
}

#[test]
fn wgpu_rhi_write_buffer_validates_usage_and_range() {
    let device = WgpuRenderDevice::new_headless();
    let read_only = device
        .create_buffer(&BufferDesc::new("read-only", 8, BufferUsage::STAGING_READ))
        .unwrap();

    assert_eq!(
        device.write_buffer(read_only, 0, &[1, 2, 3]).unwrap_err(),
        crate::rhi::RhiError::InvalidBufferUsage {
            buffer: read_only.raw(),
            required: BufferUsage::STAGING_WRITE,
            actual: BufferUsage::STAGING_READ,
        }
    );

    let upload = device
        .create_buffer(&BufferDesc::new("upload", 8, BufferUsage::STAGING_WRITE))
        .unwrap();
    assert_eq!(
        device.write_buffer(upload, 6, &[1, 2, 3]).unwrap_err(),
        crate::rhi::RhiError::WriteOutOfRange {
            buffer: upload.raw(),
            offset: 6,
            size: 3,
        }
    );
}

#[test]
fn wgpu_rhi_read_texture_validates_usage() {
    let device = WgpuRenderDevice::new_headless();
    let write_only = device
        .create_texture(&TextureDesc::new(
            "write-only-texture",
            2,
            2,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::COPY_DST,
        ))
        .unwrap();

    assert_eq!(
        device.read_texture(write_only).unwrap_err(),
        crate::rhi::RhiError::InvalidTextureUsage {
            texture: write_only.raw(),
            required: TextureUsage::COPY_SRC,
            actual: TextureUsage::COPY_DST,
        }
    );
}

#[test]
fn wgpu_rhi_read_buffer_validates_usage_and_range() {
    let device = WgpuRenderDevice::new_headless();
    let non_readback = device
        .create_buffer(&BufferDesc::new("non-readback", 8, BufferUsage::COPY_DST))
        .unwrap();

    assert_eq!(
        device.read_buffer(non_readback, 0, 4).unwrap_err(),
        crate::rhi::RhiError::InvalidBufferUsage {
            buffer: non_readback.raw(),
            required: BufferUsage::STAGING_READ,
            actual: BufferUsage::COPY_DST,
        }
    );

    let readback = device
        .create_buffer(&BufferDesc::new("readback", 8, BufferUsage::STAGING_READ))
        .unwrap();
    assert_eq!(
        device.read_buffer(readback, 6, 3).unwrap_err(),
        crate::rhi::RhiError::ReadbackOutOfRange {
            buffer: readback.raw(),
            offset: 6,
            size: 3,
        }
    );
}

#[test]
fn app_editor_and_core_framework_sources_do_not_import_wgpu() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = runtime_root
        .parent()
        .expect("zircon_runtime should live under the workspace root");
    let boundary_roots = [
        runtime_root.join("src").join("core").join("framework"),
        workspace_root.join("zircon_app").join("src"),
        workspace_root.join("zircon_editor").join("src"),
    ];
    let mut offenders = Vec::new();
    for root in boundary_roots {
        collect_wgpu_imports(&root, &mut offenders);
    }

    assert!(
        offenders.is_empty(),
        "app/editor/framework sources must stay behind RenderFramework/RHI boundaries: {offenders:?}"
    );
}

fn collect_wgpu_imports(path: &Path, offenders: &mut Vec<String>) {
    let entries = std::fs::read_dir(path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    });
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_wgpu_imports(&path, offenders);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        for (line_index, line) in source.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            let imports_wgpu = trimmed.starts_with("use wgpu")
                || trimmed.starts_with("use ::wgpu")
                || (trimmed.contains("wgpu::") && !trimmed.contains('"'));
            if imports_wgpu {
                offenders.push(format!("{}:{}", path.display(), line_index + 1));
            }
        }
    }
}
