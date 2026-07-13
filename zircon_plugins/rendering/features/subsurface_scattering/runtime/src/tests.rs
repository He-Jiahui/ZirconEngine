use super::*;
use zircon_runtime::core::framework::render::{
    RenderFrameExtract, RenderWorldSnapshotHandle, SubsurfaceProfileData,
};
use zircon_runtime::core::math::Vec3;
use zircon_runtime::graphics::{RenderPipelineAsset, RenderPipelineCompileOptions};
use zircon_runtime::scene::world::World;

#[test]
fn sss_feature_registers_three_passes_executors_and_shading_model() {
    let report = plugin_feature_registration();
    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, FEATURE_ID);
    assert!(!report.manifest.enabled_by_default);
    assert_eq!(report.extensions.render_features()[0].stage_passes.len(), 3);
    assert_eq!(report.extensions.render_pass_executors().len(), 3);
    assert_eq!(report.extensions.shading_models().len(), 1);
    assert_eq!(report.extensions.shading_models()[0].id, SHADING_MODEL_ID);
}

#[test]
fn sss_graph_requires_deferred_pipeline_and_non_empty_profile_table() {
    let extract = extract_with_profiles(true);
    let compiled = RenderPipelineAsset::default_deferred()
        .with_plugin_render_features([render_feature_descriptor()])
        .compile(&extract)
        .expect("deferred SSS graph should compile");
    for pass in [SETUP_PASS, SCATTER_PASS, RECOMBINE_PASS] {
        assert!(compiled
            .graph
            .passes()
            .iter()
            .any(|found| found.name == pass));
    }

    let no_profiles = extract_with_profiles(false);
    assert_exact_baseline(RenderPipelineAsset::default_deferred(), &no_profiles);
    assert_exact_baseline(RenderPipelineAsset::default_forward_plus(), &extract);
}

#[test]
fn explicitly_disabled_sss_feature_is_exact_graph_baseline() {
    let extract = extract_with_profiles(true);
    let baseline = RenderPipelineAsset::default_deferred()
        .compile(&extract)
        .unwrap();
    let disabled = RenderPipelineAsset::default_deferred()
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
fn sss_graph_is_removed_for_msaa_instead_of_creating_invalid_wgpu_bindings() {
    let extract = extract_with_profiles(true);
    let compiled = RenderPipelineAsset::default_deferred()
        .with_plugin_render_features([render_feature_descriptor()])
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_graph_msaa_sample_count(4),
        )
        .expect("deferred MSAA graph should fall back without SSS compute bindings");

    assert!([SETUP_PASS, SCATTER_PASS, RECOMBINE_PASS]
        .iter()
        .all(|pass| compiled
            .graph
            .passes()
            .iter()
            .all(|found| found.name != *pass)));
}

#[test]
fn forward_resolution_is_standard_pbr_fallback_with_diagnostic() {
    assert_eq!(
        resolve_subsurface_pipeline(true),
        SubsurfacePipelineResolution::DeferredScattering
    );
    let SubsurfacePipelineResolution::ForwardStandardPbrFallback { diagnostic } =
        resolve_subsurface_pipeline(false)
    else {
        panic!("forward path must resolve to explicit fallback");
    };
    assert!(diagnostic.contains("requires deferred rendering"));
}

fn assert_exact_baseline(pipeline: RenderPipelineAsset, extract: &RenderFrameExtract) {
    let baseline = pipeline.clone().compile(extract).unwrap();
    let installed = pipeline
        .with_plugin_render_features([render_feature_descriptor()])
        .compile(extract)
        .unwrap();
    assert_eq!(
        baseline.graph.dump().to_text(),
        installed.graph.dump().to_text()
    );
}

fn extract_with_profiles(with_profile: bool) -> RenderFrameExtract {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    );
    if with_profile {
        extract.lighting.advanced_lighting.subsurface_profiles = vec![SubsurfaceProfileData::new(
            0,
            Vec3::new(0.8, 1.2, 1.8),
            Vec3::new(1.0, 0.45, 0.3),
            1.0,
        )];
        extract
            .lighting
            .advanced_lighting
            .subsurface_material_profile_indices = vec![0];
    }
    extract
}
