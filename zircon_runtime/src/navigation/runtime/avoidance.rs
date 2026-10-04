use crate::core::framework::navigation::{NavAvoidanceQuality, NavMeshAgentDescriptor};
use crate::core::math::{Real, Vec3};

use super::world_scan::RuntimeObstacle;

const MIN_AVOIDANCE_DISTANCE: Real = 0.001;

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
        let limit = obstacle.radius + agent.radius.max(0.05) + 0.5;
        if let Some(contribution) = avoidance_contribution(current, obstacle.center, limit) {
            avoidance += contribution;
        }
    }
    for (other_entity, other_position, other_radius) in agents {
        if *other_entity == entity {
            continue;
        }
        let limit = agent.radius.max(0.05) + *other_radius + 0.25;
        if let Some(contribution) = avoidance_contribution(current, *other_position, limit) {
            avoidance += contribution;
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

#[inline]
fn avoidance_contribution(current: Vec3, other: Vec3, limit: Real) -> Option<Vec3> {
    if !limit.is_finite() || limit <= MIN_AVOIDANCE_DISTANCE {
        return None;
    }
    let delta = Vec3::new(current.x - other.x, 0.0, current.z - other.z);
    let distance_squared = delta.x * delta.x + delta.z * delta.z;
    if !distance_squared.is_finite()
        || distance_squared <= MIN_AVOIDANCE_DISTANCE * MIN_AVOIDANCE_DISTANCE
        || distance_squared >= limit * limit
    {
        return None;
    }
    let distance = distance_squared.sqrt();
    Some((delta / distance) * (limit - distance))
}

#[cfg(test)]
#[path = "tests/avoidance.rs"]
mod tests;
