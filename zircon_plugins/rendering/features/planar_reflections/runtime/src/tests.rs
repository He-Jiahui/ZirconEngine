use super::*;
use zircon_runtime::core::framework::render::{
    CameraRenderDescriptor, PlanarReflectionProbeData, PlanarReflectionQuality, PlanarUpdateMode,
    RenderCameraTarget, RenderFrameExtract, RenderLayerSet, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use zircon_runtime::core::math::{Mat4, Vec3};
use zircon_runtime::core::resource::{ResourceHandle, ResourceId, TextureMarker};
use zircon_runtime::graphics::{RenderPipelineAsset, RenderPipelineCompileOptions};
use zircon_runtime::scene::world::World;

#[test]
fn planar_feature_registers_filter_executor_and_stays_default_off() {
    let report = plugin_feature_registration();
    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, FEATURE_ID);
    assert!(!report.manifest.enabled_by_default);
    assert_eq!(
        report.extensions.render_features()[0].stage_passes[0].pass_name,
        FILTER_PASS
    );
    assert!(report.extensions.render_features()[0].stage_passes[0]
        .compute_workload
        .is_some());
    assert_eq!(report.extensions.render_pass_executors().len(), 1);
    assert_eq!(
        report.extensions.render_pass_executors()[0]
            .executor_id()
            .as_str(),
        FILTER_PASS
    );
}

#[test]
fn planar_filter_enters_only_the_owned_capture_camera_graph() {
    let (capture, target) = planar_extract(true);
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([render_feature_descriptor()])
        .compile(&capture)
        .unwrap();
    assert!(compiled
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == FILTER_PASS));

    let (main, _) = planar_extract(false);
    let baseline = RenderPipelineAsset::default_forward_plus()
        .compile(&main)
        .unwrap();
    let installed = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([render_feature_descriptor()])
        .compile(&main)
        .unwrap();
    assert_eq!(
        baseline.graph.dump().to_text(),
        installed.graph.dump().to_text()
    );
    assert!(
        matches!(capture.view.selected_camera_target(), RenderCameraTarget::Texture(found) if *found == target)
    );
}

#[test]
fn explicitly_disabled_planar_feature_is_exact_graph_baseline() {
    let (extract, _) = planar_extract(true);
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

fn planar_extract(capture_selected: bool) -> (RenderFrameExtract, ResourceHandle<TextureMarker>) {
    let target = ResourceHandle::new(ResourceId::from_stable_label("tests/planar/capture"));
    let mut camera =
        CameraRenderDescriptor::from_camera_payload(Some(7), ViewportCameraSnapshot::default());
    if capture_selected {
        camera.target = RenderCameraTarget::Texture(target);
    }
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    );
    extract.view.cameras = vec![camera.clone()];
    extract.view.scene_camera_entity = camera.entity;
    extract.lighting.advanced_lighting.planar_probes = vec![PlanarReflectionProbeData {
        probe_id: 11,
        plane_transform: Mat4::IDENTITY,
        local_reference_position: Vec3::ZERO,
        bounds_min: Vec3::splat(-8.0),
        bounds_max: Vec3::splat(8.0),
        resolution: PlanarReflectionQuality::Low.resolution(),
        update: PlanarUpdateMode::OnDemand,
        capture_target: Some(target),
        layer_mask: RenderLayerSet::default(),
    }];
    (extract, target)
}
