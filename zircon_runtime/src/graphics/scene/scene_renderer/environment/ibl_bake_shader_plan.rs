use crate::core::framework::render::{
    source_cubemap_roughness_from_pmrem_mip, IblBakeArtifactContents, IblBakeArtifactRequest,
    SourceCubemapPrefilterQuality, CANONICAL_IBL_BAKE_RECIPE,
    SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
};
use crate::graphics::shader::invocation::{
    ComputeDispatchBuilder, ComputeDispatchPlan, ComputeKernelRef,
    RenderShaderEntryPointDescriptor, RenderShaderStage, ShaderAssetKind, ShaderResourceAccess,
    ShaderResourceDescriptor, ShaderResourceKind,
};

use super::ibl_bake_graph_plan::{
    ibl_bake_pmrem_dispatch_groups, ibl_bake_terminal_pmrem_average_mip,
    IBL_BAKE_IRRADIANCE_CUBE_PIPELINE_LABEL, IBL_BAKE_IRRADIANCE_CUBE_RESOURCE,
    IBL_BAKE_IRRADIANCE_SH9_DISPATCH_GROUPS, IBL_BAKE_IRRADIANCE_SH9_PIPELINE_LABEL,
    IBL_BAKE_IRRADIANCE_SH9_RESOURCE, IBL_BAKE_PMREM_PIPELINE_LABEL, IBL_BAKE_PMREM_RESOURCE,
    IBL_BAKE_SOURCE_CUBEMAP_RESOURCE,
};

pub(in crate::graphics::scene::scene_renderer) const IBL_BAKE_COMPUTE_ENTRY_POINT: &str = "cs_main";
pub(in crate::graphics::scene::scene_renderer) const IBL_BAKE_SOURCE_SAMPLER_RESOURCE: &str =
    "environment.ibl.source_sampler";
pub(in crate::graphics::scene::scene_renderer) const IBL_BAKE_PMREM_SHADER: &str =
    "builtin://shaders/environment/ibl_prefilter";
pub(in crate::graphics::scene::scene_renderer) const IBL_BAKE_IRRADIANCE_SH9_SHADER: &str =
    "builtin://shaders/environment/ibl_irradiance_sh";
pub(in crate::graphics::scene::scene_renderer) const IBL_BAKE_IRRADIANCE_CUBE_SHADER: &str =
    "builtin://shaders/environment/ibl_irradiance_cube";

pub(in crate::graphics::scene::scene_renderer) const IBL_BAKE_PMREM_WGSL: &str =
    include_str!("shaders/ibl_prefilter.wgsl");
pub(in crate::graphics::scene::scene_renderer) const IBL_BAKE_IRRADIANCE_SH9_WGSL: &str =
    include_str!("shaders/ibl_irradiance_sh.wgsl");
pub(in crate::graphics::scene::scene_renderer) const IBL_BAKE_IRRADIANCE_CUBE_WGSL: &str =
    include_str!("shaders/ibl_irradiance_cube.wgsl");

const IBL_BAKE_WORKGROUP_SIZE: [u32; 3] = [8, 8, 1];
const IBL_BAKE_CUBE_FACE_COUNT: u32 = 6;
const IBL_BAKE_PMREM_SHADER_CONTENT_HASH: u64 = shader_source_content_hash(IBL_BAKE_PMREM_WGSL);
const IBL_BAKE_IRRADIANCE_SH9_SHADER_CONTENT_HASH: u64 =
    shader_source_content_hash(IBL_BAKE_IRRADIANCE_SH9_WGSL);
const IBL_BAKE_IRRADIANCE_CUBE_SHADER_CONTENT_HASH: u64 =
    shader_source_content_hash(IBL_BAKE_IRRADIANCE_CUBE_WGSL);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer) enum IblBakeComputeKernelKind {
    Pmrem { mip_level: u32 },
    IrradianceSh9,
    IrradianceCube,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::graphics::scene::scene_renderer) struct IblBakeComputeKernelPlan {
    pub kind: IblBakeComputeKernelKind,
    pub shader_locator: &'static str,
    pub wgsl_source: &'static str,
    pub dispatch: ComputeDispatchPlan,
}

pub(in crate::graphics::scene::scene_renderer) fn ibl_bake_compute_kernel_plans_for_request(
    request: &IblBakeArtifactRequest,
) -> Vec<IblBakeComputeKernelPlan> {
    let mut plans = Vec::new();
    let contents = request.required_contents();
    if contents.contains(IblBakeArtifactContents::PMREM) {
        plans.extend(
            (0..request.pmrem_mip_count())
                .map(|mip_level| ibl_bake_pmrem_kernel_plan(request, mip_level)),
        );
    }
    if contents.contains(IblBakeArtifactContents::SH9) {
        plans.push(ibl_bake_irradiance_sh9_kernel_plan(request));
    }
    if contents.contains(IblBakeArtifactContents::IEM) {
        plans.push(ibl_bake_irradiance_cube_kernel_plan(request));
    }
    plans
}

pub(in crate::graphics::scene::scene_renderer) fn ibl_bake_pmrem_kernel_plan(
    request: &IblBakeArtifactRequest,
    mip_level: u32,
) -> IblBakeComputeKernelPlan {
    ibl_bake_pmrem_kernel_plan_with_quality(
        request,
        mip_level,
        SourceCubemapPrefilterQuality::Normal,
    )
}

pub(in crate::graphics::scene::scene_renderer) fn ibl_bake_pmrem_kernel_plan_with_quality(
    request: &IblBakeArtifactRequest,
    mip_level: u32,
    quality: SourceCubemapPrefilterQuality,
) -> IblBakeComputeKernelPlan {
    let roughness = pmrem_roughness_for_mip(request.pmrem_mip_count(), mip_level);
    let sample_count = pmrem_sample_count(roughness, mip_level, quality);
    let mip_size = pmrem_mip_size(request.pmrem_face_size(), mip_level);
    let write_terminal_average_to_all_faces = ibl_bake_terminal_pmrem_average_mip(
        request.pmrem_face_size(),
        request.pmrem_mip_count(),
        mip_level,
    );
    let builder = ComputeDispatchBuilder::new(kernel_ref(IBL_BAKE_PMREM_SHADER))
        .with_pipeline_label(IBL_BAKE_PMREM_PIPELINE_LABEL)
        .with_workgroup_size(IBL_BAKE_WORKGROUP_SIZE)
        .with_content_hash(IBL_BAKE_PMREM_SHADER_CONTENT_HASH)
        .set_u32("face_size", request.pmrem_face_size())
        .set_u32("mip_face_size", mip_size)
        .set_u32("mip_level", mip_level)
        .set_u32("mip_count", request.pmrem_mip_count())
        .set_u32("sample_count", sample_count)
        .set_f32("roughness", roughness)
        .set_f32(
            "write_terminal_average_to_all_faces",
            if write_terminal_average_to_all_faces {
                1.0
            } else {
                0.0
            },
        )
        .bind_texture(IBL_BAKE_SOURCE_CUBEMAP_RESOURCE)
        .bind_sampler(IBL_BAKE_SOURCE_SAMPLER_RESOURCE)
        .bind_storage_texture_write(IBL_BAKE_PMREM_RESOURCE)
        .dispatch_groups(ibl_bake_pmrem_dispatch_groups(
            request.pmrem_face_size(),
            request.pmrem_mip_count(),
            mip_level,
        ));

    IblBakeComputeKernelPlan {
        kind: IblBakeComputeKernelKind::Pmrem { mip_level },
        shader_locator: IBL_BAKE_PMREM_SHADER,
        wgsl_source: IBL_BAKE_PMREM_WGSL,
        dispatch: builder
            .build(
                ShaderAssetKind::Compute,
                &compute_entry_points(),
                &pmrem_resources(),
            )
            .expect("IBL PMREM compute dispatch contract must be valid"),
    }
}

pub(in crate::graphics::scene::scene_renderer) fn ibl_bake_irradiance_sh9_kernel_plan(
    request: &IblBakeArtifactRequest,
) -> IblBakeComputeKernelPlan {
    let builder = ComputeDispatchBuilder::new(kernel_ref(IBL_BAKE_IRRADIANCE_SH9_SHADER))
        .with_pipeline_label(IBL_BAKE_IRRADIANCE_SH9_PIPELINE_LABEL)
        .with_workgroup_size(IBL_BAKE_WORKGROUP_SIZE)
        .with_content_hash(IBL_BAKE_IRRADIANCE_SH9_SHADER_CONTENT_HASH)
        .set_u32("source_face_size", request.source_face_size())
        .set_u32("sample_face_size", SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE)
        .set_f32(
            "source_lod",
            canonical_diffuse_source_mip_level(request) as f32,
        )
        .bind_texture(IBL_BAKE_SOURCE_CUBEMAP_RESOURCE)
        .bind_sampler(IBL_BAKE_SOURCE_SAMPLER_RESOURCE)
        .bind_storage_write(IBL_BAKE_IRRADIANCE_SH9_RESOURCE)
        .dispatch_groups(IBL_BAKE_IRRADIANCE_SH9_DISPATCH_GROUPS);

    IblBakeComputeKernelPlan {
        kind: IblBakeComputeKernelKind::IrradianceSh9,
        shader_locator: IBL_BAKE_IRRADIANCE_SH9_SHADER,
        wgsl_source: IBL_BAKE_IRRADIANCE_SH9_WGSL,
        dispatch: builder
            .build(
                ShaderAssetKind::Compute,
                &compute_entry_points(),
                &irradiance_sh9_resources(),
            )
            .expect("IBL SH9 compute dispatch contract must be valid"),
    }
}

pub(in crate::graphics::scene::scene_renderer) fn ibl_bake_irradiance_cube_kernel_plan(
    request: &IblBakeArtifactRequest,
) -> IblBakeComputeKernelPlan {
    let builder = ComputeDispatchBuilder::new(kernel_ref(IBL_BAKE_IRRADIANCE_CUBE_SHADER))
        .with_pipeline_label(IBL_BAKE_IRRADIANCE_CUBE_PIPELINE_LABEL)
        .with_workgroup_size(IBL_BAKE_WORKGROUP_SIZE)
        .with_content_hash(IBL_BAKE_IRRADIANCE_CUBE_SHADER_CONTENT_HASH)
        .set_u32("source_face_size", request.source_face_size())
        .set_u32(
            "irradiance_face_size",
            SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
        )
        .set_u32(
            "sample_count",
            CANONICAL_IBL_BAKE_RECIPE.runtime_diffuse_sample_count(),
        )
        .set_u32(
            "source_mip_level",
            canonical_diffuse_source_mip_level(request),
        )
        .bind_texture(IBL_BAKE_SOURCE_CUBEMAP_RESOURCE)
        .bind_sampler(IBL_BAKE_SOURCE_SAMPLER_RESOURCE)
        .bind_storage_texture_write(IBL_BAKE_IRRADIANCE_CUBE_RESOURCE)
        .dispatch_groups(irradiance_dispatch_groups());

    IblBakeComputeKernelPlan {
        kind: IblBakeComputeKernelKind::IrradianceCube,
        shader_locator: IBL_BAKE_IRRADIANCE_CUBE_SHADER,
        wgsl_source: IBL_BAKE_IRRADIANCE_CUBE_WGSL,
        dispatch: builder
            .build(
                ShaderAssetKind::Compute,
                &compute_entry_points(),
                &irradiance_cube_resources(),
            )
            .expect("IBL irradiance cube compute dispatch contract must be valid"),
    }
}

fn kernel_ref(shader_locator: &str) -> ComputeKernelRef {
    ComputeKernelRef::from_locator_str(shader_locator, IBL_BAKE_COMPUTE_ENTRY_POINT)
        .expect("builtin IBL compute shader locator must be valid")
}

fn compute_entry_points() -> [RenderShaderEntryPointDescriptor; 1] {
    [RenderShaderEntryPointDescriptor {
        name: IBL_BAKE_COMPUTE_ENTRY_POINT.to_string(),
        stage: RenderShaderStage::Compute,
    }]
}

fn pmrem_resources() -> [ShaderResourceDescriptor; 3] {
    [
        texture_resource(IBL_BAKE_SOURCE_CUBEMAP_RESOURCE),
        sampler_resource(IBL_BAKE_SOURCE_SAMPLER_RESOURCE),
        storage_texture_write_resource(IBL_BAKE_PMREM_RESOURCE),
    ]
}

fn irradiance_sh9_resources() -> [ShaderResourceDescriptor; 3] {
    [
        texture_resource(IBL_BAKE_SOURCE_CUBEMAP_RESOURCE),
        sampler_resource(IBL_BAKE_SOURCE_SAMPLER_RESOURCE),
        storage_write_resource(IBL_BAKE_IRRADIANCE_SH9_RESOURCE),
    ]
}

fn irradiance_cube_resources() -> [ShaderResourceDescriptor; 3] {
    [
        texture_resource(IBL_BAKE_SOURCE_CUBEMAP_RESOURCE),
        sampler_resource(IBL_BAKE_SOURCE_SAMPLER_RESOURCE),
        storage_texture_write_resource(IBL_BAKE_IRRADIANCE_CUBE_RESOURCE),
    ]
}

fn texture_resource(name: &str) -> ShaderResourceDescriptor {
    ShaderResourceDescriptor {
        name: name.to_string(),
        kind: ShaderResourceKind::Texture,
        access: Some(ShaderResourceAccess::Read),
    }
}

fn sampler_resource(name: &str) -> ShaderResourceDescriptor {
    ShaderResourceDescriptor {
        name: name.to_string(),
        kind: ShaderResourceKind::Sampler,
        access: Some(ShaderResourceAccess::Read),
    }
}

fn storage_write_resource(name: &str) -> ShaderResourceDescriptor {
    ShaderResourceDescriptor {
        name: name.to_string(),
        kind: ShaderResourceKind::StorageBuffer,
        access: Some(ShaderResourceAccess::Write),
    }
}

fn storage_texture_write_resource(name: &str) -> ShaderResourceDescriptor {
    ShaderResourceDescriptor {
        name: name.to_string(),
        kind: ShaderResourceKind::StorageTexture,
        access: Some(ShaderResourceAccess::Write),
    }
}

fn irradiance_dispatch_groups() -> [u32; 3] {
    [
        div_ceil(
            SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
            IBL_BAKE_WORKGROUP_SIZE[0],
        ),
        div_ceil(
            SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
            IBL_BAKE_WORKGROUP_SIZE[1],
        ),
        IBL_BAKE_CUBE_FACE_COUNT,
    ]
}

fn pmrem_sample_count(
    roughness: f32,
    mip_level: u32,
    quality: SourceCubemapPrefilterQuality,
) -> u32 {
    let normal = CANONICAL_IBL_BAKE_RECIPE.pmrem_sample_count(roughness, mip_level);
    match quality {
        SourceCubemapPrefilterQuality::Fast => (normal / 2).max(16),
        SourceCubemapPrefilterQuality::Normal => normal,
        SourceCubemapPrefilterQuality::High => normal.saturating_mul(2),
    }
}

fn pmrem_roughness_for_mip(mip_count: u32, mip_level: u32) -> f32 {
    source_cubemap_roughness_from_pmrem_mip(mip_level, mip_count)
}

fn canonical_diffuse_source_mip_level(request: &IblBakeArtifactRequest) -> u32 {
    CANONICAL_IBL_BAKE_RECIPE
        .diffuse_source_mip_level(request.source_face_size(), request.source_mip_count())
}

const fn pmrem_mip_size(face_size: u32, mip_level: u32) -> u32 {
    let shifted = face_size >> mip_level;
    if shifted == 0 {
        1
    } else {
        shifted
    }
}

const fn div_ceil(value: u32, divisor: u32) -> u32 {
    value.saturating_add(divisor.saturating_sub(1)) / divisor
}

const fn shader_source_content_hash(source: &str) -> u64 {
    let bytes = source.as_bytes();
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    let mut index = 0;
    while index < bytes.len() {
        hash ^= bytes[index] as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        index += 1;
    }
    hash
}

#[cfg(test)]
#[path = "tests/ibl_bake_shader_plan.rs"]
mod tests;
