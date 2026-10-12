use crate::core::framework::render::{
    IblBakeArtifactContents, IblBakeArtifactRequest, ProceduralSkyParams,
    IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES,
};
use crate::graphics::backend::RenderBackend;

use super::super::ibl_bake_shader_plan::IblBakeComputeKernelKind;
use super::super::ibl_bake_wgpu_command_plan::{
    ibl_bake_wgpu_command_plan_for_request, IblBakeWgpuOutputPlan,
};
use super::*;

#[test]
fn bind_groups_create_for_storage_texture_and_storage_buffer_outputs() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let device = &backend.device;
    let layouts = IblBakeWgpuBindGroupLayouts::new(device);
    let sampler = create_ibl_bake_wgpu_source_sampler(device);
    let source_texture = create_source_cubemap_texture(device);
    let source_view = source_texture.create_view(&source_cubemap_view_descriptor());

    let request = request(64, 7, IblBakeArtifactContents::PMREM_SH9_IEM);
    let plan = ibl_bake_wgpu_command_plan_for_request(&request);

    let pmrem_command = command_for_kind(
        &plan.commands,
        IblBakeComputeKernelKind::Pmrem { mip_level: 2 },
    );
    let params = create_ibl_bake_wgpu_params_buffer(device, pmrem_command);
    let output_texture = create_storage_output_texture(device, 64, 7);
    let output_view = output_texture.create_view(&storage_texture_descriptor(pmrem_command));
    let texture_bind_group = create_ibl_bake_wgpu_bind_group(
        device,
        &layouts,
        pmrem_command,
        &params,
        &source_view,
        &sampler,
        IblBakeWgpuOutputBindingResource::StorageTexture2DArray(&output_view),
    );
    assert!(texture_bind_group.is_ok());

    let sh9_command = command_for_kind(&plan.commands, IblBakeComputeKernelKind::IrradianceSh9);
    let params = create_ibl_bake_wgpu_params_buffer(device, sh9_command);
    let sh9_output = create_sh9_output_buffer(device);
    let buffer_bind_group = create_ibl_bake_wgpu_bind_group(
        device,
        &layouts,
        sh9_command,
        &params,
        &source_view,
        &sampler,
        IblBakeWgpuOutputBindingResource::StorageBuffer(&sh9_output),
    );
    assert!(buffer_bind_group.is_ok());
}

#[test]
fn bind_group_creation_rejects_output_kind_mismatches_before_wgpu_validation() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let device = &backend.device;
    let layouts = IblBakeWgpuBindGroupLayouts::new(device);
    let sampler = create_ibl_bake_wgpu_source_sampler(device);
    let source_texture = create_source_cubemap_texture(device);
    let source_view = source_texture.create_view(&source_cubemap_view_descriptor());
    let sh9_output = create_sh9_output_buffer(device);

    let request = request(64, 7, IblBakeArtifactContents::PMREM_SH9);
    let plan = ibl_bake_wgpu_command_plan_for_request(&request);
    let pmrem_command = command_for_kind(
        &plan.commands,
        IblBakeComputeKernelKind::Pmrem { mip_level: 0 },
    );
    let params = create_ibl_bake_wgpu_params_buffer(device, pmrem_command);

    let result = create_ibl_bake_wgpu_bind_group(
        device,
        &layouts,
        pmrem_command,
        &params,
        &source_view,
        &sampler,
        IblBakeWgpuOutputBindingResource::StorageBuffer(&sh9_output),
    );

    assert!(result.is_err());
    assert!(result
        .err()
        .unwrap()
        .contains("expects StorageTexture2DArray output binding"));
}

fn request(
    face_size: u32,
    mip_count: u32,
    contents: IblBakeArtifactContents,
) -> IblBakeArtifactRequest {
    IblBakeArtifactRequest::new(
        ProceduralSkyParams::default_gradient().ibl_bake_key(),
        face_size,
        mip_count,
    )
    .with_required_contents(contents)
}

fn command_for_kind(
    commands: &[IblBakeWgpuCommandPlan],
    kind: IblBakeComputeKernelKind,
) -> &IblBakeWgpuCommandPlan {
    commands
        .iter()
        .find(|command| command.kind == kind)
        .expect("requested command should be present")
}

fn create_source_cubemap_texture(device: &wgpu::Device) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ibl-bake-test-source-cubemap"),
        size: wgpu::Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 6,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

fn source_cubemap_view_descriptor() -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        label: Some("ibl-bake-test-source-cubemap-view"),
        format: Some(wgpu::TextureFormat::Rgba8Unorm),
        dimension: Some(wgpu::TextureViewDimension::Cube),
        usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: Some(1),
        base_array_layer: 0,
        array_layer_count: Some(6),
    }
}

fn create_storage_output_texture(
    device: &wgpu::Device,
    face_size: u32,
    mip_count: u32,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ibl-bake-test-storage-output"),
        size: wgpu::Extent3d {
            width: face_size,
            height: face_size,
            depth_or_array_layers: 6,
        },
        mip_level_count: mip_count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn storage_texture_descriptor(
    command: &IblBakeWgpuCommandPlan,
) -> wgpu::TextureViewDescriptor<'static> {
    let IblBakeWgpuOutputPlan::StorageTexture { view, .. } = &command.output else {
        panic!("command should write a storage texture")
    };
    (*view).to_wgpu_descriptor()
}

fn create_sh9_output_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ibl-bake-test-sh9-output"),
        size: IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    })
}
