use crate::core::math::{Vec2, Vec3};

const PROJECTION_EPSILON: f32 = 1.0e-6;

/// 将世界位置投影为方向光局部 cookie UV，并应用 offset/scale；结果未经 wrap、裁剪或图集槽位变换。
pub(crate) fn directional_cookie_uv(
    world_position: Vec3,
    light_direction: Vec3,
    offset: Vec2,
    scale: Vec2,
) -> Vec2 {
    let (right, up, _) = light_basis(light_direction);
    Vec2::new(world_position.dot(right), world_position.dot(up)) * scale + offset
}

/// 返回未裁剪的聚光局部 UV；深度不超过阈值或投影半宽绝对值接近零时返回 None。
/// 本函数不检查输入有限性。
pub(crate) fn spot_cookie_uv(
    world_position: Vec3,
    light_position: Vec3,
    light_direction: Vec3,
    outer_angle_radians: f32,
) -> Option<Vec2> {
    let (right, up, forward) = light_basis(light_direction);
    let local = world_position - light_position;
    let depth = local.dot(forward);
    let half_extent = depth * outer_angle_radians.tan();
    if depth <= PROJECTION_EPSILON || half_extent.abs() <= PROJECTION_EPSILON {
        return None;
    }
    Some(Vec2::new(local.dot(right), local.dot(up)) / (2.0 * half_extent) + Vec2::splat(0.5))
}

/// 返回点光方向的八面体局部 UV；绝对分量和不超过阈值时取中心，负 z 半球折叠。
pub(crate) fn point_octahedral_cookie_uv(direction: Vec3) -> Vec2 {
    let denominator = direction.x.abs() + direction.y.abs() + direction.z.abs();
    if denominator <= PROJECTION_EPSILON {
        return Vec2::splat(0.5);
    }
    let normalized = direction / denominator;
    let mut folded = normalized.truncate();
    if normalized.z < 0.0 {
        folded = Vec2::new(
            (1.0 - normalized.y.abs()) * normalized.x.signum(),
            (1.0 - normalized.x.abs()) * normalized.y.signum(),
        );
    }
    folded * 0.5 + Vec2::splat(0.5)
}

fn light_basis(direction: Vec3) -> (Vec3, Vec3, Vec3) {
    let forward = direction.try_normalize().unwrap_or(Vec3::NEG_Z);
    let reference_up = if forward.dot(Vec3::Y).abs() < 0.999 {
        Vec3::Y
    } else {
        Vec3::X
    };
    let right = forward.cross(reference_up).normalize();
    let up = right.cross(forward).normalize();
    (right, up, forward)
}

#[cfg(test)]
#[path = "tests/projection.rs"]
mod tests;
