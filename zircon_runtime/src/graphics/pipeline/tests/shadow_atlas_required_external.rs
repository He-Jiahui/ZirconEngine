use crate::core::framework::render::{
    PostProcessGraphResourceNames, RenderFrameExtract, RenderWorldSnapshotHandle,
};
use crate::graphics::pipeline::{CompiledRenderPipeline, RenderPipelineAsset};
use crate::render_graph::{
    RenderGraphExternalResourceBinding, RenderGraphResourceAccessKind, RenderGraphResourceKind,
};
use crate::scene::world::World;

#[test]
fn compile_forward_plus_preserves_shadow_atlas_required_external_texture_binding() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&test_extract())
        .unwrap();

    assert_shadow_atlas_is_required_external_texture(&compiled);
    assert_graph_pass_uses_shadow_atlas(
        &compiled,
        "shadow-atlas",
        RenderGraphResourceAccessKind::Write,
    );
    for pass_name in ["opaque-mesh", "alpha-mask-mesh", "transparent-mesh"] {
        assert_graph_pass_uses_shadow_atlas(
            &compiled,
            pass_name,
            RenderGraphResourceAccessKind::Read,
        );
    }
}

#[test]
fn compile_deferred_preserves_shadow_atlas_required_external_texture_binding() {
    let compiled = RenderPipelineAsset::default_deferred()
        .compile(&test_extract())
        .unwrap();

    assert_shadow_atlas_is_required_external_texture(&compiled);
    assert_graph_pass_uses_shadow_atlas(
        &compiled,
        "shadow-atlas",
        RenderGraphResourceAccessKind::Write,
    );
    for pass_name in ["deferred-lighting", "transparent-mesh"] {
        assert_graph_pass_uses_shadow_atlas(
            &compiled,
            pass_name,
            RenderGraphResourceAccessKind::Read,
        );
    }
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}

fn assert_shadow_atlas_is_required_external_texture(compiled: &CompiledRenderPipeline) {
    let lifetime = compiled
        .graph()
        .resource_lifetime_by_name(PostProcessGraphResourceNames::SHADOW_ATLAS)
        .expect("shadow atlas external lifetime");

    assert_eq!(lifetime.kind, RenderGraphResourceKind::External);
    assert_eq!(
        lifetime.external_binding,
        RenderGraphExternalResourceBinding::required_texture()
    );
}

fn assert_graph_pass_uses_shadow_atlas(
    compiled: &CompiledRenderPipeline,
    pass_name: &str,
    access: RenderGraphResourceAccessKind,
) {
    let pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == pass_name)
        .unwrap_or_else(|| panic!("missing graph pass `{pass_name}`"));

    assert!(
        pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SHADOW_ATLAS
                && resource.kind == RenderGraphResourceKind::External
                && resource.access == access
        }),
        "graph pass `{pass_name}` does not use the shadow atlas as {access:?}"
    );
}
