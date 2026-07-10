use zircon_runtime::core::framework::physics::PhysicsRayCastHit;
use zircon_runtime::core::math::{Real, Vec3};

use super::super::geometry::ray_hit_position;

pub(super) fn ray_cast_aabb(
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

    let position = ray_hit_position(origin, direction, distance)?;
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
