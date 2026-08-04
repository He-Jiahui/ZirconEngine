use crate::core::framework::render::{
    AntiAliasSettings, CameraRenderDescriptor, PostProcessGraphResourceNames,
    PostProcessStackDescriptor, RenderBloomSettings, RenderBlurSettings,
    RenderDepthOfFieldSettings, RenderExposureSettings, RenderFrameExtract, RenderLayerSet,
    RenderMotionBlurSettings, RenderParticleSpriteSnapshot, RenderPhase, RenderPipelineHandle,
    RenderPostProcessEffectStackSettings, RenderScreenSpaceReflectionSettings,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::{Vec2, Vec3, Vec4};
use crate::graphics::feature::{
    BuiltinRenderFeature, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
};
use crate::graphics::pipeline::{
    RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions, RendererAsset,
};
use crate::render_graph::{
    QueueLane, RenderGraphAttachmentOps, RenderGraphComputeDispatchExtent,
    RenderGraphComputeWorkload, RenderGraphExternalResourceBinding, RenderGraphResourceAccessKind,
    RenderGraphResourceKind,
};
use crate::scene::world::World;

mod advanced_materials;
mod core_contracts;
mod external_compute_guards;
mod postprocess_routes;

#[test]
fn descriptor_filtering_builds_active_resources_once_per_descriptor() {
    let source = include_str!("descriptor_filtering.rs");
    let pass_start = source
        .find("fn filter_post_process_pass(")
        .expect("post-process pass filter");
    let pass_end = pass_start
        + source[pass_start..]
            .find("fn post_process_pass_can_be_filtered")
            .expect("next post-process helper");

    assert!(!source[pass_start..pass_end].contains("active_post_process_graph_resources(stack)"));
    assert_eq!(
        source
            .matches(concat!("active_", "post_process_graph_resources(stack)"))
            .count(),
        2
    );
    assert!(source.contains("let active_resources = OnceCell::new();"));
}

#[test]
fn graph_resource_authoring_moves_the_compiled_plan_without_cloning_it() {
    let source = include_str!("pass_authoring.rs");

    assert!(!source.contains(concat!("graph_resources: graph_resources", ".clone()")));
    assert!(source.contains(concat!(
        "graph_resources: BTreeMap<String, ",
        "PipelineGraphResourcePlan>"
    )));
}

#[test]
fn compile_half_resolution_transparency_is_profile_gated_and_declares_depth_aware_resources() {
    let mut extract = test_extract();
    extract
        .particles
        .sprites
        .push(RenderParticleSpriteSnapshot::default());
    let disabled = RenderPipelineAsset::default_forward_plus()
        .compile(&extract)
        .expect("default graph should compile");
    let enabled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_half_resolution_transparency(true)
                .with_half_resolution_transparency_depth_sigma(144),
        )
        .expect("half-resolution transparency graph should compile");

    assert_eq!(enabled.half_resolution_transparency_depth_sigma(), 144);

    assert!(!disabled
        .graph()
        .passes()
        .iter()
        .any(|pass| pass.name == "halfres-transparency-depth-downsample"));
    for pass_name in [
        "halfres-transparency-depth-downsample",
        "particle-render",
        "halfres-transparent-mesh",
        "halfres-transparency-composite",
    ] {
        assert!(
            enabled
                .graph()
                .passes()
                .iter()
                .any(|pass| pass.name == pass_name),
            "enabled graph should contain `{pass_name}`"
        );
    }

    assert_pass_reads(
        &enabled,
        "halfres-transparency-depth-downsample",
        PostProcessGraphResourceNames::SCENE_DEPTH,
    );
    assert_pass_writes(
        &enabled,
        "halfres-transparency-depth-downsample",
        PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_DEPTH,
    );
    assert_pass_writes(
        &enabled,
        "particle-render",
        PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_COLOR,
    );
    assert!(!enabled
        .graph()
        .passes()
        .iter()
        .any(|pass| pass.name == "transparent-mesh"));
    assert_pass_reads(
        &enabled,
        "halfres-transparent-mesh",
        PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_DEPTH,
    );
    assert_pass_writes(
        &enabled,
        "halfres-transparent-mesh",
        PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_COLOR,
    );
    assert_pass_reads(
        &enabled,
        "halfres-transparent-mesh",
        PostProcessGraphResourceNames::SHADOW_ATLAS,
    );
    for resource_name in [
        PostProcessGraphResourceNames::LIGHT_GRID_PARAMS,
        PostProcessGraphResourceNames::LIGHT_ZBINS,
        PostProcessGraphResourceNames::LIGHT_TILE_MASKS,
    ] {
        assert_pass_reads(&enabled, "halfres-transparent-mesh", resource_name);
    }
    assert_pass_reads(
        &enabled,
        "halfres-transparency-composite",
        PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_COLOR,
    );
    assert_pass_reads(
        &enabled,
        "halfres-transparency-composite",
        PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_DEPTH,
    );
    assert_pass_writes(
        &enabled,
        "halfres-transparency-composite",
        PostProcessGraphResourceNames::SCENE_COLOR,
    );

    let half_color = texture_lifetime(
        &enabled,
        PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_COLOR,
    );
    let half_depth = texture_lifetime(
        &enabled,
        PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_DEPTH,
    );
    assert_eq!(
        (half_color.width, half_color.height),
        (
            extract.view.effective_render_size().x.div_ceil(2),
            extract.view.effective_render_size().y.div_ceil(2)
        )
    );
    assert_eq!(half_color.format, crate::rhi::TextureFormat::Rgba16Float);
    assert_eq!(half_depth.format, crate::rhi::TextureFormat::Depth32Float);
    assert_eq!(half_color.sample_count, 1);
    assert_eq!(half_depth.sample_count, 1);
}

#[test]
fn compile_half_resolution_transparency_falls_back_for_multisample_graphs() {
    let extract = test_extract();
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_half_resolution_transparency(true)
                .with_graph_msaa_sample_count(4),
        )
        .expect("multisample graph should compile through the full-resolution fallback");

    for pass_name in [
        "halfres-transparency-depth-downsample",
        "halfres-transparency-composite",
    ] {
        assert!(
            !compiled
                .graph()
                .passes()
                .iter()
                .any(|pass| pass.name == pass_name),
            "multisample graph should not contain `{pass_name}`"
        );
    }
}

#[test]
fn compile_half_resolution_transparency_keeps_complete_graph_for_particle_only_renderer() {
    let mut extract = test_extract();
    extract
        .particles
        .sprites
        .push(RenderParticleSpriteSnapshot::default());
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_disabled(BuiltinRenderFeature::Mesh)
                .with_half_resolution_transparency(true),
        )
        .expect("particle-only half-resolution graph should compile");

    for pass_name in [
        "halfres-transparency-depth-downsample",
        "particle-render",
        "halfres-transparency-composite",
    ] {
        assert!(
            compiled
                .graph()
                .passes()
                .iter()
                .any(|pass| pass.name == pass_name),
            "particle-only graph should contain `{pass_name}`"
        );
    }
    assert_pass_writes(
        &compiled,
        "particle-render",
        PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_COLOR,
    );
}

#[test]
fn compile_half_resolution_transparency_preserves_plugin_transparent_pass_replacement() {
    let transparent_replacement = RenderFeatureDescriptor::new(
        "test-transparent-replacement",
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .with_replaced_pass("transparent-mesh");
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([transparent_replacement])
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default().with_half_resolution_transparency(true),
        )
        .expect("transparent pass replacement should compile before half-resolution injection");

    for pass_name in [
        "transparent-mesh",
        "halfres-transparency-depth-downsample",
        "halfres-transparency-composite",
    ] {
        assert!(
            !compiled
                .graph()
                .passes()
                .iter()
                .any(|pass| pass.name == pass_name),
            "plugin-owned transparent path should not contain `{pass_name}`"
        );
    }
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}

fn assert_pass_reads(
    compiled: &crate::graphics::pipeline::CompiledRenderPipeline,
    pass_name: &str,
    resource_name: &str,
) {
    let pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == pass_name)
        .unwrap_or_else(|| panic!("missing graph pass `{pass_name}`"));
    assert!(
        pass.resources.iter().any(|resource| {
            resource.name == resource_name && resource.access == RenderGraphResourceAccessKind::Read
        }),
        "`{pass_name}` should read `{resource_name}`"
    );
}

fn assert_pass_does_not_read(
    compiled: &crate::graphics::pipeline::CompiledRenderPipeline,
    pass_name: &str,
    resource_name: &str,
) {
    let pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == pass_name)
        .unwrap_or_else(|| panic!("missing graph pass `{pass_name}`"));
    assert!(
        !pass.resources.iter().any(|resource| {
            resource.name == resource_name && resource.access == RenderGraphResourceAccessKind::Read
        }),
        "`{pass_name}` should not read `{resource_name}`"
    );
}

fn assert_pass_writes(
    compiled: &crate::graphics::pipeline::CompiledRenderPipeline,
    pass_name: &str,
    resource_name: &str,
) {
    let pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == pass_name)
        .unwrap_or_else(|| panic!("missing graph pass `{pass_name}`"));
    assert!(
        pass.resources.iter().any(|resource| {
            resource.name == resource_name
                && resource.access == RenderGraphResourceAccessKind::Write
        }),
        "`{pass_name}` should write `{resource_name}`"
    );
}

fn texture_lifetime<'a>(
    compiled: &'a crate::graphics::pipeline::CompiledRenderPipeline,
    name: &str,
) -> &'a crate::rhi::TextureDesc {
    let lifetime = compiled
        .graph()
        .resource_lifetimes()
        .iter()
        .find(|lifetime| lifetime.name == name)
        .unwrap_or_else(|| panic!("missing graph resource lifetime `{name}`"));
    match &lifetime.desc {
        crate::render_graph::RenderGraphResourceDesc::Texture(desc) => desc,
        other => panic!("expected texture desc for `{name}`, got {other:?}"),
    }
}
