use super::*;

#[test]
fn rhi_handles_are_stable_raw_identifiers() {
    assert_eq!(BufferHandle::new(11).raw(), 11);
    assert_eq!(TextureHandle::new(12).raw(), 12);
    assert_eq!(SamplerHandle::new(13).raw(), 13);
    assert_eq!(BindGroupLayoutHandle::new(14).raw(), 14);
    assert_eq!(BindGroupHandle::new(15).raw(), 15);
    assert_eq!(ShaderModuleHandle::new(16).raw(), 16);
    assert_eq!(PipelineLayoutHandle::new(17).raw(), 17);
    assert_eq!(PipelineHandle::new(18).raw(), 18);
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
fn deterministic_rhi_contract_device_allocates_stable_resource_handles_and_fences() {
    let device = DeterministicRhiContractDevice::new_headless();

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
        .create_shader_module(&ShaderModuleDesc::new(
            "fullscreen",
            ShaderStage::Compute,
            "main",
            "@compute @workgroup_size(1) fn main() {}",
        ))
        .unwrap();
    let pipeline_layout = create_test_pipeline_layout(&device, "compute-layout");
    let pipeline = device
        .create_pipeline(
            &PipelineDesc::new("compute", PipelineKind::Compute)
                .with_layout(pipeline_layout)
                .with_compute_shader(shader),
        )
        .unwrap();

    assert_ne!(buffer.raw(), texture.raw());
    assert_ne!(sampler.raw(), shader.raw());
    assert_ne!(pipeline.raw(), 0);

    let command_list = device
        .create_command_list(RenderQueueClass::Copy, "copy-upload")
        .unwrap();
    assert_eq!(command_list.queue_class(), RenderQueueClass::Copy);
    assert_eq!(command_list.label(), Some("copy-upload"));
    let compute_command_list = device
        .create_command_list(RenderQueueClass::Compute, "compute-main")
        .unwrap();
    assert_eq!(
        compute_command_list.queue_class(),
        RenderQueueClass::Compute
    );

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
