use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderFramework,
    RenderLayerSet, RenderMeshSnapshot, RenderQualityProfile, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderStats, RenderViewportDescriptor, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Real, Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::graphics::runtime::WgpuRenderFramework;

const LARGE_STATIC_SCENE_MESH_COUNT: usize = 10_001;

#[test]
fn render_framework_reuses_static_index_and_reports_main_view_prefilter() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("visibility-static-index-submit")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_anti_alias(false),
        )
        .unwrap();

    server
        .submit_frame_extract(viewport, large_static_scene_extract(viewport_size))
        .unwrap();
    assert_large_static_scene_prefilter(server.query_stats().unwrap());

    server
        .submit_frame_extract(viewport, large_static_scene_extract(viewport_size))
        .unwrap();
    let second_frame_stats = server.query_stats().unwrap();

    assert_eq!(
        second_frame_stats.last_visibility_static_index_full_rebuild_count, 0,
        "the second static frame should update the viewport-owned static index instead of rebuilding it"
    );
    assert_large_static_scene_prefilter(second_frame_stats);
}

fn assert_large_static_scene_prefilter(stats: RenderStats) {
    assert!(stats.last_visibility_static_index_main_view_prefilter_used);
    assert_eq!(
        stats.last_visibility_static_index_main_view_static_input_count,
        LARGE_STATIC_SCENE_MESH_COUNT
    );
    assert!(
        stats.last_visibility_static_index_main_view_static_candidate_count
            < stats.last_visibility_static_index_main_view_static_input_count,
        "static-index coarse query should reduce main-view static candidates; input={}, candidates={}",
        stats.last_visibility_static_index_main_view_static_input_count,
        stats.last_visibility_static_index_main_view_static_candidate_count
    );
}

fn large_static_scene_extract(viewport_size: UVec2) -> RenderFrameExtract {
    let mut meshes = Vec::with_capacity(LARGE_STATIC_SCENE_MESH_COUNT);
    meshes.push(static_mesh(1, Vec3::new(0.0, 0.0, -5.0)));
    for index in 0..(LARGE_STATIC_SCENE_MESH_COUNT - 1) {
        meshes.push(static_mesh(
            1_000 + index as u64,
            Vec3::new(10_000.0 + index as Real * 32.0, 0.0, -5.0),
        ));
    }

    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes,
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    )
    .with_viewport_size(viewport_size)
}

fn static_mesh(node_id: u64, translation: Vec3) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: node_id << 16,
        transform_revision: 0,
        transform: Transform::from_translation(translation),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "builtin://material/pbr",
        )),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        static_state: Default::default(),
        render_layer_mask: RenderLayerSet::from_legacy_mask(u32::MAX),
    }
}
