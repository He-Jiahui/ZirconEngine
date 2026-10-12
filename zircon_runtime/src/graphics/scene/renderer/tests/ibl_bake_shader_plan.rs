use crate::core::framework::render::ProceduralSkyParams;
use crate::graphics::shader::invocation::{
    ShaderDispatchExtent, ShaderParameterValue, COMPUTE_SHADER_FIRST_RESOURCE_BINDING,
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
fn ibl_bake_pipeline_content_hashes_are_derived_from_the_wgsl_bytes() {
    assert_eq!(
        IBL_BAKE_PMREM_SHADER_CONTENT_HASH,
        shader_source_content_hash(IBL_BAKE_PMREM_WGSL)
    );
    assert_eq!(
        IBL_BAKE_IRRADIANCE_SH9_SHADER_CONTENT_HASH,
        shader_source_content_hash(IBL_BAKE_IRRADIANCE_SH9_WGSL)
    );
    assert_eq!(
        IBL_BAKE_IRRADIANCE_CUBE_SHADER_CONTENT_HASH,
        shader_source_content_hash(IBL_BAKE_IRRADIANCE_CUBE_WGSL)
    );
    assert_ne!(
        IBL_BAKE_PMREM_SHADER_CONTENT_HASH,
        IBL_BAKE_IRRADIANCE_SH9_SHADER_CONTENT_HASH
    );
    assert_ne!(
        IBL_BAKE_IRRADIANCE_SH9_SHADER_CONTENT_HASH,
        IBL_BAKE_IRRADIANCE_CUBE_SHADER_CONTENT_HASH
    );
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
        ShaderDispatchExtent::Fixed([1, 1, 1])
    );
    assert!(
        IBL_BAKE_PMREM_WGSL.contains("write_terminal_average_to_all_faces: f32"),
        "the terminal PMREM command requires the reserved uniform word that writes its six-face average"
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
        mip0.dispatch
            .parameters
            .get("write_terminal_average_to_all_faces"),
        Some(&ShaderParameterValue::F32 { value: 0.0 })
    );
    assert_eq!(
        mip7.dispatch
            .parameters
            .get("write_terminal_average_to_all_faces"),
        Some(&ShaderParameterValue::F32 { value: 1.0 })
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
fn capture_pmrem_quality_scales_the_canonical_sample_budget() {
    let request = IblBakeArtifactRequest::new(
        ProceduralSkyParams::default_gradient().ibl_bake_key(),
        128,
        8,
    );
    let sample_count = |quality| {
        ibl_bake_pmrem_kernel_plan_with_quality(&request, 7, quality)
            .dispatch
            .parameters
            .get("sample_count")
            .cloned()
    };

    assert_eq!(
        sample_count(SourceCubemapPrefilterQuality::Fast),
        Some(ShaderParameterValue::U32 { value: 64 })
    );
    assert_eq!(
        sample_count(SourceCubemapPrefilterQuality::Normal),
        Some(ShaderParameterValue::U32 { value: 128 })
    );
    assert_eq!(
        sample_count(SourceCubemapPrefilterQuality::High),
        Some(ShaderParameterValue::U32 { value: 256 })
    );
}

#[test]
fn ibl_bake_shader_plans_keep_source_and_fixed_pmrem_layouts_independent() {
    let request = IblBakeArtifactRequest::new(
        ProceduralSkyParams::default_gradient().ibl_bake_key(),
        512,
        10,
    )
    .with_required_contents(IblBakeArtifactContents::PMREM_SH9_IEM);

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
    let irradiance_cube = ibl_bake_irradiance_cube_kernel_plan(&request);
    assert_eq!(
        irradiance_cube.dispatch.parameters.get("source_mip_level"),
        Some(&ShaderParameterValue::U32 { value: 4 }),
        "GPU IEM must consume the framework's canonical diffuse source mip"
    );
    assert_eq!(pmrem_roughness_for_mip(8, 6), 1.0);
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
        IBL_BAKE_PMREM_WGSL
            .contains("let denominator = (1.0 - no_h_squared) + no_h_squared * alpha2"),
        "PMREM D_GGX must evaluate the positive denominator without cancellation"
    );
    assert!(
        !IBL_BAKE_PMREM_WGSL.contains("max(PI * denominator * denominator, 0.000001)"),
        "a full-denominator floor must not flatten valid low-roughness PMREM peaks"
    );
    assert!(
        IBL_BAKE_PMREM_WGSL.contains("textureDimensions(source_cubemap)")
            && IBL_BAKE_PMREM_WGSL.contains("textureNumLevels(source_cubemap)"),
        "source LOD must use the actual source texture layout, not the fixed PMREM layout"
    );
    assert!(
        IBL_BAKE_PMREM_WGSL.matches("source_footprint_lod(").count() == 2,
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
fn ibl_bake_pmrem_hoists_source_layout_queries_per_invocation() {
    assert_eq!(
        IBL_BAKE_PMREM_WGSL
            .matches("textureDimensions(source_cubemap)")
            .count(),
        1,
        "PMREM should query source dimensions once per invocation, not per importance sample"
    );
    assert_eq!(
        IBL_BAKE_PMREM_WGSL
            .matches("textureNumLevels(source_cubemap)")
            .count(),
        1,
        "PMREM should query the source mip count once per invocation, not per importance sample"
    );
    assert!(
        IBL_BAKE_PMREM_WGSL
            .contains("let source_face_size = f32(max(textureDimensions(source_cubemap).x, 1u));")
            && IBL_BAKE_PMREM_WGSL.contains(
                "let source_max_mip = f32(max(textureNumLevels(source_cubemap), 1u) - 1u);"
            ),
        "PMREM must prepare the actual source layout before sampling"
    );
    assert!(
        IBL_BAKE_PMREM_WGSL.contains(
            "fn source_lod_for_pdf(\n    pdf: f32,\n    sample_count: u32,\n    source_face_size: f32,\n    source_max_mip: f32,"
        ) && IBL_BAKE_PMREM_WGSL
            .contains("source_lod_for_pdf(pdf, sample_count, source_face_size, source_max_mip)"),
        "PDF LOD selection must consume the invocation-prepared source layout"
    );
}

#[test]
fn ibl_bake_pmrem_wgsl_matches_shared_cubemap_face_orientation_contract() {
    assert!(
        IBL_BAKE_PMREM_WGSL.contains("fn cube_face_direction"),
        "GPU PMREM should retain the shared cubemap face-direction helper"
    );

    for direction in [
        "return normalize(vec3<f32>(1.0, -uv.y, -uv.x));",
        "return normalize(vec3<f32>(-1.0, -uv.y, uv.x));",
        "return normalize(vec3<f32>(uv.x, 1.0, uv.y));",
        "return normalize(vec3<f32>(uv.x, -1.0, -uv.y));",
        "return normalize(vec3<f32>(uv.x, -uv.y, 1.0));",
        "return normalize(vec3<f32>(-uv.x, -uv.y, -1.0));",
    ] {
        assert!(
            IBL_BAKE_PMREM_WGSL.contains(direction),
            "GPU PMREM face direction must match the CPU cubemap projection owner: {direction}"
        );
    }
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
        ShaderDispatchExtent::Fixed([1, 1, 1])
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
    assert!(IBL_BAKE_IRRADIANCE_CUBE_WGSL.contains("texture_storage_2d_array<rgba16float, write>"));
}

#[test]
fn ibl_bake_sh9_uses_one_sixty_four_thread_parallel_reduction_group() {
    assert!(IBL_BAKE_IRRADIANCE_SH9_WGSL.contains("@workgroup_size(8, 8, 1)"));
    assert!(IBL_BAKE_IRRADIANCE_SH9_WGSL.contains("var<workgroup> sh0_shared"));
    assert!(IBL_BAKE_IRRADIANCE_SH9_WGSL.contains("workgroupBarrier()"));
    assert!(IBL_BAKE_IRRADIANCE_SH9_WGSL.contains("local_invocation_index"));
    assert!(
        !IBL_BAKE_IRRADIANCE_SH9_WGSL.contains("global_id != vec3<u32>(0u, 0u, 0u)"),
        "SH9 projection must not serialize all cubemap samples onto one invocation"
    );
}
