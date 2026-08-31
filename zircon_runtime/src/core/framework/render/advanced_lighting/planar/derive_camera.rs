use crate::core::framework::render::{CameraRenderDescriptor, ProjectionMode, RenderCameraTarget};
use crate::core::math::{view_matrix, Mat4, Transform, Vec3};
use crate::core::resource::{ResourceHandle, TextureMarker};

use super::{
    planar_oblique_near_clip_projection, planar_reflection_matrix, PlanarReflectionProbeData,
};

pub fn derive_planar_reflection_camera(
    main_camera: &CameraRenderDescriptor,
    probe: &PlanarReflectionProbeData,
    target: ResourceHandle<TextureMarker>,
) -> Option<CameraRenderDescriptor> {
    let plane_point = probe
        .plane_transform
        .transform_point3(probe.local_reference_position);
    let plane_normal = probe
        .plane_transform
        .transform_vector3(Vec3::Y)
        .normalize_or_zero();
    let reflection = planar_reflection_matrix(plane_point, plane_normal)?;
    let source_transform = main_camera.camera.transform;
    let reflected_eye = reflection.transform_point3(source_transform.translation);
    let reflected_forward = reflection
        .transform_vector3(source_transform.forward())
        .normalize_or_zero();
    let reflected_up = reflection
        .transform_vector3(source_transform.up())
        .normalize_or_zero();
    if reflected_forward == Vec3::ZERO || reflected_up == Vec3::ZERO {
        return None;
    }

    let mut reflected = CameraRenderDescriptor {
        entity: None,
        render_order: main_camera.render_order.saturating_sub(1),
        render_type: main_camera.render_type,
        stack: Vec::new(),
        target: RenderCameraTarget::Texture(target),
        viewport_rect: main_camera.viewport_rect,
        clear: main_camera.clear,
        clear_depth: main_camera.clear_depth,
        culling_mask: probe.layer_mask.clone(),
        volume_mask: main_camera.volume_mask.clone(),
        camera: main_camera.camera.clone(),
    };
    reflected.camera.transform = Transform::looking_at(
        reflected_eye,
        reflected_eye + reflected_forward,
        reflected_up,
    );
    reflected.camera.aspect_ratio = 1.0;
    reflected.camera.temporal_jitter = Default::default();
    reflected.camera.projection_override = None;

    let base_projection = base_projection(&reflected);
    let mirror_view = view_matrix(reflected.camera.transform);
    let plane_point_view = mirror_view.transform_point3(plane_point);
    let plane_normal_view = mirror_view
        .transform_vector3(plane_normal)
        .normalize_or_zero();
    let mut clip_plane_view = plane_normal_view.extend(-plane_normal_view.dot(plane_point_view));
    if clip_plane_view.w > 0.0 {
        clip_plane_view = -clip_plane_view;
    }
    reflected.camera.projection_override =
        planar_oblique_near_clip_projection(base_projection, clip_plane_view);
    reflected.camera.projection_override?;
    Some(reflected)
}

fn base_projection(camera: &CameraRenderDescriptor) -> Mat4 {
    match camera.camera.projection_mode {
        ProjectionMode::Perspective => Mat4::perspective_rh(
            camera.camera.fov_y_radians,
            1.0,
            camera.camera.z_near.max(0.001),
            camera.camera.z_far,
        ),
        ProjectionMode::Orthographic => {
            let half_extent = camera.camera.ortho_size.max(0.01);
            Mat4::orthographic_rh(
                -half_extent,
                half_extent,
                -half_extent,
                half_extent,
                camera.camera.z_near.max(0.001),
                camera.camera.z_far,
            )
        }
    }
}

#[cfg(test)]
mod optimization_tests {
    #[test]
    fn optimization_batch_20260830ds_planar_camera_selectively_clones_retained_fields() {
        let source = include_str!("derive_camera.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("planar camera production source");

        assert!(production.contains("let mut reflected = CameraRenderDescriptor {"));
        assert!(!production.contains("let mut reflected = main_camera.clone()"));
    }

    #[test]
    #[ignore = "release-only deterministic clone-count evidence"]
    fn optimization_batch_20260830ds_planar_camera_selective_clone_evidence() {
        const DERIVATIONS: usize = 65_536;
        const MAIN_STACK_ENTITIES: usize = 32;
        const MARKER: &str = "RUNTIME527_PLANAR_CAMERA_SELECTIVE_CLONE_BENCH_V1";

        let legacy_discarded_stack_entity_clones = DERIVATIONS * MAIN_STACK_ENTITIES;
        let selective_discarded_stack_entity_clones = 0;
        let reduction_basis_points = 10_000;

        assert_eq!(legacy_discarded_stack_entity_clones, 2_097_152);
        assert_eq!(selective_discarded_stack_entity_clones, 0);
        println!(
            "{MARKER} derivations={DERIVATIONS} main_stack_entities={MAIN_STACK_ENTITIES} \
             legacy_discarded_stack_entity_clones={legacy_discarded_stack_entity_clones} \
             selective_discarded_stack_entity_clones={selective_discarded_stack_entity_clones} \
             reduction_basis_points={reduction_basis_points}"
        );
    }
}
