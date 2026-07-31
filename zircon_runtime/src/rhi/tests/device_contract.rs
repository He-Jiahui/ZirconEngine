use crate::rhi::{
    BindGroupDesc, BindGroupEntryDesc, BindGroupEntryResource, BindGroupHandle,
    BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindGroupLayoutHandle, BindingResourceType,
    BufferDesc, BufferHandle, BufferUsage, CommandList, CompareFunction, FenceValue, PipelineDesc,
    PipelineHandle, PipelineKind, PipelineLayoutDesc, PipelineLayoutHandle, RenderDevice,
    RenderQueueClass, SamplerDesc, SamplerHandle, ShaderModuleDesc, ShaderModuleHandle,
    ShaderStage, TextureDesc, TextureDimension, TextureFormat, TextureHandle, TextureUsage,
};
use crate::rhi_wgpu::DeterministicRhiContractDevice;

fn test_bind_group_layout_desc(label: &str) -> BindGroupLayoutDesc {
    BindGroupLayoutDesc::new(
        label,
        vec![BindGroupLayoutEntryDesc::new(
            0,
            BindingResourceType::UniformBuffer,
            vec![
                ShaderStage::Vertex,
                ShaderStage::Fragment,
                ShaderStage::Compute,
            ],
        )],
    )
}

fn create_test_pipeline_layout(
    device: &DeterministicRhiContractDevice,
    label: &str,
) -> PipelineLayoutHandle {
    let bind_group_layout = device
        .create_bind_group_layout(&test_bind_group_layout_desc(&format!("{label}-bind-group")))
        .unwrap();
    device
        .create_pipeline_layout(&PipelineLayoutDesc::new(label, vec![bind_group_layout]))
        .unwrap()
}

mod basic_resources;
mod bind_groups;
mod framework_boundary;
mod invalid_descriptors;
mod texture_sampler_descriptors;
mod transfer_and_fences;
