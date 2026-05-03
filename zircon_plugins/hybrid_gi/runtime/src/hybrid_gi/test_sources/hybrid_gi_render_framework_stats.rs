use std::fs;
use std::path::PathBuf;

use zircon_runtime::core::framework::render::{
    DisplayMode, FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode, RenderFrameExtract,
    RenderFramework, RenderHybridGiDebugView, RenderHybridGiExtract, RenderHybridGiQuality,
    RenderMeshSnapshot, RenderOverlayExtract, RenderQualityProfile, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderViewportDescriptor, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::{Transform, UVec2, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle};

use crate::test_support::render_feature_fixtures::pluginized_wgpu_render_framework_with_asset_manager;

use super::hybrid_gi_scene_prepare_material_fixtures::{
    material_capture_test_assets, model_handle,
};

#[test]
fn render_framework_stats_expose_scene_representation_screen_probe_and_radiance_cache_counts() {
    let (asset_manager, root, black_material, emissive_material) = material_capture_test_assets();
    let _cleanup = TempProjectCleanup(root);
    let model = model_handle(&asset_manager);
    let viewport_size = UVec2::new(160, 120);
    let extract =
        scene_representation_extract(viewport_size, model, black_material, emissive_material);

    let server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(viewport, hybrid_gi_only_quality_profile())
        .unwrap();
    server.submit_frame_extract(viewport, extract).unwrap();

    let stats = server.query_stats().unwrap();
    assert_eq!(stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert_eq!(stats.last_hybrid_gi_active_probe_count, 0);
    assert_eq!(stats.last_hybrid_gi_requested_probe_count, 0);
    assert_eq!(stats.last_hybrid_gi_dirty_probe_count, 0);
    assert_eq!(
        stats.last_hybrid_gi_scene_card_count, 2,
        "expected public RenderFramework stats to expose scene-representation cards without direct renderer readback access"
    );
    assert_eq!(
        stats.last_hybrid_gi_surface_cache_resident_page_count, 1,
        "expected the HGI plugin runtime provider to project card-budgeted surface-cache residency through neutral RenderStats"
    );
    assert_eq!(
        stats.last_hybrid_gi_surface_cache_feedback_card_count, 1,
        "expected the over-budget second scene card to remain visible as plugin-owned surface-cache feedback"
    );
    assert_eq!(
        stats.last_hybrid_gi_scene_screen_probe_count, 2,
        "expected screen-probe placement from scene-representation budgets to cross only the public RenderFramework stats seam"
    );
    assert_eq!(
        stats.last_hybrid_gi_scene_radiance_cache_entry_count, 2,
        "expected one radiance-cache seed per screen probe without reopening renderer-private HGI frame internals"
    );
}

fn scene_representation_extract(
    viewport_size: UVec2,
    model: ResourceHandle<ModelMarker>,
    first_material: ResourceHandle<MaterialMarker>,
    second_material: ResourceHandle<MaterialMarker>,
) -> RenderFrameExtract {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 6.0),
            ..Transform::default()
        },
        projection_mode: ProjectionMode::Orthographic,
        ortho_size: 6.0,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(viewport_size);

    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: vec![
                mesh(
                    11,
                    model.clone(),
                    first_material,
                    Vec3::new(-1.0, 0.0, 0.0),
                    2.0,
                ),
                mesh(22, model, second_material, Vec3::new(3.0, 0.0, 0.0), 1.0),
            ],
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..RenderOverlayExtract::default()
        },
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        quality: RenderHybridGiQuality::High,
        trace_budget: 2,
        card_budget: 1,
        voxel_budget: 1,
        debug_view: RenderHybridGiDebugView::SurfaceCache,
        probe_budget: 0,
        tracing_budget: 0,
        probes: Vec::new(),
        trace_regions: Vec::new(),
    });
    extract
}

fn mesh(
    node_id: u64,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
    translation: Vec3,
    uniform_scale: f32,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        transform: Transform::from_translation(translation).with_scale(Vec3::splat(uniform_scale)),
        model,
        material,
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        render_layer_mask: u32::MAX,
    }
}

fn hybrid_gi_only_quality_profile() -> RenderQualityProfile {
    RenderQualityProfile::new("hgi-scene-representation-stats")
        .with_virtual_geometry(false)
        .with_hybrid_global_illumination(true)
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_history_resolve(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_reflection_probes(false)
        .with_baked_lighting(false)
        .with_particle_rendering(false)
        .with_async_compute(false)
}

struct TempProjectCleanup(PathBuf);

impl Drop for TempProjectCleanup {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
