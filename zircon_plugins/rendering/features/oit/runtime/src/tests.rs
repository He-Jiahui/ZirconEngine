use super::*;
use zircon_runtime::core::framework::render::{
    AdvancedLightingExtract, OitSettings, RenderFrameExtract, RenderWorldSnapshotHandle,
};
use zircon_runtime::core::math::UVec2;
use zircon_runtime::graphics::{
    RenderFeatureResourceAccess, RenderFeatureResourceKind, RenderPipelineAsset,
    RenderPipelineCompileOptions,
};
use zircon_runtime::render_graph::{RenderGraphResourceAccessKind, RenderGraphResourceDesc};
use zircon_runtime::scene::world::World;

#[test]
fn oit_feature_declares_dual_nodes_and_storage_buffers() {
    let report = plugin_feature_registration();
    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, FEATURE_ID);
    assert!(!report.manifest.enabled_by_default);

    let feature = &report.extensions.render_features()[0];
    assert_eq!(feature.name, FEATURE_NAME);
    assert_eq!(
        feature
            .stage_passes
            .iter()
            .map(|pass| pass.pass_name.as_str())
            .collect::<Vec<_>>(),
        vec![FRAGMENT_STORE_PASS, RESOLVE_PASS]
    );
    for resource in [
        PostProcessGraphResourceNames::OIT_LAYERS,
        PostProcessGraphResourceNames::OIT_COUNTS,
    ] {
        assert!(feature.stage_passes[0].resources.iter().any(|entry| {
            entry.name == resource
                && entry.kind == RenderFeatureResourceKind::Buffer
                && entry.access == RenderFeatureResourceAccess::Write
        }));
        assert!(feature.stage_passes[1].resources.iter().any(|entry| {
            entry.name == resource
                && entry.kind == RenderFeatureResourceKind::Buffer
                && entry.access == RenderFeatureResourceAccess::Read
        }));
    }
}

#[test]
fn oit_fragment_store_declares_forward_transparent_shading_inputs() {
    let descriptor = render_feature_descriptor();
    let pass = descriptor
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == FRAGMENT_STORE_PASS)
        .expect("OIT fragment-store pass");
    let resources = pass
        .resources
        .iter()
        .map(|resource| resource.name.as_str())
        .collect::<Vec<_>>();

    for required in [
        PostProcessGraphResourceNames::SCENE_DEPTH,
        PostProcessGraphResourceNames::SHADOW_ATLAS,
        PostProcessGraphResourceNames::LIGHT_GRID_PARAMS,
        PostProcessGraphResourceNames::LIGHT_ZBINS,
        PostProcessGraphResourceNames::LIGHT_TILE_MASKS,
        PostProcessGraphResourceNames::OIT_LAYERS,
        PostProcessGraphResourceNames::OIT_COUNTS,
    ] {
        assert!(resources.contains(&required), "missing `{required}`");
    }
    assert!(!resources.contains(&PostProcessGraphResourceNames::SCENE_COLOR));
}

#[test]
fn oit_enabled_replaces_sorted_transparency_and_sizes_buffers() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([render_feature_descriptor()])
        .compile(&oit_extract())
        .unwrap();
    let passes = pass_names(&compiled);

    assert!(!passes.contains(&"transparent-mesh"));
    assert_before(&passes, "preview-sky", FRAGMENT_STORE_PASS);
    assert_before(&passes, FRAGMENT_STORE_PASS, RESOLVE_PASS);
    assert_eq!(
        buffer_size(&compiled, PostProcessGraphResourceNames::OIT_LAYERS),
        320 * 180 * 4 * 8
    );
    assert_eq!(
        buffer_size(&compiled, PostProcessGraphResourceNames::OIT_COUNTS),
        320 * 180 * 4
    );
    assert!(compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == RESOLVE_PASS)
        .unwrap()
        .resources
        .iter()
        .any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_COLOR
                && resource.access == RenderGraphResourceAccessKind::Write
        }));
}

#[test]
fn oit_disabled_keeps_sorted_transparency_graph_baseline() {
    let extract = oit_extract();
    let baseline = RenderPipelineAsset::default_forward_plus()
        .compile(&extract)
        .unwrap();
    let disabled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([render_feature_descriptor()])
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_plugin_feature_disabled(FEATURE_NAME),
        )
        .unwrap();

    assert_eq!(
        baseline.graph.dump().to_text(),
        disabled.graph.dump().to_text()
    );
}

#[test]
fn oit_without_camera_settings_falls_back_to_sorted_transparency() {
    let extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    );
    let baseline = RenderPipelineAsset::default_forward_plus()
        .compile(&extract)
        .unwrap();
    let installed = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([render_feature_descriptor()])
        .compile(&extract)
        .unwrap();

    assert_eq!(
        baseline.graph.dump().to_text(),
        installed.graph.dump().to_text()
    );
    assert!(!installed
        .enabled_features
        .iter()
        .any(|feature| feature.feature_name() == FEATURE_NAME));
}

fn oit_extract() -> RenderFrameExtract {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    );
    extract.apply_viewport_size(UVec2::new(320, 180));
    extract.lighting.advanced_lighting = AdvancedLightingExtract {
        oit: Some(OitSettings::default()),
        ..AdvancedLightingExtract::default()
    };
    extract
}

fn pass_names(compiled: &zircon_runtime::graphics::CompiledRenderPipeline) -> Vec<&str> {
    compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect()
}

fn assert_before(passes: &[&str], before: &str, after: &str) {
    let before_index = passes.iter().position(|pass| *pass == before).unwrap();
    let after_index = passes.iter().position(|pass| *pass == after).unwrap();
    assert!(
        before_index < after_index,
        "{before} should precede {after}: {passes:?}"
    );
}

fn buffer_size(
    compiled: &zircon_runtime::graphics::CompiledRenderPipeline,
    resource_name: &str,
) -> u64 {
    let lifetime = compiled
        .graph
        .resource_lifetimes()
        .iter()
        .find(|lifetime| lifetime.name == resource_name)
        .unwrap();
    match &lifetime.desc {
        RenderGraphResourceDesc::Buffer(desc) => desc.size_bytes,
        other => panic!("expected OIT buffer, got {other:?}"),
    }
}
