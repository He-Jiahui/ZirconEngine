use crate::DeterministicRhiContractDevice;
use zr_rhi::{
    BindGroupDesc, BindGroupEntryDesc, BindGroupEntryResource, BindGroupLayoutDesc,
    BindGroupLayoutEntryDesc, BindingResourceType, BufferDesc, BufferUsage, PipelineDesc,
    PipelineKind, PipelineLayoutDesc, RenderDevice, RenderQueueClass, RhiError, ShaderModuleDesc,
    ShaderStage, TextureDesc, TextureFormat, TextureUsage, TransientAllocatorStats,
};

const RESOURCE_CHURN_ITERATIONS: usize = 100_000;

#[test]
fn deterministic_rhi_contract_resource_churn_returns_registry_and_bytes_to_baseline() {
    let device = DeterministicRhiContractDevice::new_headless();

    for iteration in 0..RESOURCE_CHURN_ITERATIONS {
        if iteration % 2 == 0 {
            let buffer = device
                .create_buffer(&BufferDesc::new("buffer-churn", 4, BufferUsage::COPY_DST))
                .unwrap();
            device.destroy_buffer(buffer).unwrap();
        } else {
            let texture = device
                .create_texture(&TextureDesc::new(
                    "texture-churn",
                    1,
                    1,
                    TextureFormat::Rgba8Unorm,
                    TextureUsage::COPY_DST,
                ))
                .unwrap();
            device.destroy_texture(texture).unwrap();
        }
    }

    assert_eq!(
        device.transient_allocator_stats(),
        TransientAllocatorStats::default()
    );
    assert_eq!(
        device.memory_snapshot(),
        zr_rhi::GpuMemorySnapshot::default()
    );
}

#[test]
fn deterministic_rhi_contract_reports_live_transient_allocator_stats_for_buffers_and_textures() {
    let device = DeterministicRhiContractDevice::new_headless();

    assert_eq!(
        device.transient_allocator_stats(),
        TransientAllocatorStats::default()
    );

    let uniform = device
        .create_buffer(&BufferDesc::new("uniform", 64, BufferUsage::UNIFORM))
        .unwrap();
    let staging = device
        .create_buffer(&BufferDesc::new(
            "staging",
            16,
            BufferUsage::STAGING_READ | BufferUsage::COPY_DST,
        ))
        .unwrap();
    let color = device
        .create_texture(&TextureDesc::new(
            "color",
            4,
            4,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
        ))
        .unwrap();

    assert_eq!(
        device.transient_allocator_stats(),
        TransientAllocatorStats {
            bytes_reserved: 64 + 16 + 4 * 4 * 4,
            allocations: 3,
        }
    );

    device.destroy_buffer(staging).unwrap();
    assert_eq!(
        device.transient_allocator_stats(),
        TransientAllocatorStats {
            bytes_reserved: 64 + 4 * 4 * 4,
            allocations: 2,
        }
    );

    device.destroy_texture(color).unwrap();
    device.destroy_buffer(uniform).unwrap();
    assert_eq!(
        device.transient_allocator_stats(),
        TransientAllocatorStats::default()
    );
}

#[test]
fn deterministic_rhi_contract_destroying_bound_resources_updates_stats_without_releasing_descriptors(
) {
    let device = DeterministicRhiContractDevice::new_headless();
    let uniform = device
        .create_buffer(&BufferDesc::new("uniform", 64, BufferUsage::UNIFORM))
        .unwrap();
    let layout = device
        .create_bind_group_layout(&BindGroupLayoutDesc::new(
            "layout",
            vec![BindGroupLayoutEntryDesc::new(
                0,
                BindingResourceType::UniformBuffer,
                vec![ShaderStage::Vertex],
            )],
        ))
        .unwrap();
    let bind_group = device
        .create_bind_group(&BindGroupDesc::new(
            "bind-group",
            layout,
            vec![BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::Buffer(zr_rhi::BindGroupBufferBinding::whole(uniform)),
            )],
        ))
        .unwrap();

    assert_eq!(
        device.transient_allocator_stats(),
        TransientAllocatorStats {
            bytes_reserved: 64,
            allocations: 1,
        }
    );

    device.destroy_buffer(uniform).unwrap();
    assert_eq!(
        device.transient_allocator_stats(),
        TransientAllocatorStats::default()
    );
    assert!(device.bind_group_desc(bind_group).is_ok());

    device.destroy_bind_group(bind_group).unwrap();
    device.destroy_bind_group_layout(layout).unwrap();
}

#[test]
fn deterministic_rhi_contract_submit_rejects_bind_group_with_destroyed_resource() {
    let device = DeterministicRhiContractDevice::new_headless();
    let shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "stale-bind-group-compute",
            ShaderStage::Compute,
            "main",
            "@compute @workgroup_size(1) fn main() {}",
        ))
        .unwrap();
    let layout = device
        .create_bind_group_layout(&BindGroupLayoutDesc::new(
            "stale-layout",
            vec![BindGroupLayoutEntryDesc::new(
                0,
                BindingResourceType::UniformBuffer,
                vec![ShaderStage::Compute],
            )],
        ))
        .unwrap();
    let pipeline_layout = device
        .create_pipeline_layout(&PipelineLayoutDesc::new(
            "stale-pipeline-layout",
            vec![layout],
        ))
        .unwrap();
    let pipeline = device
        .create_pipeline(
            &PipelineDesc::new("stale-compute", PipelineKind::Compute)
                .with_layout(pipeline_layout)
                .with_compute_shader(shader),
        )
        .unwrap();
    let uniform = device
        .create_buffer(&BufferDesc::new("stale-uniform", 64, BufferUsage::UNIFORM))
        .unwrap();
    let bind_group = device
        .create_bind_group(&BindGroupDesc::new(
            "stale-bind-group",
            layout,
            vec![BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::Buffer(zr_rhi::BindGroupBufferBinding::whole(uniform)),
            )],
        ))
        .unwrap();

    device.destroy_buffer(uniform).unwrap();

    let mut command_list = device
        .create_command_list(RenderQueueClass::Compute, "stale-bind-group-dispatch")
        .unwrap();
    command_list.set_pipeline(pipeline);
    command_list.set_bind_group(0, bind_group);
    command_list.dispatch_compute(1, 1, 1);

    assert_eq!(
        device.submit(command_list).unwrap_err(),
        RhiError::UnknownBuffer(uniform.diagnostic_id())
    );
}
