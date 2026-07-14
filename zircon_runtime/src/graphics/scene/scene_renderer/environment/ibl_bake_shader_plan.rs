use crate::core::framework::render::{
    ComputeDispatchBuilder, ComputeDispatchPlan, ComputeKernelRef, IblBakeArtifactContents,
    IblBakeArtifactRequest, RenderShaderEntryPointDescriptor, RenderShaderStage,
    SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE, ShaderAssetKind, ShaderResourceAccess,
    ShaderResourceDescriptor, ShaderResourceKind,
};

use super::ibl_bake_graph_plan::{
    IBL_BAKE_IRRADIANCE_CUBE_PIPELINE_LABEL, IBL_BAKE_IRRADIANCE_CUBE_RESOURCE,
    IBL_BAKE_IRRADIANCE_SH9_PIPELINE_LABEL, IBL_BAKE_IRRADIANCE_SH9_RESOURCE,
    IBL_BAKE_PMREM_PIPELINE_LABEL, IBL_BAKE_PMREM_RESOURCE, IBL_BAKE_SOURCE_CUBEMAP_RESOURCE,
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
const IBL_BAKE_SHADER_CONTENT_HASH_V2: u64 = 0x1b1_ba6e_0006;
const IBL_BAKE_PMREM_NORMAL_SAMPLE_COUNT: u32 = 64;
const IBL_BAKE_PMREM_FAST_SAMPLE_COUNT: u32 = 32;
const IBL_BAKE_PMREM_ROUGH_SAMPLE_COUNT: u32 = 128;
const IBL_BAKE_IRRADIANCE_SAMPLE_COUNT: u32 = 64;
const IBL_BAKE_ROUGHEST_MIP: f32 = 1.0;
const IBL_BAKE_ROUGHNESS_MIP_SCALE: f32 = 1.2;
const IBL_BAKE_PMREM_LOW_ROUGHNESS_THRESHOLD: f32 = 0.1;
const IBL_BAKE_PMREM_HIGH_ROUGHNESS_THRESHOLD: f32 = 0.75;

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
    let roughness = pmrem_roughness_for_mip(request.pmrem_mip_count(), mip_level);
    let sample_count = pmrem_sample_count(roughness, mip_level);
    let mip_size = pmrem_mip_size(request.pmrem_face_size(), mip_level);
    let builder = ComputeDispatchBuilder::new(kernel_ref(IBL_BAKE_PMREM_SHADER))
        .with_pipeline_label(IBL_BAKE_PMREM_PIPELINE_LABEL)
        .with_workgroup_size(IBL_BAKE_WORKGROUP_SIZE)
        .with_content_hash(IBL_BAKE_SHADER_CONTENT_HASH_V2)
        .set_u32("face_size", request.pmrem_face_size())
        .set_u32("mip_face_size", mip_size)
        .set_u32("mip_level", mip_level)
        .set_u32("mip_count", request.pmrem_mip_count())
        .set_u32("sample_count", sample_count)
        .set_f32("roughness", roughness)
        .bind_texture(IBL_BAKE_SOURCE_CUBEMAP_RESOURCE)
        .bind_sampler(IBL_BAKE_SOURCE_SAMPLER_RESOURCE)
        .bind_storage_texture_write(IBL_BAKE_PMREM_RESOURCE)
        .dispatch_groups(pmrem_dispatch_groups(request.pmrem_face_size(), mip_level));

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
        .with_content_hash(IBL_BAKE_SHADER_CONTENT_HASH_V2)
        .set_u32("source_face_size", request.source_face_size())
        .set_u32("sample_face_size", SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE)
        .set_f32(
            "source_lod",
            source_lod_for_sample_face_size(
                request.source_face_size(),
                SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
            ),
        )
        .bind_texture(IBL_BAKE_SOURCE_CUBEMAP_RESOURCE)
        .bind_sampler(IBL_BAKE_SOURCE_SAMPLER_RESOURCE)
        .bind_storage_write(IBL_BAKE_IRRADIANCE_SH9_RESOURCE)
        .dispatch_groups(irradiance_dispatch_groups());

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
        .with_content_hash(IBL_BAKE_SHADER_CONTENT_HASH_V2)
        .set_u32("source_face_size", request.source_face_size())
        .set_u32(
            "irradiance_face_size",
            SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
        )
        .set_u32("sample_count", IBL_BAKE_IRRADIANCE_SAMPLE_COUNT)
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

fn pmrem_dispatch_groups(face_size: u32, mip_level: u32) -> [u32; 3] {
    let mip_size = pmrem_mip_size(face_size, mip_level);
    [
        div_ceil(mip_size, IBL_BAKE_WORKGROUP_SIZE[0]),
        div_ceil(mip_size, IBL_BAKE_WORKGROUP_SIZE[1]),
        IBL_BAKE_CUBE_FACE_COUNT,
    ]
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

fn pmrem_sample_count(roughness: f32, mip_level: u32) -> u32 {
    if mip_level == 0 || roughness < IBL_BAKE_PMREM_LOW_ROUGHNESS_THRESHOLD {
        IBL_BAKE_PMREM_FAST_SAMPLE_COUNT
    } else if roughness >= IBL_BAKE_PMREM_HIGH_ROUGHNESS_THRESHOLD {
        IBL_BAKE_PMREM_ROUGH_SAMPLE_COUNT
    } else {
        IBL_BAKE_PMREM_NORMAL_SAMPLE_COUNT
    }
}

fn pmrem_roughness_for_mip(mip_count: u32, mip_level: u32) -> f32 {
    if mip_level == 0 || mip_count <= 1 {
        return 0.0;
    }
    let max_mip = mip_count.saturating_sub(1);
    let clamped_mip = mip_level.min(max_mip);
    let level_from_1x1 = max_mip.saturating_sub(1).saturating_sub(clamped_mip) as f32;
    2.0_f32
        .powf((IBL_BAKE_ROUGHEST_MIP - level_from_1x1) / IBL_BAKE_ROUGHNESS_MIP_SCALE)
        .clamp(0.0, 1.0)
}

fn source_lod_for_sample_face_size(source_face_size: u32, sample_face_size: u32) -> f32 {
    if source_face_size <= sample_face_size || sample_face_size == 0 {
        0.0
    } else {
        ((source_face_size as f32) / (sample_face_size as f32)).log2()
    }
}

const fn pmrem_mip_size(face_size: u32, mip_level: u32) -> u32 {
    let shifted = face_size >> mip_level;
    if shifted == 0 { 1 } else { shifted }
}

const fn div_ceil(value: u32, divisor: u32) -> u32 {
    value.saturating_add(divisor.saturating_sub(1)) / divisor
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        COMPUTE_SHADER_FIRST_RESOURCE_BINDING, ProceduralSkyParams, ShaderDispatchExtent,
        ShaderParameterValue,
    };

    use super::*;

    #[test]
    fn ibl_bake_compute_shader_sources_parse_as_wgsl() {
        for (label, source) in [
            (IBL_BAKE_PMREM_SHADER, IBL_BAKE_PMREM_WGSL),
            (IBL_BAKE_IRRADIANCE_SH9_SHADER, IBL_BAKE_IRRADIANCE_SH9_WGSL),
            (
                IBL_BAKE_IRRADIANCE_CUBE_SHADER,
                IBL_BAKE_IRRADIANCE_CUBE_WGSL,
            ),
        ] {
            naga::front::wgsl::parse_str(source)
                .unwrap_or_else(|error| panic!("{label} WGSL should parse: {error}"));
        }
    }

    #[test]
    fn ibl_bake_compute_kernel_plans_follow_graph_content_order() {
        let request = IblBakeArtifactRequest::new(
            ProceduralSkyParams::default_gradient().ibl_bake_key(),
            128,
            8,
        )
        .with_required_contents(IblBakeArtifactContents::PMREM_SH9_IEM);

        let plans = ibl_bake_compute_kernel_plans_for_request(&request);

        assert_eq!(plans.len(), 10);
        assert_eq!(
            plans[0].kind,
            IblBakeComputeKernelKind::Pmrem { mip_level: 0 }
        );
        assert_eq!(
            plans[7].kind,
            IblBakeComputeKernelKind::Pmrem { mip_level: 7 }
        );
        assert_eq!(plans[8].kind, IblBakeComputeKernelKind::IrradianceSh9);
        assert_eq!(plans[9].kind, IblBakeComputeKernelKind::IrradianceCube);
    }

    #[test]
    fn ibl_bake_pmrem_kernel_plans_are_mip_scoped_wgpu_storage_views() {
        let request = IblBakeArtifactRequest::new(
            ProceduralSkyParams::default_gradient().ibl_bake_key(),
            128,
            8,
        )
        .with_required_contents(IblBakeArtifactContents::PMREM);

        let mip0 = ibl_bake_pmrem_kernel_plan(&request, 0);
        let mip7 = ibl_bake_pmrem_kernel_plan(&request, 7);

        assert_eq!(mip0.dispatch.pipeline_label, IBL_BAKE_PMREM_PIPELINE_LABEL);
        assert_eq!(mip0.dispatch.workgroup_size, [8, 8, 1]);
        assert_eq!(
            mip0.dispatch.dispatch_extent,
            ShaderDispatchExtent::Fixed([16, 16, 6])
        );
        assert_eq!(
            mip7.dispatch.dispatch_extent,
            ShaderDispatchExtent::Fixed([1, 1, 6])
        );
        assert_eq!(
            mip7.dispatch.parameters.get("mip_level"),
            Some(&ShaderParameterValue::U32 { value: 7 })
        );
        assert_eq!(
            mip7.dispatch.parameters.get("sample_count"),
            Some(&ShaderParameterValue::U32 { value: 128 })
        );
        assert_eq!(
            mip0.dispatch.resources[0].name,
            IBL_BAKE_SOURCE_CUBEMAP_RESOURCE
        );
        assert_eq!(
            mip0.dispatch.resources[0].abi.binding,
            COMPUTE_SHADER_FIRST_RESOURCE_BINDING
        );
        assert_eq!(
            mip0.dispatch.resources[1].name,
            IBL_BAKE_SOURCE_SAMPLER_RESOURCE
        );
        assert_eq!(mip0.dispatch.resources[1].abi.binding, 2);
        assert_eq!(mip0.dispatch.resources[2].name, IBL_BAKE_PMREM_RESOURCE);
        assert_eq!(
            mip0.dispatch.resources[2].kind,
            ShaderResourceKind::StorageTexture
        );
        assert_eq!(
            mip0.dispatch.resources[2].access,
            ShaderResourceAccess::Write
        );
        assert!(
            IBL_BAKE_PMREM_WGSL.contains("texture_storage_2d_array<rgba16float, write>"),
            "WGPU writes cube mip slices through a D2Array storage view"
        );
        assert!(
            !IBL_BAKE_PMREM_WGSL.contains("texture_storage_cube"),
            "WGPU has no texture_storage_cube binding"
        );
    }

    #[test]
    fn ibl_bake_shader_plans_keep_source_and_fixed_pmrem_layouts_independent() {
        let request = IblBakeArtifactRequest::new(
            ProceduralSkyParams::default_gradient().ibl_bake_key(),
            512,
            10,
        )
        .with_required_contents(IblBakeArtifactContents::PMREM_SH9);

        let pmrem = ibl_bake_pmrem_kernel_plan(&request, 0);
        assert_eq!(
            pmrem.dispatch.parameters.get("face_size"),
            Some(&ShaderParameterValue::U32 { value: 128 })
        );
        assert_eq!(
            pmrem.dispatch.parameters.get("mip_count"),
            Some(&ShaderParameterValue::U32 { value: 8 })
        );
        assert_eq!(
            pmrem.dispatch.dispatch_extent,
            ShaderDispatchExtent::Fixed([16, 16, 6])
        );

        let irradiance = ibl_bake_irradiance_sh9_kernel_plan(&request);
        assert_eq!(
            irradiance.dispatch.parameters.get("source_face_size"),
            Some(&ShaderParameterValue::U32 { value: 512 })
        );
        assert_eq!(
            irradiance.dispatch.parameters.get("source_lod"),
            Some(&ShaderParameterValue::F32 { value: 4.0 })
        );
        assert_eq!(pmrem_roughness_for_mip(8, 5), 1.0);
    }

    #[test]
    fn ibl_bake_pmrem_wgsl_matches_plan06_filtered_importance_contract() {
        assert!(
            IBL_BAKE_PMREM_WGSL.contains("FULL_ROUGHNESS_COSINE_THRESHOLD"),
            "PMREM WGSL should keep the roughness>=0.99 cosine convolution branch"
        );
        assert!(
            IBL_BAKE_PMREM_WGSL.contains("cosine_sample_hemisphere"),
            "roughness>=0.99 should sample a cosine hemisphere instead of downsampling PMREM mips"
        );
        assert!(
            IBL_BAKE_PMREM_WGSL.contains("FIS_SOLID_ANGLE_TEXEL_SCALE")
                && IBL_BAKE_PMREM_WGSL.contains("* FIS_SOLID_ANGLE_TEXEL_SCALE"),
            "filtered importance sampling must use the UE texel solid-angle scale"
        );
        assert!(
            IBL_BAKE_PMREM_WGSL.contains("xi.y * 0.995"),
            "GGX importance sampling should preserve the Unreal grazing-angle guard"
        );
        assert!(
            IBL_BAKE_PMREM_WGSL.contains("f32(index) + 0.5"),
            "Hammersley samples should be centered to match the CPU PMREM bridge"
        );
        assert!(
            IBL_BAKE_PMREM_WGSL.contains("source_lod_for_pdf"),
            "GGX and cosine paths should share the PDF-driven source mip selection"
        );
        assert!(
            IBL_BAKE_PMREM_WGSL.contains("distribution_ggx(no_h, roughness) * 0.25"),
            "V=N reduces the GGX light-direction PDF to D/4, matching Unreal"
        );
        assert!(
            !IBL_BAKE_PMREM_WGSL.contains("distribution_ggx(no_h, roughness) * no_h * 0.25"),
            "the canceled NoH/VoH factor must not be multiplied into the UE PDF"
        );
        assert!(
            IBL_BAKE_PMREM_WGSL.contains("textureDimensions(source_cubemap)")
                && IBL_BAKE_PMREM_WGSL.contains("textureNumLevels(source_cubemap)"),
            "source LOD must use the actual source texture layout, not the fixed PMREM layout"
        );
        assert!(
            IBL_BAKE_PMREM_WGSL
                .matches("source_footprint_lod()")
                .count()
                == 2,
            "destination-footprint source LOD should be defined and used only by mip0 downsampling"
        );
        assert!(
            !IBL_BAKE_PMREM_WGSL.contains("max(lod, source_footprint_lod())"),
            "filtered GGX/cosine FIS must not apply the mip0 downsampling footprint as a LOD floor"
        );
        assert!(
            IBL_BAKE_PMREM_WGSL.contains("final_pmrem_face_average")
                && IBL_BAKE_PMREM_WGSL.contains("params.mip_level + 1u >= params.mip_count"),
            "the final 1x1 PMREM mip should write the same six-face average to every face"
        );
    }

    #[test]
    fn ibl_bake_irradiance_kernel_plans_use_source_cube_sampler_and_outputs() {
        let request = IblBakeArtifactRequest::new(
            ProceduralSkyParams::default_gradient().ibl_bake_key(),
            128,
            8,
        )
        .with_required_contents(IblBakeArtifactContents::SH9 | IblBakeArtifactContents::IEM);

        let plans = ibl_bake_compute_kernel_plans_for_request(&request);

        assert_eq!(plans.len(), 2);
        assert_eq!(
            plans[0].dispatch.dispatch_extent,
            ShaderDispatchExtent::Fixed([4, 4, 6])
        );
        assert_eq!(
            plans[0].dispatch.resources[2].name,
            IBL_BAKE_IRRADIANCE_SH9_RESOURCE
        );
        assert_eq!(
            plans[0].dispatch.resources[2].kind,
            ShaderResourceKind::StorageBuffer
        );
        assert_eq!(
            plans[1].dispatch.dispatch_extent,
            ShaderDispatchExtent::Fixed([4, 4, 6])
        );
        assert_eq!(
            plans[1].dispatch.resources[2].name,
            IBL_BAKE_IRRADIANCE_CUBE_RESOURCE
        );
        assert_eq!(
            plans[1].dispatch.resources[2].kind,
            ShaderResourceKind::StorageTexture
        );
        assert!(
            IBL_BAKE_IRRADIANCE_CUBE_WGSL.contains("texture_storage_2d_array<rgba16float, write>")
        );
    }
}
