use super::*;

#[test]
fn rhi_handles_are_opaque_device_owned_identifiers() {
    let device = DeterministicRhiContractDevice::new_headless();
    let buffer = device
        .create_buffer(&BufferDesc::new("handle-buffer", 4, BufferUsage::COPY_DST))
        .unwrap();
    let texture = device
        .create_texture(&TextureDesc::new(
            "handle-texture",
            1,
            1,
            TextureFormat::Rgba8Unorm,
            TextureUsage::COPY_DST,
        ))
        .unwrap();

    assert_eq!(buffer.device_id(), texture.device_id());
    assert_eq!(buffer.device_generation(), texture.device_generation());
    assert_ne!(buffer.diagnostic_id(), texture.diagnostic_id());
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
fn deterministic_rhi_contract_device_allocates_stable_resource_handles_and_submissions() {
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

    assert_ne!(buffer.diagnostic_id(), texture.diagnostic_id());
    assert_ne!(sampler.diagnostic_id(), shader.diagnostic_id());
    assert_ne!(pipeline.diagnostic_id(), 0);

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

    let ticket = device.submit(command_list).unwrap();
    assert_eq!(ticket.device_id(), buffer.device_id());
    assert_eq!(ticket.generation(), buffer.device_generation());
    assert_eq!(ticket.queue_class(), RenderQueueClass::Copy);
    assert_eq!(ticket.sequence(), 1);
    assert_eq!(
        device.submission_status(ticket).unwrap(),
        SubmissionStatus::Completed
    );

    let bytes = device.read_buffer(buffer, 0, 16).unwrap();
    assert_eq!(bytes.len(), 16);

    device.destroy_pipeline(pipeline).unwrap();
    device.destroy_shader_module(shader).unwrap();
    device.destroy_sampler(sampler).unwrap();
    device.destroy_texture(texture).unwrap();
    device.destroy_buffer(buffer).unwrap();
}
