use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::framework::navigation::{
    NavAgentTickReport, NavMeshAgentDescriptor, NavMeshAsset, NavMeshBakeReport,
    NavMeshBakeRequest, NavMeshHandle, NavPathQuery, NavPathResult, NavPathStatus, NavQueryFilter,
    NavRaycastQuery, NavRaycastResult, NavSampleHit, NavSampleQuery, NavigationError,
    NavigationErrorKind, NavigationGeneratedBakeSnapshot, NavigationManager,
    NavigationRuntimeStats, NavigationSettingsAsset,
};
use crate::core::math::{Real, Transform, Vec3};
use crate::scene::{SceneNavigationRuntime, World};

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
use world_scan::NavigationWorldProjection;

#[derive(Debug)]
pub struct BuiltinNavigationManager {
    state: Mutex<BuiltinNavigationState>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AgentTickOutcome {
    Handled,
    RepathBudgetExhausted,
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

impl BuiltinNavigationManager {
    pub fn bake_surface(
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
        state
            .loaded
            .insert(handle, Arc::new(BakedNavMesh::new(asset)));
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
        let mesh = self.lock_state().selected_mesh_snapshot(query.nav_mesh)?;
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
        let mesh = self.lock_state().selected_mesh_snapshot(query.nav_mesh)?;
        Ok(mesh.sample_position(query))
    }

    fn raycast(&self, query: NavRaycastQuery) -> Result<NavRaycastResult, NavigationError> {
        let mesh = self.lock_state().selected_mesh_snapshot(query.nav_mesh)?;
        Ok(mesh.raycast(query))
    }

    pub fn tick_world_agents(
        &self,
        world: &mut World,
        dt_seconds: Real,
    ) -> Result<NavAgentTickReport, NavigationError> {
        if dt_seconds <= 0.0 || !dt_seconds.is_finite() {
            return Ok(NavAgentTickReport::default());
        }

        let mut projection = {
            let mut state = self.lock_state();
            let projection = state.take_navigation_projection(world);
            state.stats.active_agents = projection.agents.len();
            state.stats.active_obstacles = projection.obstacles.len();
            projection
        };
        let mut report = NavAgentTickReport {
            scanned_agents: projection.agents.len(),
            ..NavAgentTickReport::default()
        };

        let agent_count = projection.agents.len();
        let repath_start = self.lock_state().begin_repath_frame(agent_count);
        let mut next_repath = repath_start;
        projection.begin_avoidance_frame();
        for offset in 0..agent_count {
            let index = (repath_start + offset) % agent_count;
            let (entity, agent) = {
                let runtime_agent = &projection.agents[index];
                (runtime_agent.entity, runtime_agent.descriptor.clone())
            };
            match self.tick_agent(
                world,
                entity,
                &agent,
                &mut projection,
                dt_seconds,
                &mut report,
            ) {
                AgentTickOutcome::Handled => {
                    next_repath = (index + 1) % agent_count;
                }
                AgentTickOutcome::RepathBudgetExhausted => {
                    next_repath = index;
                    break;
                }
            }
        }
        self.lock_state()
            .set_repath_cursor(next_repath, agent_count);
        for index in 0..projection.agents.len() {
            let entity = projection.agents[index].entity;
            if let Some(transform) = world.world_transform(entity) {
                projection.update_agent_position(entity, transform.translation);
            }
        }
        // Navigation movement advances the world revision. Store the matching
        // post-tick projection so the next stable frame can reuse typed rows.
        projection.generation = world.world_generation();
        self.lock_state().store_navigation_projection(projection);
        Ok(report)
    }

    pub fn tick_world_agent(
        &self,
        world: &mut World,
        entity: u64,
        dt_seconds: Real,
    ) -> Result<NavAgentTickReport, NavigationError> {
        if dt_seconds <= 0.0 || !dt_seconds.is_finite() {
            return Ok(NavAgentTickReport::default());
        }

        let mut projection = {
            let mut state = self.lock_state();
            let projection = state.take_navigation_projection(world);
            state.stats.active_agents = projection.agents.len();
            state.stats.active_obstacles = projection.obstacles.len();
            projection
        };
        let Some(agent) = projection
            .agents
            .iter()
            .find(|candidate| candidate.entity == entity)
            .map(|candidate| candidate.descriptor.clone())
        else {
            self.lock_state().store_navigation_projection(projection);
            return Ok(NavAgentTickReport::default());
        };
        let mut report = NavAgentTickReport {
            scanned_agents: 1,
            ..NavAgentTickReport::default()
        };

        self.lock_state().begin_repath_frame(1);
        projection.begin_avoidance_frame();
        let _ = self.tick_agent(
            world,
            entity,
            &agent,
            &mut projection,
            dt_seconds,
            &mut report,
        );
        self.lock_state().set_repath_cursor(0, 1);
        if let Some(transform) = world.world_transform(entity) {
            projection.update_agent_position(entity, transform.translation);
        }
        // Keep the retained rows valid after a targeted navigation writeback.
        projection.generation = world.world_generation();
        self.lock_state().store_navigation_projection(projection);
        Ok(report)
    }

    fn stats(&self) -> NavigationRuntimeStats {
        self.lock_state().stats.clone()
    }
}

impl NavigationManager for BuiltinNavigationManager {
    fn load_nav_mesh(&self, asset: NavMeshAsset) -> Result<NavMeshHandle, NavigationError> {
        BuiltinNavigationManager::load_nav_mesh(self, asset)
    }

    fn load_navigation_settings(
        &self,
        settings: NavigationSettingsAsset,
    ) -> Result<(), NavigationError> {
        BuiltinNavigationManager::load_navigation_settings(self, settings)
    }

    fn find_path(&self, query: NavPathQuery) -> Result<NavPathResult, NavigationError> {
        BuiltinNavigationManager::find_path(self, query)
    }

    fn find_path_with_filter(
        &self,
        query: NavPathQuery,
        filter: &NavQueryFilter,
    ) -> Result<NavPathResult, NavigationError> {
        BuiltinNavigationManager::find_path_with_filter(self, query, filter)
    }

    fn sample_position(
        &self,
        query: NavSampleQuery,
    ) -> Result<Option<NavSampleHit>, NavigationError> {
        BuiltinNavigationManager::sample_position(self, query)
    }

    fn raycast(&self, query: NavRaycastQuery) -> Result<NavRaycastResult, NavigationError> {
        BuiltinNavigationManager::raycast(self, query)
    }

    fn stats(&self) -> NavigationRuntimeStats {
        BuiltinNavigationManager::stats(self)
    }
}

impl SceneNavigationRuntime for BuiltinNavigationManager {
    fn bake_surface(
        &self,
        world: &World,
        request: NavMeshBakeRequest,
    ) -> Result<NavMeshBakeReport, NavigationError> {
        BuiltinNavigationManager::bake_surface(self, world, request)
    }

    fn generated_bake_snapshot(
        &self,
        surface_entity: Option<u64>,
    ) -> NavigationGeneratedBakeSnapshot {
        self.lock_state().generated_snapshot(surface_entity)
    }

    fn replace_generated_bake_snapshot(
        &self,
        snapshot: NavigationGeneratedBakeSnapshot,
    ) -> Result<(), NavigationError> {
        self.lock_state().replace_generated_snapshot(snapshot);
        Ok(())
    }

    fn tick_world_agents(
        &self,
        world: &mut World,
        dt_seconds: Real,
    ) -> Result<NavAgentTickReport, NavigationError> {
        BuiltinNavigationManager::tick_world_agents(self, world, dt_seconds)
    }

    fn tick_world_agent(
        &self,
        world: &mut World,
        entity: u64,
        dt_seconds: Real,
    ) -> Result<NavAgentTickReport, NavigationError> {
        BuiltinNavigationManager::tick_world_agent(self, world, entity, dt_seconds)
    }
}

impl BuiltinNavigationManager {
    fn tick_agent(
        &self,
        world: &mut World,
        entity: u64,
        agent: &NavMeshAgentDescriptor,
        projection: &mut NavigationWorldProjection,
        dt_seconds: Real,
        report: &mut NavAgentTickReport,
    ) -> AgentTickOutcome {
        let Some(destination) = agent.destination else {
            self.lock_state().clear_repath_route(entity);
            return AgentTickOutcome::Handled;
        };
        if !agent.update_position {
            self.lock_state().clear_repath_route(entity);
            return AgentTickOutcome::Handled;
        }
        let Some(transform) = world.world_transform(entity) else {
            report.blocked_agents += 1;
            report
                .diagnostics
                .push(format!("agent {entity} has no world transform"));
            return AgentTickOutcome::Handled;
        };
        let current = transform.translation;
        let destination = Vec3::from_array(destination);
        let stopping_distance = agent.stopping_distance.max(0.0);
        if distance_xz(current, destination) <= stopping_distance {
            self.lock_state().clear_repath_route(entity);
            return AgentTickOutcome::Handled;
        }
        let path_target = if let Some(target) = self.lock_state().cached_repath_target(
            entity,
            current,
            destination,
            &agent.agent_type,
            agent.area_mask,
            stopping_distance,
        ) {
            target
        } else {
            let mesh = self.lock_state().selected_mesh_snapshot(None);
            match mesh {
                Ok(mesh) => {
                    if !self.lock_state().try_consume_repath_query() {
                        return AgentTickOutcome::RepathBudgetExhausted;
                    }
                    self.lock_state().record_repath_query();
                    match mesh.find_path(NavPathQuery {
                        nav_mesh: None,
                        start: current.to_array(),
                        end: destination.to_array(),
                        agent_type: agent.agent_type.clone(),
                        area_mask: agent.area_mask,
                    }) {
                        result if result.status != NavPathStatus::NoPath => {
                            let mut waypoints = result
                                .points
                                .iter()
                                .skip(1)
                                .map(|point| Vec3::from_array(point.position))
                                .collect::<Vec<_>>();
                            if waypoints.is_empty() {
                                waypoints.push(destination);
                            }
                            let target = waypoints[0];
                            self.lock_state().store_repath_route(
                                entity,
                                destination,
                                agent.agent_type.clone(),
                                agent.area_mask,
                                waypoints,
                            );
                            target
                        }
                        _ => {
                            self.lock_state().clear_repath_route(entity);
                            report.blocked_agents += 1;
                            report
                                .diagnostics
                                .push(format!("agent {entity} has no path on loaded navmesh"));
                            return AgentTickOutcome::Handled;
                        }
                    }
                }
                Err(_) => destination,
            }
        };
        let (obstacles, agent_positions) =
            projection.local_avoidance_rows(entity, current, agent.radius);
        let movement_target = avoidance_adjusted_target(
            entity,
            current,
            path_target,
            agent,
            obstacles,
            agent_positions,
        );
        let delta = movement_target - current;
        let distance = distance_xz(current, movement_target);
        if distance <= stopping_distance {
            return AgentTickOutcome::Handled;
        }
        let max_step = agent.speed.max(0.0) * dt_seconds;
        if max_step <= Real::EPSILON {
            return AgentTickOutcome::Handled;
        }
        let direction = Vec3::new(delta.x, 0.0, delta.z).normalize_or_zero();
        let mut next = current + direction * max_step.min(distance);
        if let Some(sampled) = {
            self.lock_state()
                .selected_mesh_snapshot(None)
                .ok()
                .and_then(|mesh| {
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
        AgentTickOutcome::Handled
    }
}
