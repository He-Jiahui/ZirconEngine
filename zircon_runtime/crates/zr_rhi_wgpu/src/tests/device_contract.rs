use crate::DeterministicRhiContractDevice;
use zr_rhi::{
    BindGroupDesc, BindGroupEntryDesc, BindGroupEntryResource, BindGroupHandle,
    BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindGroupLayoutHandle, BindingResourceType,
    BufferDesc, BufferHandle, BufferUsage, CommandList, CompareFunction, PipelineDesc,
    PipelineHandle, PipelineKind, PipelineLayoutDesc, PipelineLayoutHandle, RenderDevice,
    RenderQueueClass, RhiError, SamplerBindingType, SamplerDesc, SamplerHandle, ShaderModuleDesc,
    ShaderModuleHandle, ShaderStage, SubmissionStatus, TextureCopyRegion, TextureDesc,
    TextureDimension, TextureFormat, TextureHandle, TextureSampleType, TextureUsage,
    TextureViewAspect, TextureViewDesc, TextureViewDimension, TextureViewHandle,
};

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

#[path = "device_contract/basic_resources.rs"]
mod basic_resources;
#[path = "device_contract/bind_groups.rs"]
mod bind_groups;
#[path = "../../tests/device_contract/framework_boundary.rs"]
mod framework_boundary;
#[path = "../../tests/device_contract/invalid_descriptors.rs"]
mod invalid_descriptors;
#[path = "../../tests/device_contract/texture_sampler_descs.rs"]
mod texture_sampler_descriptors;
#[path = "device_contract/texture_views.rs"]
mod texture_views;
#[path = "../../tests/transfer_and_submissions.rs"]
mod transfer_and_submissions;
