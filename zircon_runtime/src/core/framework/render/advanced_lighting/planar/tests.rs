use crate::core::framework::render::{
    CameraRenderDescriptor, RenderCameraTarget, RenderLayerSet, ViewProjectionMatrixPair,
    ViewportCameraSnapshot,
};
use crate::core::math::{Mat4, Transform, UVec2, Vec3, Vec4, perspective};
use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};

use super::{
    PlanarReflectionProbeData, PlanarReflectionQuality, PlanarReflectionUpdateState,
    PlanarUpdateMode, derive_planar_reflection_camera, planar_oblique_near_clip_projection,
    planar_reflection_matrix,
};

const EPSILON: f32 = 1.0e-5;

#[test]
fn render_planar_mirror_matrix_reflects_view() {
    let plane_point = Vec3::new(0.0, 1.0, 0.0);
    let plane_normal = Vec3::Y;
    let reflection = planar_reflection_matrix(plane_point, plane_normal)
        .expect("finite non-zero plane normal should produce a reflection matrix");
    let original = Vec3::new(2.0, 4.0, -3.0);
    let mirrored = reflection.transform_point3(original);

    assert_vec3_near(mirrored, Vec3::new(2.0, -2.0, -3.0));
    assert_mat4_near(reflection * reflection, Mat4::IDENTITY);

    let main_camera = Vec3::new(0.0, 3.0, 5.0);
    let main_target = Vec3::new(0.0, 1.0, 0.0);
    let mirrored_camera = reflection.transform_point3(main_camera);
    let mirrored_target = reflection.transform_point3(main_target);
    assert_near(
        main_camera.distance(main_target),
        mirrored_camera.distance(mirrored_target),
    );
}

#[test]
fn render_planar_oblique_clip_contains_plane() {
    let projection = perspective(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0);
    let reflection_plane_view = Vec4::new(0.0, 0.0, -1.0, -1.0);
    let oblique = planar_oblique_near_clip_projection(projection, reflection_plane_view)
        .expect("visible view-space plane should produce an oblique projection");

    for point in [
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::new(0.5, -0.25, -1.0),
        Vec3::new(-0.4, 0.3, -1.0),
    ] {
        let clip = oblique * point.extend(1.0);
        assert_near(clip.z, 0.0);
    }

    let retained = oblique * Vec3::new(0.0, 0.0, -2.0).extend(1.0);
    let rejected = oblique * Vec3::new(0.0, 0.0, -0.5).extend(1.0);
    assert!(retained.z > 0.0 && retained.z <= retained.w);
    assert!(rejected.z < 0.0);
}

#[test]
fn render_planar_quality_tiers_and_update_default_match_plan() {
    assert_eq!(PlanarUpdateMode::default(), PlanarUpdateMode::OnDemand);
    assert_eq!(PlanarReflectionQuality::Low.resolution(), 256);
    assert_eq!(PlanarReflectionQuality::Medium.resolution(), 512);
    assert_eq!(PlanarReflectionQuality::High.resolution(), 1024);
}

#[test]
fn render_planar_camera_derivation_targets_texture_and_applies_oblique_clip() {
    let mut main = CameraRenderDescriptor::from_camera_payload(
        Some(73),
        ViewportCameraSnapshot {
            transform: Transform::looking_at(Vec3::new(0.0, 3.0, 5.0), Vec3::ZERO, Vec3::Y),
            ..ViewportCameraSnapshot::default()
        },
    );
    main.render_order = 7;
    let layer_mask = RenderLayerSet::layer(3);
    let target = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "runtime://planar/probe-91",
    ));
    let probe = PlanarReflectionProbeData {
        probe_id: 91,
        plane_transform: Mat4::IDENTITY,
        local_reference_position: Vec3::ZERO,
        bounds_min: Vec3::splat(-10.0),
        bounds_max: Vec3::splat(10.0),
        resolution: PlanarReflectionQuality::Medium.resolution(),
        update: PlanarUpdateMode::OnDemand,
        capture_target: Some(target),
        layer_mask: layer_mask.clone(),
    };

    let reflected = derive_planar_reflection_camera(&main, &probe, target)
        .expect("valid planar probe should derive a reflection camera");

    assert_eq!(reflected.render_order, 6);
    assert_eq!(reflected.target, RenderCameraTarget::Texture(target));
    assert_eq!(reflected.culling_mask, layer_mask);
    assert_near(reflected.camera.transform.translation.y, -3.0);
    assert!(reflected.camera.projection_override.is_some());

    let matrix_pair = ViewProjectionMatrixPair::from_camera(
        &reflected.camera,
        UVec2::new(probe.resolution, probe.resolution),
    );
    for point in [Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, -1.0)] {
        let clip = matrix_pair.clip_from_world_unjittered * point.extend(1.0);
        assert_near(clip.z, 0.0);
    }
}

#[test]
fn render_planar_on_demand_state_captures_first_dirty_and_every_frame_updates() {
    let mut state = PlanarReflectionUpdateState::default();
    let mut probe = test_probe();

    assert!(state.should_capture(&probe));
    state.mark_captured(probe.probe_id);
    assert!(!state.should_capture(&probe));

    state.mark_dirty(probe.probe_id);
    assert!(state.should_capture(&probe));
    state.mark_captured(probe.probe_id);
    assert!(!state.should_capture(&probe));

    probe.update = PlanarUpdateMode::EveryFrame;
    assert!(state.should_capture(&probe));
    state.mark_captured(probe.probe_id);
    assert!(state.should_capture(&probe));

    state.forget(probe.probe_id);
    probe.update = PlanarUpdateMode::OnDemand;
    assert!(state.should_capture(&probe));
}

fn test_probe() -> PlanarReflectionProbeData {
    PlanarReflectionProbeData {
        probe_id: 91,
        plane_transform: Mat4::IDENTITY,
        local_reference_position: Vec3::ZERO,
        bounds_min: Vec3::splat(-10.0),
        bounds_max: Vec3::splat(10.0),
        resolution: PlanarReflectionQuality::Medium.resolution(),
        update: PlanarUpdateMode::OnDemand,
        capture_target: None,
        layer_mask: RenderLayerSet::default(),
    }
}

fn assert_mat4_near(actual: Mat4, expected: Mat4) {
    for (actual, expected) in actual
        .to_cols_array()
        .into_iter()
        .zip(expected.to_cols_array())
    {
        assert_near(actual, expected);
    }
}

fn assert_vec3_near(actual: Vec3, expected: Vec3) {
    assert_near(actual.x, expected.x);
    assert_near(actual.y, expected.y);
    assert_near(actual.z, expected.z);
}

fn assert_near(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= EPSILON,
        "expected {expected}, got {actual}"
    );
}
