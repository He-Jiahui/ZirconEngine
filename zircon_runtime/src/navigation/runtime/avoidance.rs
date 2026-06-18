use crate::core::framework::navigation::{NavAvoidanceQuality, NavMeshAgentDescriptor};
use crate::core::math::{Real, Vec3};

use super::math::distance_xz;
use super::world_scan::RuntimeObstacle;

pub(super) fn avoidance_adjusted_target(
    entity: u64,
    current: Vec3,
    target: Vec3,
    agent: &NavMeshAgentDescriptor,
    obstacles: &[RuntimeObstacle],
    agents: &[(u64, Vec3, Real)],
) -> Vec3 {
    if matches!(agent.avoidance_quality, NavAvoidanceQuality::None) {
        return target;
    }
    let desired_delta = Vec3::new(target.x - current.x, 0.0, target.z - current.z);
    let desired_distance = desired_delta.length();
    if desired_distance <= Real::EPSILON {
        return target;
    }
    let mut avoidance = Vec3::ZERO;
    for obstacle in obstacles
        .iter()
        .filter(|obstacle| obstacle.avoidance_enabled && obstacle.entity != entity)
    {
        let distance = distance_xz(current, obstacle.center);
        let limit = obstacle.radius + agent.radius.max(0.05) + 0.5;
        if distance > 0.001 && distance < limit {
            let away = Vec3::new(
                current.x - obstacle.center.x,
                0.0,
                current.z - obstacle.center.z,
            )
            .normalize_or_zero();
            avoidance += away * (limit - distance);
        }
    }
    for (other_entity, other_position, other_radius) in agents {
        if *other_entity == entity {
            continue;
        }
        let distance = distance_xz(current, *other_position);
        let limit = agent.radius.max(0.05) + *other_radius + 0.25;
        if distance > 0.001 && distance < limit {
            let away = Vec3::new(
                current.x - other_position.x,
                0.0,
                current.z - other_position.z,
            )
            .normalize_or_zero();
            avoidance += away * (limit - distance);
        }
    }
    if avoidance.length_squared() <= Real::EPSILON {
        return target;
    }
    let direction = avoidance.normalize_or_zero();
    if direction.length_squared() <= Real::EPSILON {
        current
    } else {
        current + direction * desired_distance
    }
}
