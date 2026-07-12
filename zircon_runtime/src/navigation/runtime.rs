use std::sync::{Mutex, MutexGuard};

use crate::asset::{NavMeshAsset, NavigationSettingsAsset};
use crate::core::framework::navigation::{
    NavAgentTickReport, NavMeshAgentDescriptor, NavMeshBakeReport, NavMeshBakeRequest,
    NavMeshHandle, NavPathQuery, NavPathResult, NavPathStatus, NavQueryFilter, NavRaycastQuery,
    NavRaycastResult, NavSampleHit, NavSampleQuery, NavigationError, NavigationErrorKind,
    NavigationManager, NavigationRuntimeStats,
};
use crate::core::math::{Real, Transform, Vec3};
use crate::scene::World;

mod avoidance;
mod baked_mesh;
mod math;
mod state;
#[cfg(test)]
mod tests;
mod world_scan;

use avoidance::avoidance_adjusted_target;
use baked_mesh::BakedNavMesh;
use math::{distance_xz, rotation_from_direction};
use state::BuiltinNavigationState;
use world_scan::{
    collect_agent, collect_agent_positions, collect_agents, collect_obstacles, RuntimeObstacle,
};

#[derive(Debug)]
pub struct BuiltinNavigationManager {
    state: Mutex<BuiltinNavigationState>,
}

impl BuiltinNavigationManager {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(BuiltinNavigationState::default()),
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, BuiltinNavigationState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for BuiltinNavigationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationManager for BuiltinNavigationManager {
    fn bake_surface(
        &self,
        _world: &World,
        _request: NavMeshBakeRequest,
    ) -> Result<NavMeshBakeReport, NavigationError> {
        Err(NavigationError::new(
            NavigationErrorKind::BackendFailure,
            "built-in navigation can load baked navmeshes but does not bake surfaces",
        ))
    }

    fn load_nav_mesh(&self, asset: NavMeshAsset) -> Result<NavMeshHandle, NavigationError> {
        if asset.is_empty() {
            return Err(NavigationError::missing_nav_mesh(
                "cannot load an empty navmesh asset",
            ));
        }
        let mut state = self.lock_state();
        let handle = NavMeshHandle(state.next_handle);
        state.next_handle += 1;
        state.loaded.insert(handle, BakedNavMesh::new(asset));
        state.stats.loaded_nav_meshes = state.loaded.len();
        Ok(handle)
    }

    fn load_navigation_settings(
        &self,
        settings: NavigationSettingsAsset,
    ) -> Result<(), NavigationError> {
        let mut state = self.lock_state();
        state.settings = settings;
        Ok(())
    }

    fn find_path(&self, query: NavPathQuery) -> Result<NavPathResult, NavigationError> {
        let state = self.lock_state();
        let mesh = state.selected_mesh(query.nav_mesh)?;
        Ok(mesh.find_path(query))
    }

    fn find_path_with_filter(
        &self,
        _query: NavPathQuery,
        _filter: &NavQueryFilter,
    ) -> Result<NavPathResult, NavigationError> {
        Err(NavigationError::new(
            NavigationErrorKind::BackendFailure,
            "built-in navigation does not support per-query filters; activate the navigation plugin",
        ))
    }

    fn sample_position(
        &self,
        query: NavSampleQuery,
    ) -> Result<Option<NavSampleHit>, NavigationError> {
        let state = self.lock_state();
        let mesh = state.selected_mesh(query.nav_mesh)?;
        Ok(mesh.sample_position(query))
    }

    fn raycast(&self, query: NavRaycastQuery) -> Result<NavRaycastResult, NavigationError> {
        let state = self.lock_state();
        let mesh = state.selected_mesh(query.nav_mesh)?;
        Ok(mesh.raycast(query))
    }

    fn tick_world_agents(
        &self,
        world: &mut World,
        dt_seconds: Real,
    ) -> Result<NavAgentTickReport, NavigationError> {
        if dt_seconds <= 0.0 || !dt_seconds.is_finite() {
            return Ok(NavAgentTickReport::default());
        }

        let agents = collect_agents(world);
        let agent_positions = collect_agent_positions(world, &agents);
        let obstacles = collect_obstacles(world);
        let mut report = NavAgentTickReport {
            scanned_agents: agents.len(),
            ..NavAgentTickReport::default()
        };
        {
            let mut state = self.lock_state();
            state.stats.active_agents = agents.len();
            state.stats.active_obstacles = obstacles.len();
        }

        for (entity, agent) in agents {
            self.tick_agent(
                world,
                entity,
                agent,
                &obstacles,
                &agent_positions,
                dt_seconds,
                &mut report,
            );
        }
        Ok(report)
    }

    fn tick_world_agent(
        &self,
        world: &mut World,
        entity: u64,
        dt_seconds: Real,
    ) -> Result<NavAgentTickReport, NavigationError> {
        if dt_seconds <= 0.0 || !dt_seconds.is_finite() {
            return Ok(NavAgentTickReport::default());
        }

        let Some(agent) = collect_agent(world, entity) else {
            return Ok(NavAgentTickReport::default());
        };
        let agents = collect_agents(world);
        let agent_positions = collect_agent_positions(world, &agents);
        let obstacles = collect_obstacles(world);
        let mut report = NavAgentTickReport {
            scanned_agents: 1,
            ..NavAgentTickReport::default()
        };
        {
            let mut state = self.lock_state();
            state.stats.active_agents = agents.len();
            state.stats.active_obstacles = obstacles.len();
        }

        self.tick_agent(
            world,
            entity,
            agent,
            &obstacles,
            &agent_positions,
            dt_seconds,
            &mut report,
        );
        Ok(report)
    }

    fn stats(&self) -> NavigationRuntimeStats {
        self.lock_state().stats.clone()
    }
}

impl BuiltinNavigationManager {
    fn tick_agent(
        &self,
        world: &mut World,
        entity: u64,
        agent: NavMeshAgentDescriptor,
        obstacles: &[RuntimeObstacle],
        agent_positions: &[(u64, Vec3, Real)],
        dt_seconds: Real,
        report: &mut NavAgentTickReport,
    ) {
        let Some(destination) = agent.destination else {
            return;
        };
        if !agent.update_position {
            return;
        }
        let Some(transform) = world.world_transform(entity) else {
            report.blocked_agents += 1;
            report
                .diagnostics
                .push(format!("agent {entity} has no world transform"));
            return;
        };
        let current = transform.translation;
        let destination = Vec3::from_array(destination);
        let path_target = {
            let state = self.lock_state();
            match state.selected_mesh(None) {
                Ok(mesh) => match mesh.find_path(NavPathQuery {
                    nav_mesh: None,
                    start: current.to_array(),
                    end: destination.to_array(),
                    agent_type: agent.agent_type.clone(),
                    area_mask: agent.area_mask,
                }) {
                    result if result.status != NavPathStatus::NoPath => result
                        .points
                        .get(1)
                        .or_else(|| result.points.last())
                        .map(|point| Vec3::from_array(point.position))
                        .unwrap_or(destination),
                    _ => {
                        report.blocked_agents += 1;
                        report
                            .diagnostics
                            .push(format!("agent {entity} has no path on loaded navmesh"));
                        return;
                    }
                },
                Err(_) => destination,
            }
        };
        let movement_target = avoidance_adjusted_target(
            entity,
            current,
            path_target,
            &agent,
            obstacles,
            agent_positions,
        );
        let delta = movement_target - current;
        let distance = distance_xz(current, movement_target);
        if distance <= agent.stopping_distance.max(0.0) {
            return;
        }
        let max_step = agent.speed.max(0.0) * dt_seconds;
        if max_step <= Real::EPSILON {
            return;
        }
        let direction = Vec3::new(delta.x, 0.0, delta.z).normalize_or_zero();
        let mut next = current + direction * max_step.min(distance);
        if let Some(sampled) = {
            let state = self.lock_state();
            state.selected_mesh(None).ok().and_then(|mesh| {
                mesh.sample_position(NavSampleQuery {
                    nav_mesh: None,
                    position: next.to_array(),
                    extents: [
                        agent.radius.max(0.25),
                        agent.height.max(0.5),
                        agent.radius.max(0.25),
                    ],
                    agent_type: agent.agent_type.clone(),
                    area_mask: agent.area_mask,
                })
            })
        } {
            next = Vec3::from_array(sampled.position);
        }
        let rotation = if agent.update_rotation && direction.length_squared() > Real::EPSILON {
            rotation_from_direction(direction)
        } else {
            transform.rotation
        };
        let updated = Transform {
            translation: next,
            rotation,
            ..transform
        };
        match world.update_transform(entity, updated) {
            Ok(true) | Ok(false) => {
                report.moved_agents += 1;
            }
            Err(error) => {
                report.blocked_agents += 1;
                report
                    .diagnostics
                    .push(format!("agent {entity} could not move: {error}"));
            }
        }
    }
}
