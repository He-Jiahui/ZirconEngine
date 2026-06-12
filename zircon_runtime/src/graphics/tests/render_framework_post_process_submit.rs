use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    MotionVectorCameraStatus, RenderDepthOfFieldSettings, RenderFrameExtract, RenderFramework,
    RenderMeshSnapshot, RenderMotionBlurSettings, RenderPostProcessEffectStackSettings,
    RenderQualityProfile, RenderScreenSpaceReflectionSettings, RenderViewportDescriptor,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::graphics::runtime::WgpuRenderFramework;
use crate::scene::world::World;

#[test]
fn render_framework_skips_advanced_postprocess_work_when_effects_are_disabled() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("default-postprocess-submit")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_history_resolve(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_anti_alias(false),
        )
        .unwrap();

    server
        .submit_frame_extract(viewport, default_post_process_extract(viewport_size))
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(
        stats.last_motion_vector_camera_status,
        MotionVectorCameraStatus::NotRequested
    );
    assert!(!stats
        .last_post_process_effect_stack_report
        .active_families
        .contains(&"depth-of-field".to_string()));
    assert!(!stats
        .last_post_process_effect_stack_report
        .active_families
        .contains(&"motion-blur".to_string()));
    assert!(!stats
        .last_post_process_effect_stack_report
        .active_families
        .contains(&"screen-space-reflection".to_string()));
}

#[test]
fn render_framework_submits_advanced_postprocess_graph_passes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("advanced-postprocess-submit")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_history_resolve(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_anti_alias(false),
        )
        .unwrap();

    server
        .submit_frame_extract(viewport, advanced_post_process_extract(viewport_size))
        .unwrap();
    assert_eq!(
        server
            .query_stats()
            .unwrap()
            .last_motion_vector_camera_status,
        MotionVectorCameraStatus::MissingPreviousCamera
    );
    let first_stats = server.query_stats().unwrap();
    assert_eq!(
        first_stats.last_motion_vector_previous_object_history_count,
        0
    );
    assert_eq!(
        first_stats.last_motion_vector_current_object_history_count,
        2
    );
    assert_eq!(
        first_stats.last_motion_vector_matched_object_history_count,
        0
    );
    assert_eq!(
        first_stats.last_motion_vector_missing_object_history_count,
        2
    );

    server
        .submit_frame_extract(viewport, advanced_post_process_extract(viewport_size))
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(
        stats.last_motion_vector_camera_status,
        MotionVectorCameraStatus::Ready
    );
    assert_eq!(stats.last_motion_vector_previous_object_history_count, 2);
    assert_eq!(stats.last_motion_vector_current_object_history_count, 2);
    assert_eq!(stats.last_motion_vector_matched_object_history_count, 2);
    assert_eq!(stats.last_motion_vector_missing_object_history_count, 0);
    for (pass_name, executor_id) in ADVANCED_POST_PROCESS_GRAPH_PASSES {
        assert!(
            stats
                .last_graph_executed_passes
                .iter()
                .any(|pass| pass == pass_name),
            "expected submitted frame to execute graph pass `{pass_name}`; executed={:?}",
            stats.last_graph_executed_passes
        );
        assert!(
            stats
                .last_graph_executed_executor_ids
                .iter()
                .any(|executor| executor == executor_id),
            "expected submitted frame to record executor `{executor_id}`; executors={:?}",
            stats.last_graph_executed_executor_ids
        );
    }
    assert_eq!(
        stats
            .last_graph_execution_coverage_report
            .missing_planned_pass_count,
        0
    );
    assert_eq!(
        stats
            .last_graph_execution_coverage_report
            .unexpected_executed_pass_count,
        0
    );
    assert!(stats
        .last_post_process_graph_executed_nodes
        .contains(&"effect-stack".to_string()));
    assert!(stats
        .last_post_process_graph_executed_nodes
        .contains(&"final-composite".to_string()));
    assert!(stats
        .last_post_process_effect_stack_report
        .active_families
        .contains(&"depth-of-field".to_string()));
    assert!(stats
        .last_post_process_effect_stack_report
        .active_families
        .contains(&"motion-blur".to_string()));
    assert!(stats
        .last_post_process_effect_stack_report
        .active_families
        .contains(&"screen-space-reflection".to_string()));
    assert!(
        !stats
            .last_post_process_effect_stack_report
            .missing_resources
            .iter()
            .any(|resource| resource.contains("velocity-prepass")),
        "executed motion-vector chain should clear velocity-prepass gaps; missing={:?}",
        stats
            .last_post_process_effect_stack_report
            .missing_resources
    );
}

const ADVANCED_POST_PROCESS_GRAPH_PASSES: &[(&str, &str)] = &[
    ("motion-vector-clear", "post.motion-vector-clear"),
    ("motion-vector-camera", "post.motion-vector-camera"),
    ("motion-vector-object", "post.motion-vector-object"),
    ("motion-vector-tile-max", "post.motion-vector-tile-max"),
    (
        "motion-vector-tile-max-coarse",
        "post.motion-vector-tile-max-coarse",
    ),
    (
        "motion-vector-neighbor-max",
        "post.motion-vector-neighbor-max",
    ),
    ("depth-of-field-prepare", "post.depth-of-field-prepare"),
    (
        "screen-space-reflection-depth-pyramid",
        "post.screen-space-reflection-depth-pyramid",
    ),
    (
        "screen-space-reflection-reflection-pyramid",
        "post.screen-space-reflection-reflection-pyramid",
    ),
    (
        "screen-space-reflection-depth-pyramid-coarse",
        "post.screen-space-reflection-depth-pyramid-coarse",
    ),
    (
        "screen-space-reflection-reflection-pyramid-coarse",
        "post.screen-space-reflection-reflection-pyramid-coarse",
    ),
    (
        "screen-space-reflection-specular-occlusion",
        "post.screen-space-reflection-specular-occlusion",
    ),
    (
        "screen-space-reflection-resolve",
        "post.screen-space-reflection-resolve",
    ),
    ("post-process", "post.stack"),
];

fn advanced_post_process_extract(viewport_size: UVec2) -> RenderFrameExtract {
    let mut extract = World::new().to_render_frame_extract();
    extract.apply_viewport_size(viewport_size);
    extract.geometry.meshes = vec![
        motion_vector_mesh(1001, Vec3::new(1.0, 0.0, 0.0)),
        motion_vector_mesh(1002, Vec3::new(-1.0, 0.0, 0.0)),
    ];
    extract.post_process.effect_stack = RenderPostProcessEffectStackSettings {
        motion_blur: RenderMotionBlurSettings {
            shutter_angle: 90.0,
            samples: 8,
        },
        depth_of_field: RenderDepthOfFieldSettings {
            aperture: 0.75,
            max_blur_radius: 3.0,
            ..Default::default()
        },
        screen_space_reflection: RenderScreenSpaceReflectionSettings {
            intensity: 0.5,
            max_steps: 24,
            ..Default::default()
        },
        ..Default::default()
    };
    extract
}

fn default_post_process_extract(viewport_size: UVec2) -> RenderFrameExtract {
    let mut extract = World::new().to_render_frame_extract();
    extract.apply_viewport_size(viewport_size);
    extract.geometry.meshes = vec![motion_vector_mesh(1001, Vec3::new(1.0, 0.0, 0.0))];
    extract
}

fn motion_vector_mesh(node_id: u64, translation: Vec3) -> RenderMeshSnapshot {
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
        mobility: Mobility::Dynamic,
        static_state: Default::default(),
        render_layer_mask: u32::MAX,
    }
}
