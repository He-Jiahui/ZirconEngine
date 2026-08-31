use super::super::super::render_feature_pass_descriptor::{
    RenderFeatureResourceAccess, RenderFeatureResourceKind, RenderFeatureResourceWriteMode,
};
use super::*;
use crate::render_graph::RenderGraphExternalResourceBinding;

#[test]
fn ssao_uses_the_generic_compute_executor_with_dynamic_graph_resources() {
    let descriptor = descriptor();
    let pass = descriptor
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == "ssao-evaluate")
        .expect("SSAO evaluate pass");
    let compute = pass
        .compute_pass
        .as_ref()
        .expect("generic compute descriptor");

    assert_eq!(pass.executor_id.as_str(), "compute.generic");
    assert_eq!(compute.workgroup_size, SSAO_WORKGROUP_SIZE);
    let RenderGraphComputeDispatchExtent::PerPixel { target, local_size } = &compute.dispatch
    else {
        panic!("SSAO must use a per-pixel compute dispatch");
    };
    assert_eq!(target, PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW);
    assert_eq!(
        local_size,
        &[SSAO_WORKGROUP_SIZE[0], SSAO_WORKGROUP_SIZE[1]]
    );
    assert_eq!(compute.bindings.len(), 5);
    assert!(compute.bindings.iter().any(|binding| {
        binding.binding == 4
            && binding.resource == PostProcessGraphResourceNames::HZB_FURTHEST
            && binding.texture_full_mip_chain
    }));
    assert!(descriptor.stage_passes.iter().all(|stage_pass| {
        stage_pass.resources.iter().all(|resource| {
            resource.name != PostProcessGraphResourceNames::HISTORY_PREVIOUS_AMBIENT_OCCLUSION
        })
    }));
    assert!(pass.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::SSAO_PARAMS
            && resource.kind == RenderFeatureResourceKind::External
            && resource.access == RenderFeatureResourceAccess::Read
            && resource.external_binding == RenderGraphExternalResourceBinding::report_only_buffer()
            && resource.schema == Some(ssao_params_schema())
            && resource.access_metadata
                == Some(crate::render_graph::RenderGraphResourceAccessMetadata::new(
                    crate::render_graph::RenderGraphResourceAccessRange::Buffer(
                        RenderGraphBufferRange::full(),
                    ),
                    RenderGraphResourceAccessIntent::UniformBuffer {
                        stages: RenderGraphShaderStages::COMPUTE,
                    },
                ))
    }));
    assert!(pass.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW
            && resource.kind == RenderFeatureResourceKind::Texture
            && resource.access == RenderFeatureResourceAccess::Write
            && resource.write_mode == RenderFeatureResourceWriteMode::Storage
            && resource.schema == Some(ambient_occlusion_schema())
            && !resource.usage.persistent
    }));

    let spatial = descriptor
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == "ssao-spatial-denoise")
        .expect("SSAO spatial denoise pass");
    let spatial_compute = spatial
        .compute_pass
        .as_ref()
        .expect("generic spatial compute descriptor");
    let RenderGraphComputeDispatchExtent::PerPixel { target, local_size } =
        &spatial_compute.dispatch
    else {
        panic!("SSAO spatial denoise must use a per-pixel compute dispatch");
    };
    assert_eq!(target, PostProcessGraphResourceNames::AMBIENT_OCCLUSION);
    assert_eq!(
        local_size,
        &[SSAO_WORKGROUP_SIZE[0], SSAO_WORKGROUP_SIZE[1]]
    );
    assert_eq!(spatial_compute.bindings.len(), 5);
    assert!(spatial_compute.bindings.iter().any(|binding| {
        binding.binding == 0
            && binding.resource == PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW
    }));
    assert!(spatial.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW
            && resource.access == RenderFeatureResourceAccess::Read
            && resource
                .input_version
                .as_ref()
                .is_some_and(|version| version.producer_pass_name() == "ssao-evaluate")
    }));
    assert!(spatial.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::AMBIENT_OCCLUSION
            && resource.kind == RenderFeatureResourceKind::External
            && resource.access == RenderFeatureResourceAccess::Write
            && resource.write_mode == RenderFeatureResourceWriteMode::Storage
            && resource.external_binding
                == RenderGraphExternalResourceBinding::report_only_texture()
            && resource.schema == Some(ambient_occlusion_schema())
            && resource.usage.persistent
    }));
}

#[test]
fn half_resolution_ssao_uses_two_half_extent_stages_and_a_full_extent_upsample() {
    let mut descriptor = descriptor();
    configure_descriptor_for_resolution(&mut descriptor, 2)
        .expect("the supported half-resolution topology should compile");

    assert_eq!(descriptor.stage_passes.len(), 3);
    let half_extent = RenderTextureExtentPolicy::Relative {
        reference: RenderTextureExtentReference::Render,
        numerator: 1,
        denominator: 2,
        rounding: RenderTextureExtentRounding::Ceil,
    };
    let evaluate = descriptor
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == "ssao-evaluate")
        .expect("evaluate pass");
    assert!(evaluate.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW
            && resource
                .schema
                .and_then(RenderResourceSchema::texture_schema)
                .is_some_and(|schema| schema.extent == half_extent)
    }));

    let spatial = descriptor
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == "ssao-spatial-denoise")
        .expect("spatial pass");
    assert!(spatial.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::AMBIENT_OCCLUSION_SPATIAL
            && resource
                .schema
                .and_then(RenderResourceSchema::texture_schema)
                .is_some_and(|schema| schema.extent == half_extent)
    }));

    let upsample = descriptor
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == "ssao-bilateral-upsample")
        .expect("bilateral upsample pass");
    let upsample_compute = upsample
        .compute_pass
        .as_ref()
        .expect("upsample compute pass");
    assert!(matches!(
        &upsample_compute.dispatch,
        RenderGraphComputeDispatchExtent::PerPixel { target, .. }
            if target == PostProcessGraphResourceNames::AMBIENT_OCCLUSION
    ));
    assert!(upsample.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::AMBIENT_OCCLUSION_SPATIAL
            && resource
                .input_version
                .as_ref()
                .is_some_and(|version| version.producer_pass_name() == "ssao-spatial-denoise")
    }));
}

#[test]
fn ssao_evaluate_spatial_and_upsample_shaders_are_valid_wgsl() {
    for (label, source) in [
        (
            "evaluate",
            include_str!("../../../../scene/scene_renderer/post_process/shaders/ssao.wgsl"),
        ),
        (
            "spatial",
            include_str!(
                "../../../../scene/scene_renderer/post_process/shaders/ssao_spatial_denoise.wgsl"
            ),
        ),
        (
            "bilateral-upsample",
            include_str!(
                "../../../../scene/scene_renderer/post_process/shaders/ssao_bilateral_upsample.wgsl"
            ),
        ),
    ] {
        let module = naga::front::wgsl::parse_str(source)
            .unwrap_or_else(|error| panic!("{label}: {}", error.emit_to_string(source)));
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap_or_else(|error| panic!("{label}: {error}"));
    }
}
