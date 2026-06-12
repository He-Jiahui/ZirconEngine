use std::collections::BTreeSet;

use crate::core::framework::render::{
    ProjectionMode, RenderDirectionalLightSnapshot, RenderLayerSet, ViewportCameraSnapshot,
};
use crate::core::framework::scene::EntityId;
use crate::core::math::{is_finite_vec3, Real, Transform, Vec3};

use super::super::culling::parallel_frustum::{mesh_frustum_visibility, MeshFrustumCandidate};
use super::super::declarations::{VisibilityBvhInstance, VisibilityRelevanceEntry};
use super::{FrameVisibility, ViewCullingStats, ViewVisibilityContext, VisibilityViewKey};

const DEFAULT_SHADOW_LIGHT_DIRECTION_COMPONENTS: [Real; 3] = [-0.4, -1.0, -0.25];
const MIN_SHADOW_ORTHOGRAPHIC_HALF_EXTENT: Real = 4.0;
const SHADOW_CAMERA_DISTANCE_SCALE: Real = 2.0;
const SHADOW_CAMERA_FAR_PADDING: Real = 64.0;
const SHADOW_CAMERA_NEAR_PLANE: Real = 0.1;
const SHADOW_CAMERA_MIN_FAR_PLANE: Real = 1.0;
const SHADOW_UP_ALIGNMENT_LIMIT: Real = 0.95;

impl FrameVisibility {
    pub(crate) fn from_frame_views(
        camera: &ViewportCameraSnapshot,
        directional_lights: &[RenderDirectionalLightSnapshot],
        bvh_instances: &[VisibilityBvhInstance],
        primitive_relevance: &[VisibilityRelevanceEntry],
        visible_entities: &BTreeSet<EntityId>,
    ) -> Self {
        let mut frame_visibility =
            Self::from_main_view(camera, bvh_instances, primitive_relevance, visible_entities);
        let shadow_views = directional_lights
            .iter()
            .map(|light| shadow_cascade_view(&frame_visibility, camera, light))
            .collect::<Vec<_>>();
        frame_visibility.views.extend(shadow_views);
        frame_visibility
    }
}

fn shadow_cascade_view(
    frame_visibility: &FrameVisibility,
    main_camera: &ViewportCameraSnapshot,
    light: &RenderDirectionalLightSnapshot,
) -> ViewVisibilityContext {
    let camera = shadow_camera_for_light(frame_visibility, main_camera, light);
    let candidates = frame_visibility
        .entities
        .iter()
        .zip(frame_visibility.bounds.iter())
        .map(|(entity, bounds)| MeshFrustumCandidate {
            entity: *entity,
            bounds: *bounds,
        })
        .collect::<Vec<_>>();
    let frustum_visibility = mesh_frustum_visibility(&candidates, &camera);

    let mut visible = Vec::new();
    let mut relevance_filtered_count = 0usize;
    let mut frustum_culled_count = 0usize;
    for (index, relevance) in frame_visibility.relevance.iter().enumerate() {
        if !relevance.shadow_caster() {
            relevance_filtered_count += 1;
            continue;
        }
        if frustum_visibility
            .get(index)
            .is_some_and(|entry| entry.visible)
        {
            visible.push(
                u32::try_from(index).expect("frame visibility primitive index exceeds u32 range"),
            );
        } else {
            frustum_culled_count += 1;
        }
    }

    let visible_count = visible.len();
    ViewVisibilityContext {
        view: VisibilityViewKey::ShadowCascade {
            light: light.node_id,
            cascade: 0,
        },
        camera,
        visible,
        stats: ViewCullingStats {
            input_count: frame_visibility.entities.len(),
            layer_filtered_count: relevance_filtered_count,
            frustum_culled_count,
            occlusion_culled_count: 0,
            visible_count,
        },
    }
}

fn shadow_camera_for_light(
    frame_visibility: &FrameVisibility,
    main_camera: &ViewportCameraSnapshot,
    light: &RenderDirectionalLightSnapshot,
) -> ViewportCameraSnapshot {
    let direction = sanitize_direction(light.direction);
    let (center, radius) = shadow_bounds_from_frame(frame_visibility).unwrap_or((
        main_camera.transform.translation,
        MIN_SHADOW_ORTHOGRAPHIC_HALF_EXTENT,
    ));
    let half_extent = radius.max(MIN_SHADOW_ORTHOGRAPHIC_HALF_EXTENT);
    let distance = half_extent * SHADOW_CAMERA_DISTANCE_SCALE + SHADOW_CAMERA_FAR_PADDING;
    let far_plane = (distance + half_extent + SHADOW_CAMERA_FAR_PADDING)
        .max(SHADOW_CAMERA_NEAR_PLANE + SHADOW_CAMERA_MIN_FAR_PLANE);
    let eye = center - direction * distance;

    let mut camera = main_camera.clone();
    camera.transform = Transform::looking_at(eye, center, stable_shadow_up(direction));
    camera.projection_mode = ProjectionMode::Orthographic;
    camera.ortho_size = half_extent;
    camera.z_near = SHADOW_CAMERA_NEAR_PLANE;
    camera.z_far = far_plane;
    camera.aspect_ratio = 1.0;
    camera.render_layers = RenderLayerSet::from_legacy_mask(u32::MAX);
    camera
}

fn shadow_bounds_from_frame(frame_visibility: &FrameVisibility) -> Option<(Vec3, Real)> {
    let mut center = Vec3::ZERO;
    let mut count = 0usize;
    for bounds in &frame_visibility.bounds {
        if is_finite_vec3(bounds.center) {
            center += bounds.center;
            count += 1;
        }
    }
    if count == 0 {
        return None;
    }
    center /= count as Real;

    let mut radius: Real = 0.0;
    for bounds in &frame_visibility.bounds {
        if !is_finite_vec3(bounds.center) {
            continue;
        }
        let bounds_radius = if bounds.radius.is_finite() && bounds.radius > 0.0 {
            bounds.radius
        } else {
            0.0
        };
        radius = radius.max((bounds.center - center).length() + bounds_radius);
    }

    Some((center, radius.max(MIN_SHADOW_ORTHOGRAPHIC_HALF_EXTENT)))
}

fn stable_shadow_up(direction: Vec3) -> Vec3 {
    if direction.dot(Vec3::Y).abs() > SHADOW_UP_ALIGNMENT_LIMIT {
        Vec3::X
    } else {
        Vec3::Y
    }
}

fn sanitize_direction(direction: Vec3) -> Vec3 {
    if is_finite_vec3(direction) && direction.length_squared() > f32::EPSILON {
        direction.normalize_or_zero()
    } else {
        default_shadow_light_direction()
    }
}

fn default_shadow_light_direction() -> Vec3 {
    Vec3::from_array(DEFAULT_SHADOW_LIGHT_DIRECTION_COMPONENTS).normalize_or_zero()
}
