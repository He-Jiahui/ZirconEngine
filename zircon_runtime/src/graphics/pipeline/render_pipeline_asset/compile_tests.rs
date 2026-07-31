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
