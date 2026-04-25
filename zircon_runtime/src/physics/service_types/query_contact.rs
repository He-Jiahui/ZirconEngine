use crate::core::framework::physics::{
    PhysicsColliderShape, PhysicsColliderSyncState, PhysicsContactEvent, PhysicsRayCastHit,
    PhysicsRayCastQuery, PhysicsSettings, PhysicsWorldSyncState,
};
use crate::core::math::{Real, Vec3};

pub(super) fn compute_contact_events(
    sync: &PhysicsWorldSyncState,
    settings: &PhysicsSettings,
) -> Vec<PhysicsContactEvent> {
    let mut contacts = Vec::new();
    for left_index in 0..sync.colliders.len() {
        for right_index in left_index + 1..sync.colliders.len() {
            let left = &sync.colliders[left_index];
            let right = &sync.colliders[right_index];
            if !colliders_can_contact(left, right, settings) {
                continue;
            }
            if !colliders_overlap(left, right) {
                continue;
            }

            let left_center = left.transform.translation;
            let right_center = right.transform.translation;
            let mut normal = (right_center - left_center).normalize_or_zero();
            if normal.length_squared() <= Real::EPSILON {
                normal = Vec3::Y;
            }
            let point = midpoint(left_center, right_center);
            contacts.push(PhysicsContactEvent {
                world: sync.world,
                entity: left.entity,
                other_entity: right.entity,
                point: point.to_array(),
                normal: normal.to_array(),
            });
        }
    }
    contacts
}

pub(super) fn collider_matches_query(
    query: &PhysicsRayCastQuery,
    collider: &PhysicsColliderSyncState,
) -> bool {
    (query.include_sensors || !collider.sensor)
        && query
            .collision_mask
            .is_none_or(|mask| collider_matches_query_mask(collider, mask))
}

pub(super) fn ray_cast_collider(
    origin: Vec3,
    direction: Vec3,
    max_distance: Real,
    collider: &PhysicsColliderSyncState,
) -> Option<PhysicsRayCastHit> {
    if !collider_geometry_is_valid(collider) {
        return None;
    }

    match collider.shape {
        PhysicsColliderShape::Box { half_extents } => {
            let scaled_half_extents =
                scaled_box_half_extents(half_extents, collider.transform.scale)?;
            let center = collider.transform.translation;
            let min = center - scaled_half_extents;
            let max = center + scaled_half_extents;
            ray_cast_aabb(origin, direction, max_distance, collider.entity, min, max)
        }
        PhysicsColliderShape::Sphere { radius } => {
            let scaled_radius = radius * max_abs_scale(collider.transform.scale);
            if !positive_finite(scaled_radius) {
                return None;
            }
            ray_cast_sphere(
                origin,
                direction,
                max_distance,
                collider.entity,
                collider.transform.translation,
                scaled_radius,
            )
        }
        PhysicsColliderShape::Capsule {
            radius,
            half_height,
        } => {
            let scale = collider.transform.scale.abs();
            let scaled_radius = radius * scale.x.max(scale.z);
            let scaled_half_height = half_height * scale.y;
            if !positive_finite(scaled_radius) || !scaled_half_height.is_finite() {
                return None;
            }
            ray_cast_capsule_y(
                origin,
                direction,
                max_distance,
                collider.entity,
                collider.transform.translation,
                scaled_radius,
                scaled_half_height,
            )
        }
    }
}

fn colliders_can_contact(
    left: &PhysicsColliderSyncState,
    right: &PhysicsColliderSyncState,
    settings: &PhysicsSettings,
) -> bool {
    if left.sensor || right.sensor {
        return false;
    }

    let Some(left_layer) = collision_layer_bit(left.layer) else {
        return false;
    };
    let Some(right_layer) = collision_layer_bit(right.layer) else {
        return false;
    };

    left.collision_mask & right_layer != 0
        && right.collision_mask & left_layer != 0
        && collision_matrix_allows(settings, left.layer, right.layer)
        && collision_matrix_allows(settings, right.layer, left.layer)
}

fn collider_matches_query_mask(collider: &PhysicsColliderSyncState, query_mask: u32) -> bool {
    collision_layer_bit(collider.layer).is_some_and(|layer_bit| query_mask & layer_bit != 0)
}

fn collision_matrix_allows(
    settings: &PhysicsSettings,
    source_layer: u32,
    target_layer: u32,
) -> bool {
    let Some(row) = settings.collision_matrix.get(source_layer as usize) else {
        return false;
    };
    let Some(target_bit) = 1_u64.checked_shl(target_layer) else {
        return false;
    };
    row & target_bit != 0
}

fn collision_layer_bit(layer: u32) -> Option<u32> {
    1_u32.checked_shl(layer)
}

fn colliders_are_boxes(left: &PhysicsColliderSyncState, right: &PhysicsColliderSyncState) -> bool {
    matches!(left.shape, PhysicsColliderShape::Box { .. })
        && matches!(right.shape, PhysicsColliderShape::Box { .. })
}

fn colliders_overlap(left: &PhysicsColliderSyncState, right: &PhysicsColliderSyncState) -> bool {
    if !collider_geometry_is_valid(left) || !collider_geometry_is_valid(right) {
        return false;
    }

    if let (Some(left_sphere), Some(right_sphere)) = (collider_sphere(left), collider_sphere(right))
    {
        return sphere_sphere_overlap(left_sphere, right_sphere);
    }
    if let (Some(sphere), Some(box_proxy)) = (collider_sphere(left), collider_box(right)) {
        return sphere_box_overlap(sphere, box_proxy);
    }
    if let (Some(box_proxy), Some(sphere)) = (collider_box(left), collider_sphere(right)) {
        return sphere_box_overlap(sphere, box_proxy);
    }
    if let (Some(box_proxy), Some(capsule)) = (collider_box(left), collider_capsule_y(right)) {
        return capsule_box_overlap(capsule, box_proxy);
    }
    if let (Some(capsule), Some(box_proxy)) = (collider_capsule_y(left), collider_box(right)) {
        return capsule_box_overlap(capsule, box_proxy);
    }
    if let (Some(sphere), Some(capsule)) = (collider_sphere(left), collider_capsule_y(right)) {
        return sphere_capsule_overlap(sphere, capsule);
    }
    if let (Some(capsule), Some(sphere)) = (collider_capsule_y(left), collider_sphere(right)) {
        return sphere_capsule_overlap(sphere, capsule);
    }
    if let (Some(left_capsule), Some(right_capsule)) =
        (collider_capsule_y(left), collider_capsule_y(right))
    {
        return capsule_capsule_overlap(left_capsule, right_capsule);
    }
    if colliders_are_boxes(left, right) {
        return match (collider_box(left), collider_box(right)) {
            (Some(left_box), Some(right_box)) => box_box_overlap(left_box, right_box),
            _ => false,
        };
    }

    let Some((left_min, left_max)) = collider_aabb(left) else {
        return false;
    };
    let Some((right_min, right_max)) = collider_aabb(right) else {
        return false;
    };
    left_min.x <= right_max.x
        && left_max.x >= right_min.x
        && left_min.y <= right_max.y
        && left_max.y >= right_min.y
        && left_min.z <= right_max.z
        && left_max.z >= right_min.z
}

fn collider_geometry_is_valid(collider: &PhysicsColliderSyncState) -> bool {
    if !vec3_is_finite(collider.transform.translation) || !vec3_is_finite(collider.transform.scale)
    {
        return false;
    }

    match collider.shape {
        PhysicsColliderShape::Box { half_extents } => box_geometry_is_valid(half_extents),
        PhysicsColliderShape::Sphere { radius } => positive_finite(radius),
        PhysicsColliderShape::Capsule {
            radius,
            half_height,
        } => capsule_geometry_is_valid(radius, half_height),
    }
}

fn vec3_is_finite(value: Vec3) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.z.is_finite()
}

fn midpoint(left: Vec3, right: Vec3) -> Vec3 {
    left + (right - left) * 0.5
}

#[derive(Clone, Copy)]
struct SphereOverlapProxy {
    center: Vec3,
    radius: Real,
}

#[derive(Clone, Copy)]
struct BoxOverlapProxy {
    min: Vec3,
    max: Vec3,
}

#[derive(Clone, Copy)]
struct CapsuleOverlapProxy {
    center: Vec3,
    radius: Real,
    half_height: Real,
}

fn collider_sphere(collider: &PhysicsColliderSyncState) -> Option<SphereOverlapProxy> {
    let PhysicsColliderShape::Sphere { radius } = collider.shape else {
        return None;
    };
    if !positive_finite(radius) {
        return None;
    }
    let scaled_radius = radius * max_abs_scale(collider.transform.scale);
    if !positive_finite(scaled_radius) {
        return None;
    }
    Some(SphereOverlapProxy {
        center: collider.transform.translation,
        radius: scaled_radius,
    })
}

fn collider_box(collider: &PhysicsColliderSyncState) -> Option<BoxOverlapProxy> {
    let PhysicsColliderShape::Box { half_extents } = collider.shape else {
        return None;
    };
    if !box_geometry_is_valid(half_extents) {
        return None;
    }
    let center = collider.transform.translation;
    let scaled_half_extents = scaled_box_half_extents(half_extents, collider.transform.scale)?;
    Some(BoxOverlapProxy {
        min: center - scaled_half_extents,
        max: center + scaled_half_extents,
    })
}

fn collider_capsule_y(collider: &PhysicsColliderSyncState) -> Option<CapsuleOverlapProxy> {
    let PhysicsColliderShape::Capsule {
        radius,
        half_height,
    } = collider.shape
    else {
        return None;
    };
    if !capsule_geometry_is_valid(radius, half_height) {
        return None;
    }
    let scale = collider.transform.scale.abs();
    let scaled_radius = radius * scale.x.max(scale.z);
    let scaled_half_height = half_height * scale.y;
    if !positive_finite(scaled_radius) || !scaled_half_height.is_finite() {
        return None;
    }
    Some(CapsuleOverlapProxy {
        center: collider.transform.translation,
        radius: scaled_radius,
        half_height: scaled_half_height,
    })
}

fn sphere_sphere_overlap(left: SphereOverlapProxy, right: SphereOverlapProxy) -> bool {
    left.center.distance_squared(right.center) <= (left.radius + right.radius).powi(2)
}

fn sphere_box_overlap(sphere: SphereOverlapProxy, box_proxy: BoxOverlapProxy) -> bool {
    let closest = sphere.center.clamp(box_proxy.min, box_proxy.max);
    sphere.center.distance_squared(closest) <= sphere.radius * sphere.radius
}

fn sphere_capsule_overlap(sphere: SphereOverlapProxy, capsule: CapsuleOverlapProxy) -> bool {
    let closest = closest_point_on_capsule_segment_y(sphere.center, capsule);
    sphere.center.distance_squared(closest) <= (sphere.radius + capsule.radius).powi(2)
}

fn capsule_box_overlap(capsule: CapsuleOverlapProxy, box_proxy: BoxOverlapProxy) -> bool {
    capsule_segment_aabb_distance_squared_y(capsule, box_proxy) <= capsule.radius * capsule.radius
}

fn capsule_capsule_overlap(left: CapsuleOverlapProxy, right: CapsuleOverlapProxy) -> bool {
    segment_segment_distance_squared_y(left, right) <= (left.radius + right.radius).powi(2)
}

fn box_box_overlap(left: BoxOverlapProxy, right: BoxOverlapProxy) -> bool {
    left.min.x <= right.max.x
        && left.max.x >= right.min.x
        && left.min.y <= right.max.y
        && left.max.y >= right.min.y
        && left.min.z <= right.max.z
        && left.max.z >= right.min.z
}

fn closest_point_on_capsule_segment_y(point: Vec3, capsule: CapsuleOverlapProxy) -> Vec3 {
    let min_y = capsule.center.y - capsule.half_height;
    let max_y = capsule.center.y + capsule.half_height;
    Vec3::new(
        capsule.center.x,
        point.y.clamp(min_y, max_y),
        capsule.center.z,
    )
}

fn segment_segment_distance_squared_y(
    left: CapsuleOverlapProxy,
    right: CapsuleOverlapProxy,
) -> Real {
    let left_min = left.center.y - left.half_height;
    let left_max = left.center.y + left.half_height;
    let right_min = right.center.y - right.half_height;
    let right_max = right.center.y + right.half_height;
    let y_gap = if left_max < right_min {
        right_min - left_max
    } else if right_max < left_min {
        left_min - right_max
    } else {
        0.0
    };
    let xz_gap = Vec3::new(
        left.center.x - right.center.x,
        0.0,
        left.center.z - right.center.z,
    );
    xz_gap.length_squared() + y_gap * y_gap
}

fn capsule_segment_aabb_distance_squared_y(
    capsule: CapsuleOverlapProxy,
    box_proxy: BoxOverlapProxy,
) -> Real {
    let segment_min_y = capsule.center.y - capsule.half_height;
    let segment_max_y = capsule.center.y + capsule.half_height;
    let x_gap = point_interval_gap(capsule.center.x, box_proxy.min.x, box_proxy.max.x);
    let y_gap = interval_interval_gap(
        segment_min_y,
        segment_max_y,
        box_proxy.min.y,
        box_proxy.max.y,
    );
    let z_gap = point_interval_gap(capsule.center.z, box_proxy.min.z, box_proxy.max.z);
    x_gap * x_gap + y_gap * y_gap + z_gap * z_gap
}

fn point_interval_gap(point: Real, min: Real, max: Real) -> Real {
    if point < min {
        min - point
    } else if point > max {
        point - max
    } else {
        0.0
    }
}

fn interval_interval_gap(left_min: Real, left_max: Real, right_min: Real, right_max: Real) -> Real {
    if left_max < right_min {
        right_min - left_max
    } else if right_max < left_min {
        left_min - right_max
    } else {
        0.0
    }
}

fn collider_aabb(collider: &PhysicsColliderSyncState) -> Option<(Vec3, Vec3)> {
    let center = collider.transform.translation;
    let scale = collider.transform.scale.abs();
    let half_extents = match collider.shape {
        PhysicsColliderShape::Box { half_extents } => scaled_box_half_extents(half_extents, scale)?,
        PhysicsColliderShape::Sphere { radius } => {
            let scaled_radius = radius * max_abs_scale(collider.transform.scale);
            if !positive_finite(scaled_radius) {
                return None;
            }
            Vec3::splat(scaled_radius)
        }
        PhysicsColliderShape::Capsule {
            radius,
            half_height,
        } => {
            let scaled_radius_x = radius * scale.x;
            let scaled_radius_z = radius * scale.z;
            let scaled_half_height = (radius + half_height) * scale.y;
            let half_extents = Vec3::new(scaled_radius_x, scaled_half_height, scaled_radius_z);
            if !vec3_is_finite(half_extents) {
                return None;
            }
            half_extents
        }
    };
    let min = center - half_extents;
    let max = center + half_extents;
    if !vec3_is_finite(min) || !vec3_is_finite(max) {
        return None;
    }
    Some((min, max))
}

fn max_abs_scale(scale: Vec3) -> Real {
    scale.x.abs().max(scale.y.abs()).max(scale.z.abs())
}

fn positive_finite(value: Real) -> bool {
    value.is_finite() && value > 0.0
}

fn capsule_geometry_is_valid(radius: Real, half_height: Real) -> bool {
    positive_finite(radius) && half_height.is_finite() && half_height >= 0.0
}

fn box_geometry_is_valid(half_extents: [Real; 3]) -> bool {
    half_extents
        .iter()
        .all(|extent| extent.is_finite() && *extent >= 0.0)
}

fn scaled_box_half_extents(half_extents: [Real; 3], scale: Vec3) -> Option<Vec3> {
    let scaled_half_extents = Vec3::from_array(half_extents) * scale.abs();
    vec3_is_finite(scaled_half_extents).then_some(scaled_half_extents)
}

fn ray_cast_aabb(
    origin: Vec3,
    direction: Vec3,
    max_distance: Real,
    entity: u64,
    min: Vec3,
    max: Vec3,
) -> Option<PhysicsRayCastHit> {
    let mut t_min = 0.0;
    let mut t_max = max_distance;
    let mut normal = Vec3::ZERO;
    let mut exit_normal = Vec3::ZERO;
    let origin_inside = (0..3).all(|axis| origin[axis] > min[axis] && origin[axis] < max[axis]);

    for axis in 0..3 {
        let origin_axis = origin[axis];
        let direction_axis = direction[axis];
        if direction_axis.abs() <= Real::EPSILON {
            if origin_axis < min[axis] || origin_axis > max[axis] {
                return None;
            }
            continue;
        }

        let inv_dir = 1.0 / direction_axis;
        let mut near = (min[axis] - origin_axis) * inv_dir;
        let mut far = (max[axis] - origin_axis) * inv_dir;
        let mut axis_normal = match axis {
            0 => -Vec3::X,
            1 => -Vec3::Y,
            _ => -Vec3::Z,
        };
        let mut far_axis_normal = -axis_normal;
        if near > far {
            std::mem::swap(&mut near, &mut far);
            axis_normal = -axis_normal;
            far_axis_normal = -far_axis_normal;
        }
        if near > t_min {
            t_min = near;
            normal = axis_normal;
        }
        if far <= t_max {
            exit_normal = far_axis_normal;
        }
        t_max = t_max.min(far);
        if t_min > t_max {
            return None;
        }
    }

    let (distance, mut normal) = if origin_inside {
        (t_max, exit_normal)
    } else {
        (t_min, normal)
    };

    if distance < 0.0 || distance > max_distance {
        return None;
    }
    if distance <= Real::EPSILON && normal.length_squared() <= Real::EPSILON {
        normal = aabb_surface_normal(origin, direction, min, max).unwrap_or(normal);
    }

    let position = origin + direction * distance;
    Some(PhysicsRayCastHit {
        entity,
        distance,
        position: position.to_array(),
        normal: normal.to_array(),
    })
}

fn aabb_surface_normal(origin: Vec3, direction: Vec3, min: Vec3, max: Vec3) -> Option<Vec3> {
    let mut best = None;
    let mut best_direction = 0.0;
    for axis in 0..3 {
        let direction_axis = direction[axis];
        let candidate = if (origin[axis] - min[axis]).abs() <= Real::EPSILON && direction_axis < 0.0
        {
            Some((axis, -1.0))
        } else if (origin[axis] - max[axis]).abs() <= Real::EPSILON && direction_axis > 0.0 {
            Some((axis, 1.0))
        } else {
            None
        };

        let Some((candidate_axis, sign)) = candidate else {
            continue;
        };
        let abs_direction = direction_axis.abs();
        if abs_direction > best_direction {
            best_direction = abs_direction;
            best = Some((candidate_axis, sign));
        }
    }

    best.map(|(axis, sign)| match axis {
        0 => Vec3::X * sign,
        1 => Vec3::Y * sign,
        _ => Vec3::Z * sign,
    })
}

fn ray_cast_sphere(
    origin: Vec3,
    direction: Vec3,
    max_distance: Real,
    entity: u64,
    center: Vec3,
    radius: Real,
) -> Option<PhysicsRayCastHit> {
    let offset = origin - center;
    let a = direction.length_squared();
    let b = 2.0 * offset.dot(direction);
    let c = offset.length_squared() - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let near_distance = (-b - sqrt_discriminant) / (2.0 * a);
    let far_distance = (-b + sqrt_discriminant) / (2.0 * a);
    let distance = if near_distance >= 0.0 {
        near_distance
    } else {
        far_distance
    };
    if !(0.0..=max_distance).contains(&distance) {
        return None;
    }

    let position = origin + direction * distance;
    let normal = (position - center).normalize_or_zero();
    Some(PhysicsRayCastHit {
        entity,
        distance,
        position: position.to_array(),
        normal: normal.to_array(),
    })
}

fn ray_cast_capsule_cap_y(
    origin: Vec3,
    direction: Vec3,
    max_distance: Real,
    entity: u64,
    center: Vec3,
    radius: Real,
    boundary_y: Real,
    upper: bool,
) -> Option<PhysicsRayCastHit> {
    let offset = origin - center;
    let a = direction.length_squared();
    let b = 2.0 * offset.dot(direction);
    let c = offset.length_squared() - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let sqrt_discriminant = discriminant.sqrt();
    [
        (-b - sqrt_discriminant) / (2.0 * a),
        (-b + sqrt_discriminant) / (2.0 * a),
    ]
    .into_iter()
    .filter(|distance| (0.0..=max_distance).contains(distance))
    .filter_map(|distance| {
        let position = origin + direction * distance;
        let on_visible_cap = if upper {
            position.y >= boundary_y
        } else {
            position.y <= boundary_y
        };
        if !on_visible_cap {
            return None;
        }
        let normal = (position - center).normalize_or_zero();
        Some(PhysicsRayCastHit {
            entity,
            distance,
            position: position.to_array(),
            normal: normal.to_array(),
        })
    })
    .min_by(|left, right| {
        left.distance
            .partial_cmp(&right.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

fn ray_cast_capsule_y(
    origin: Vec3,
    direction: Vec3,
    max_distance: Real,
    entity: u64,
    center: Vec3,
    radius: Real,
    half_height: Real,
) -> Option<PhysicsRayCastHit> {
    if radius <= 0.0 {
        return None;
    }

    [
        ray_cast_capsule_cylinder_y(
            origin,
            direction,
            max_distance,
            entity,
            center,
            radius,
            half_height,
        ),
        ray_cast_capsule_cap_y(
            origin,
            direction,
            max_distance,
            entity,
            center + Vec3::Y * half_height,
            radius,
            center.y + half_height,
            true,
        ),
        ray_cast_capsule_cap_y(
            origin,
            direction,
            max_distance,
            entity,
            center - Vec3::Y * half_height,
            radius,
            center.y - half_height,
            false,
        ),
    ]
    .into_iter()
    .flatten()
    .min_by(|left, right| {
        left.distance
            .partial_cmp(&right.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

fn ray_cast_capsule_cylinder_y(
    origin: Vec3,
    direction: Vec3,
    max_distance: Real,
    entity: u64,
    center: Vec3,
    radius: Real,
    half_height: Real,
) -> Option<PhysicsRayCastHit> {
    let offset = origin - center;
    let a = direction.x * direction.x + direction.z * direction.z;
    if a <= Real::EPSILON {
        return None;
    }

    let b = 2.0 * (offset.x * direction.x + offset.z * direction.z);
    let c = offset.x * offset.x + offset.z * offset.z - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let near = (-b - sqrt_discriminant) / (2.0 * a);
    let far = (-b + sqrt_discriminant) / (2.0 * a);
    [near, far]
        .into_iter()
        .filter(|distance| (0.0..=max_distance).contains(distance))
        .filter_map(|distance| {
            let position = origin + direction * distance;
            let local_y = position.y - center.y;
            if local_y.abs() > half_height {
                return None;
            }
            let normal =
                Vec3::new(position.x - center.x, 0.0, position.z - center.z).normalize_or_zero();
            Some(PhysicsRayCastHit {
                entity,
                distance,
                position: position.to_array(),
                normal: normal.to_array(),
            })
        })
        .min_by(|left, right| {
            left.distance
                .partial_cmp(&right.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}
