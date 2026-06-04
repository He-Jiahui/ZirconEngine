use crate::component_json::parse_component;
use crate::off_mesh_connections::{count_off_mesh_bridges, count_off_mesh_links};
use crate::runtime_obstacles::{
    collect_runtime_obstacles, distance_xz, recast_carving_obstacles, RuntimeObstacle,
};
use zircon_runtime::core::framework::navigation::{
    NavAgentTickReport, NavMeshAgentDescriptor, NavPathQuery, NavPathStatus, NavigationError,
    NAV_MESH_AGENT_COMPONENT_TYPE,
};
use zircon_runtime::core::math::{Quat, Real, Transform, Vec3};
use zircon_runtime::scene::World;

use super::DefaultNavigationManager;

pub(super) fn tick_world_agents(
    manager: &DefaultNavigationManager,
    world: &mut World,
    dt_seconds: Real,
) -> Result<NavAgentTickReport, NavigationError> {
    let mut report = NavAgentTickReport::default();
    if dt_seconds <= 0.0 || !dt_seconds.is_finite() {
        return Ok(report);
    }

    let agents = collect_agents(world);
    report.scanned_agents = agents.len();
    let agent_positions = agent_positions(world, &agents);
    let obstacles = collect_runtime_obstacles(world);
    for (entity, agent) in agents {
        let Some(destination) = agent.destination else {
            continue;
        };
        if !agent.update_position {
            continue;
        }
        let Some(transform) = world.world_transform(entity) else {
            report.blocked_agents += 1;
            report
                .diagnostics
                .push(format!("agent {entity} has no world transform"));
            continue;
        };
        let current = transform.translation;
        let destination = Vec3::from_array(destination);
        let movement_target = match manager.selected_asset(None) {
            Ok(asset) => match manager.backend.find_path_with_obstacles(
                &asset,
                &NavPathQuery {
                    nav_mesh: None,
                    start: current.to_array(),
                    end: destination.to_array(),
                    agent_type: agent.agent_type.clone(),
                    area_mask: agent.area_mask,
                },
                &recast_carving_obstacles(&obstacles),
            ) {
                Ok(path) if path.status != NavPathStatus::NoPath => path
                    .points
                    .get(1)
                    .or_else(|| path.points.last())
                    .map(|point| Vec3::from_array(point.position))
                    .unwrap_or(destination),
                Ok(_) => {
                    report.blocked_agents += 1;
                    report
                        .diagnostics
                        .push(format!("agent {entity} has no path on loaded navmesh"));
                    continue;
                }
                Err(error) => {
                    report.blocked_agents += 1;
                    report
                        .diagnostics
                        .push(format!("agent {entity} path query failed: {error}"));
                    continue;
                }
            },
            Err(_) => destination,
        };
        let movement_target = avoidance_adjusted_target(
            entity,
            current,
            movement_target,
            &agent,
            &obstacles,
            &agent_positions,
        );
        let delta = movement_target - current;
        let distance = delta.length();
        if distance <= agent.stopping_distance {
            continue;
        }
        let step = (agent.speed.max(0.0) * dt_seconds).min(distance);
        let direction = delta.normalize_or_zero();
        let next = current + direction * step;
        let updated = Transform {
            translation: next,
            rotation: if agent.update_rotation && direction.length_squared() > Real::EPSILON {
                Quat::from_rotation_y(direction.x.atan2(-direction.z))
            } else {
                transform.rotation
            },
            ..transform
        };
        match world.update_transform(entity, updated) {
            Ok(true) => report.moved_agents += 1,
            Ok(false) => {}
            Err(error) => {
                report.blocked_agents += 1;
                report
                    .diagnostics
                    .push(format!("agent {entity} could not move: {error}"));
            }
        }
    }
    let mut state = manager
        .state
        .lock()
        .expect("navigation state lock poisoned");
    state.stats.active_agents = report.scanned_agents;
    state.stats.active_obstacles = obstacles.len();
    state.stats.active_off_mesh_links = count_off_mesh_links(world);
    state.stats.active_off_mesh_bridges = count_off_mesh_bridges(world);
    Ok(report)
}

fn collect_agents(world: &World) -> Vec<(u64, NavMeshAgentDescriptor)> {
    world
        .node_records()
        .into_iter()
        .filter_map(|node| {
            let value = world.dynamic_component(node.id, NAV_MESH_AGENT_COMPONENT_TYPE)?;
            Some((node.id, parse_component::<NavMeshAgentDescriptor>(value)))
        })
        .collect()
}

fn agent_positions(
    world: &World,
    agents: &[(u64, NavMeshAgentDescriptor)],
) -> Vec<(u64, Vec3, Real)> {
    agents
        .iter()
        .filter_map(|(entity, agent)| {
            world
                .world_transform(*entity)
                .map(|transform| (*entity, transform.translation, agent.radius.max(0.05)))
        })
        .collect()
}

fn avoidance_adjusted_target(
    entity: u64,
    current: Vec3,
    target: Vec3,
    agent: &NavMeshAgentDescriptor,
    obstacles: &[RuntimeObstacle],
    agents: &[(u64, Vec3, Real)],
) -> Vec3 {
    if matches!(
        agent.avoidance_quality,
        zircon_runtime::core::framework::navigation::NavAvoidanceQuality::None
    ) {
        return target;
    }
    let mut avoidance = Vec3::ZERO;
    for obstacle in obstacles
        .iter()
        .filter(|obstacle| obstacle.avoidance_enabled)
    {
        let away = current - obstacle.center;
        let distance = distance_xz(current, obstacle.center);
        let limit = obstacle.radius + agent.radius.max(0.05) + 0.5;
        if distance > 0.001 && distance < limit {
            avoidance += Vec3::new(away.x, 0.0, away.z).normalize_or_zero() * (limit - distance);
        }
    }
    for (other_entity, other_position, other_radius) in agents {
        if *other_entity == entity {
            continue;
        }
        let away = current - *other_position;
        let distance = distance_xz(current, *other_position);
        let limit = agent.radius.max(0.05) + *other_radius + 0.25;
        if distance > 0.001 && distance < limit {
            avoidance += Vec3::new(away.x, 0.0, away.z).normalize_or_zero() * (limit - distance);
        }
    }
    target + avoidance
}
