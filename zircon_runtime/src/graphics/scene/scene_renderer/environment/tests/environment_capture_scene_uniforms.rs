use super::*;
use crate::core::framework::render::{
    EnvironmentExtract, PreviewEnvironmentExtract, RenderEnvironmentCaptureRequest,
    RenderOverlayExtract, RenderSceneGeometryExtract, SceneViewportRenderPacket,
    ViewportCameraSnapshot,
};
use crate::core::math::Vec4;

#[test]
fn one_packed_payload_retains_six_distinct_face_uniforms() {
    let request = RenderEnvironmentCaptureRequest::new("probe", [4.0, 5.0, 6.0], 7)
        .unwrap()
        .with_face_size(256)
        .unwrap();
    let mut batch = EnvironmentCaptureSceneBatch::new(test_scene(), request);

    let plan = EnvironmentCaptureSceneUniformPlan::from_scene_batch(&mut batch, 0.75);

    assert_eq!(
        plan.payload_bytes(),
        CUBEMAP_FACE_COUNT * size_of::<SceneUniform>()
    );
    assert_eq!(size_of::<SceneUniform>(), 496);
    assert_eq!(plan.payload_bytes(), 2_976);
    assert_eq!(batch.selected_face(), Some(CubemapFace::NegativeZ));
    for face in CubemapFace::ALL {
        assert_eq!(
            plan.uniform(face).camera_world_position,
            [4.0, 5.0, 6.0, 0.75]
        );
        assert_eq!(plan.uniform(face).sky_sun_params[3], 1.0);
        assert_eq!(face_payload_range(face).len(), size_of::<SceneUniform>());
    }
    assert_ne!(
        plan.uniform(CubemapFace::PositiveX).view_proj_unjittered,
        plan.uniform(CubemapFace::NegativeX).view_proj_unjittered
    );
}

#[test]
fn capture_surface_policy_is_shared_by_forward_templates_and_fallback() {
    const SURFACE_TYPES: &str = include_str!("../../../../shader/wgsl/zr_surface_types.wgsl");
    const FORWARD: &str = include_str!("../../../../shader/wgsl/zr_template_forward.wgsl");
    const ENVIRONMENT_ONLY: &str =
        include_str!("../../../../shader/wgsl/zr_template_forward_environment_only_pbr.wgsl");
    const FALLBACK: &str = include_str!("../../mesh/shaders/fallback_mesh.wgsl");

    assert!(SURFACE_TYPES.contains("fn zr_surface_apply_environment_capture_policy("));
    assert!(SURFACE_TYPES.contains("resolved.clearcoat_roughness = 1.0;"));
    for source in [FORWARD, ENVIRONMENT_ONLY] {
        let alpha_clip = source.find("zr_apply_alpha_clip(surface);").unwrap();
        let capture_policy = source
            .find("surface = zr_surface_apply_environment_capture_policy(surface);")
            .unwrap();
        assert!(alpha_clip < capture_policy);
    }
    assert!(FALLBACK
        .contains("material.roughness = zr_environment_capture_roughness(material.roughness);"));
}

#[test]
fn packed_face_ranges_are_contiguous_and_non_overlapping() {
    let mut previous_end = 0;
    for face in CubemapFace::ALL {
        let range = face_payload_range(face);
        assert_eq!(range.start, previous_end);
        previous_end = range.end;
    }
    assert_eq!(previous_end, CUBEMAP_FACE_COUNT * size_of::<SceneUniform>());
}

#[test]
fn realtime_ibl_override_is_packed_into_every_face_without_a_second_payload() {
    let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1).unwrap();
    let mut batch = EnvironmentCaptureSceneBatch::new(test_scene(), request);

    let plan = EnvironmentCaptureSceneUniformPlan::from_scene_batch_with_realtime_ibl(
        &mut batch, 0.0, 128, 64, 7,
    );

    assert_eq!(plan.payload_bytes(), 2_976);
    for face in CubemapFace::ALL {
        assert_eq!(
            plan.uniform(face).environment_sample_params,
            [4.0, 128.0, 64.0, 7.0]
        );
    }
}

#[test]
fn capture_light_grid_packs_lights_once_and_owns_six_face_views() {
    let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1)
        .unwrap()
        .with_face_size(128)
        .unwrap();
    let mut scene = test_scene();
    scene.scene.directional_lights.push(
        crate::core::framework::render::RenderDirectionalLightSnapshot {
            node_id: 7,
            light_id: 7,
            layer_mask: crate::core::framework::render::RenderLayerSet::default(),
            direction: crate::core::math::Vec3::NEG_Y,
            color: crate::core::math::Vec3::ONE,
            intensity: 2.0,
            mobility: crate::core::framework::scene::Mobility::Dynamic,
            shadow: None,
        },
    );
    let mut batch = EnvironmentCaptureSceneBatch::new(scene, request);

    let plan = EnvironmentCaptureLightGridPlan::from_scene_batch(&mut batch);

    assert_eq!(plan.light_count(), 1);
    assert_eq!(plan.face_count(), CUBEMAP_FACE_COUNT);
    assert_eq!(plan.upload_count(), CUBEMAP_FACE_COUNT * 3);
    assert_eq!(plan.payload_bytes(), 105_192);
    assert_ne!(
        plan.grid(CubemapFace::PositiveX).params.world_to_view,
        plan.grid(CubemapFace::NegativeX).params.world_to_view
    );
}

#[test]
fn capture_light_grid_empty_path_builds_no_face_payload() {
    let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1).unwrap();
    let mut batch = EnvironmentCaptureSceneBatch::new(test_scene(), request);

    let plan = EnvironmentCaptureLightGridPlan::from_scene_batch(&mut batch);

    assert_eq!(plan.light_count(), 0);
    assert_eq!(plan.face_count(), 0);
    assert_eq!(plan.upload_count(), 0);
    assert_eq!(plan.payload_bytes(), 0);
    assert_eq!(batch.selected_face(), None);
}

fn test_scene() -> SceneViewportRenderPacket {
    let environment = EnvironmentExtract::default();
    SceneViewportRenderPacket {
        scene: RenderSceneGeometryExtract {
            camera: ViewportCameraSnapshot::default(),
            meshes: Vec::new(),
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract::default(),
        preview: PreviewEnvironmentExtract::from_environment(&environment, false, Vec4::ZERO),
        environment,
        virtual_geometry_debug: None,
    }
}
