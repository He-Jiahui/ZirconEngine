use std::collections::BTreeSet;
use std::f32::consts::{FRAC_PI_2, PI};

use crate::core::framework::render::{
    LightShadowSettings, LightingExtract, ProjectionMode, RenderDirectionalLightSnapshot,
    RenderLayerSet, RenderPointLightSnapshot, RenderSpotLightSnapshot, ViewportCameraSnapshot,
};
use crate::core::framework::scene::EntityId;
use crate::core::math::{is_finite_vec3, Real, Transform, Vec3};
use crate::graphics::scene::{
    cascade_shadow_bounds_from_camera_slice, compute_cascade_ranges, CascadeRange,
    CascadeSplitConfig,
};

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
const POINT_LIGHT_SHADOW_FACE_COUNT: u8 = 6;
const MIN_PUNCTUAL_SHADOW_RANGE: Real = SHADOW_CAMERA_NEAR_PLANE + SHADOW_CAMERA_MIN_FAR_PLANE;

impl FrameVisibility {
    pub(crate) fn from_frame_views(
        camera: &ViewportCameraSnapshot,
        lighting: &LightingExtract,
        bvh_instances: &[VisibilityBvhInstance],
        primitive_relevance: &[VisibilityRelevanceEntry],
        visible_entities: &BTreeSet<EntityId>,
    ) -> Self {
        let mut frame_visibility =
            Self::from_main_view(camera, bvh_instances, primitive_relevance, visible_entities);
        let mut shadow_views = Vec::new();
        for light in &lighting.directional_lights {
            let ranges = directional_shadow_ranges(light.shadow);
            shadow_views.extend(ranges.into_iter().enumerate().map(|(cascade, range)| {
                shadow_cascade_view(&frame_visibility, camera, light, cascade as u8, range)
            }));
        }
        for light in lighting
            .point_lights
            .iter()
            .filter(|light| shadow_enabled(light.shadow))
        {
            shadow_views.extend(
                (0..POINT_LIGHT_SHADOW_FACE_COUNT)
                    .map(|face| point_shadow_face_view(&frame_visibility, light, face)),
            );
        }
        shadow_views.extend(
            lighting
                .spot_lights
                .iter()
                .filter(|light| shadow_enabled(light.shadow))
                .map(|light| spot_shadow_view(&frame_visibility, light)),
        );
        frame_visibility.views.extend(shadow_views);
        frame_visibility
    }
}

fn shadow_cascade_view(
    frame_visibility: &FrameVisibility,
    main_camera: &ViewportCameraSnapshot,
    light: &RenderDirectionalLightSnapshot,
    cascade: u8,
    range: CascadeRange,
) -> ViewVisibilityContext {
    let camera = shadow_camera_for_light(main_camera, light, range);
    shadow_view_from_camera(
        frame_visibility,
        VisibilityViewKey::ShadowCascade {
            light: light.node_id,
            cascade,
        },
        camera,
    )
}

fn point_shadow_face_view(
    frame_visibility: &FrameVisibility,
    light: &RenderPointLightSnapshot,
    face: u8,
) -> ViewVisibilityContext {
    shadow_view_from_camera(
        frame_visibility,
        VisibilityViewKey::ShadowPointFace {
            light: light.node_id,
            face,
        },
        point_shadow_face_camera(light, face),
    )
}

fn spot_shadow_view(
    frame_visibility: &FrameVisibility,
    light: &RenderSpotLightSnapshot,
) -> ViewVisibilityContext {
    shadow_view_from_camera(
        frame_visibility,
        VisibilityViewKey::ShadowSpot {
            light: light.node_id,
        },
        spot_shadow_camera(light),
    )
}

fn shadow_view_from_camera(
    frame_visibility: &FrameVisibility,
    view: VisibilityViewKey,
    camera: ViewportCameraSnapshot,
) -> ViewVisibilityContext {
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
        view,
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
    main_camera: &ViewportCameraSnapshot,
    light: &RenderDirectionalLightSnapshot,
    range: CascadeRange,
) -> ViewportCameraSnapshot {
    let direction = sanitize_direction(light.direction);
    let bounds = cascade_shadow_bounds_from_camera_slice(main_camera, range);
    let half_extent = bounds.radius.max(MIN_SHADOW_ORTHOGRAPHIC_HALF_EXTENT);
    let distance = half_extent * SHADOW_CAMERA_DISTANCE_SCALE + SHADOW_CAMERA_FAR_PADDING;
    let far_plane = (distance + half_extent + SHADOW_CAMERA_FAR_PADDING)
        .max(SHADOW_CAMERA_NEAR_PLANE + SHADOW_CAMERA_MIN_FAR_PLANE)
        .max(range.far);
    let eye = bounds.center - direction * distance;

    let mut camera = main_camera.clone();
    camera.transform = Transform::looking_at(eye, bounds.center, stable_shadow_up(direction));
    camera.projection_mode = ProjectionMode::Orthographic;
    camera.ortho_size = half_extent;
    camera.z_near = SHADOW_CAMERA_NEAR_PLANE;
    camera.z_far = far_plane;
    camera.aspect_ratio = 1.0;
    camera.render_layers = RenderLayerSet::from_legacy_mask(u32::MAX);
    camera
}

fn point_shadow_face_camera(light: &RenderPointLightSnapshot, face: u8) -> ViewportCameraSnapshot {
    let (direction, up) = point_light_face_axes(face);
    let position = finite_vec3_or(light.position, Vec3::ZERO);
    let far_plane = sanitize_shadow_far_plane(light.range);
    ViewportCameraSnapshot {
        transform: Transform::looking_at(position, position + direction, up),
        projection_mode: ProjectionMode::Perspective,
        fov_y_radians: FRAC_PI_2,
        z_near: SHADOW_CAMERA_NEAR_PLANE,
        z_far: far_plane,
        aspect_ratio: 1.0,
        render_layers: RenderLayerSet::from_legacy_mask(u32::MAX),
        ..ViewportCameraSnapshot::default()
    }
}

fn spot_shadow_camera(light: &RenderSpotLightSnapshot) -> ViewportCameraSnapshot {
    let direction = sanitize_direction(light.direction);
    let position = finite_vec3_or(light.position, Vec3::ZERO);
    let far_plane = sanitize_shadow_far_plane(light.range);
    ViewportCameraSnapshot {
        transform: Transform::looking_at(
            position,
            position + direction,
            stable_shadow_up(direction),
        ),
        projection_mode: ProjectionMode::Perspective,
        fov_y_radians: sanitize_spot_fov(light.outer_angle_radians),
        z_near: SHADOW_CAMERA_NEAR_PLANE,
        z_far: far_plane,
        aspect_ratio: 1.0,
        render_layers: RenderLayerSet::from_legacy_mask(u32::MAX),
        ..ViewportCameraSnapshot::default()
    }
}

fn point_light_face_axes(face_index: u8) -> (Vec3, Vec3) {
    match face_index % POINT_LIGHT_SHADOW_FACE_COUNT {
        0 => (Vec3::X, Vec3::Y),
        1 => (-Vec3::X, Vec3::Y),
        2 => (Vec3::Y, Vec3::Z),
        3 => (-Vec3::Y, -Vec3::Z),
        4 => (Vec3::Z, Vec3::Y),
        _ => (-Vec3::Z, Vec3::Y),
    }
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

fn finite_vec3_or(value: Vec3, fallback: Vec3) -> Vec3 {
    if is_finite_vec3(value) {
        value
    } else {
        fallback
    }
}

fn sanitize_positive_distance(value: Real, fallback: Real) -> Real {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

fn sanitize_shadow_far_plane(value: Real) -> Real {
    sanitize_positive_distance(value, MIN_PUNCTUAL_SHADOW_RANGE).max(MIN_PUNCTUAL_SHADOW_RANGE)
}

fn sanitize_spot_fov(outer_angle_radians: Real) -> Real {
    (outer_angle_radians.max(0.001) * 2.0).clamp(0.001, PI - 0.001)
}

fn shadow_enabled(shadow: Option<LightShadowSettings>) -> bool {
    shadow.is_some_and(|settings| settings.casts_shadow)
}

fn directional_shadow_ranges(shadow: Option<LightShadowSettings>) -> Vec<CascadeRange> {
    let mut ranges =
        compute_cascade_ranges(&CascadeSplitConfig::default(), SHADOW_CAMERA_NEAR_PLANE);
    if shadow_enabled(shadow) {
        ranges
    } else {
        ranges.truncate(1);
        ranges
    }
}

fn default_shadow_light_direction() -> Vec3 {
    Vec3::from_array(DEFAULT_SHADOW_LIGHT_DIRECTION_COMPONENTS).normalize_or_zero()
}
