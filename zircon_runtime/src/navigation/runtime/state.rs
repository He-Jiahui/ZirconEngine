use std::collections::HashMap;
use std::sync::Arc;

use crate::core::framework::navigation::{
    NavMeshHandle, NavigationError, NavigationGeneratedBakeSnapshot, NavigationRuntimeStats,
    NavigationSettingsAsset,
};
use crate::core::math::{Real, Vec3};
use crate::navigation::NavRepathBudget;
use crate::scene::World;

use super::baked_mesh::BakedNavMesh;
use super::math::distance_xz;
use super::world_scan::{collect_navigation_world_projection, NavigationWorldProjection};

#[derive(Debug)]
pub(super) struct BuiltinNavigationState {
    pub(super) next_handle: u64,
    pub(super) loaded: HashMap<NavMeshHandle, Arc<BakedNavMesh>>,
    pub(super) generated: HashMap<Option<u64>, BuiltinGeneratedBakeState>,
    pub(super) settings: NavigationSettingsAsset,
    pub(super) stats: NavigationRuntimeStats,
    pub(super) navigation_projection: Option<NavigationWorldProjection>,
    pub(super) navigation_projection_builds: u64,
    pub(super) navigation_projection_component_rows: u64,
    pub(super) repath_budget: NavRepathBudget,
    repath_cursor: usize,
    repath_routes: HashMap<u64, RuntimeRepathRoute>,
    pub(super) repath_queries: u64,
}

impl Default for BuiltinNavigationState {
    fn default() -> Self {
        Self {
            next_handle: 1,
            loaded: HashMap::new(),
            generated: HashMap::new(),
            settings: NavigationSettingsAsset::default(),
            stats: NavigationRuntimeStats::default(),
            navigation_projection: None,
            navigation_projection_builds: 0,
            navigation_projection_component_rows: 0,
            repath_budget: NavRepathBudget::default(),
            repath_cursor: 0,
            repath_routes: HashMap::new(),
            repath_queries: 0,
        }
    }
}

#[derive(Debug)]
pub(super) struct BuiltinGeneratedBakeState {
    pub(super) snapshot: NavigationGeneratedBakeSnapshot,
    pub(super) loaded_handle: Option<NavMeshHandle>,
}

impl BuiltinNavigationState {
    pub(super) fn take_navigation_projection(
        &mut self,
        world: &World,
    ) -> NavigationWorldProjection {
        if self
            .navigation_projection
            .as_ref()
            .is_some_and(|projection| projection.generation == world.world_generation())
        {
            return self
                .navigation_projection
                .take()
                .expect("matching navigation projection must be present");
        }

        let projection = collect_navigation_world_projection(world);
        self.navigation_projection_builds = self.navigation_projection_builds.saturating_add(1);
        self.navigation_projection_component_rows = self
            .navigation_projection_component_rows
            .saturating_add(projection.agent_component_rows as u64)
            .saturating_add(projection.obstacle_component_rows as u64);
        projection
    }

    pub(super) fn store_navigation_projection(&mut self, projection: NavigationWorldProjection) {
        self.navigation_projection = Some(projection);
    }

    pub(super) fn begin_repath_frame(&mut self, agent_count: usize) -> usize {
        self.repath_budget.begin_frame();
        if agent_count == 0 {
            self.repath_cursor = 0;
            return 0;
        }
        self.repath_cursor %= agent_count;
        self.repath_cursor
    }

    pub(super) fn try_consume_repath_query(&mut self) -> bool {
        self.repath_budget.try_consume()
    }

    pub(super) fn set_repath_cursor(&mut self, cursor: usize, agent_count: usize) {
        self.repath_cursor = if agent_count == 0 {
            0
        } else {
            cursor % agent_count
        };
    }

    pub(super) fn cached_repath_target(
        &mut self,
        entity: u64,
        current: Vec3,
        destination: Vec3,
        agent_type: &str,
        area_mask: u64,
        stopping_distance: Real,
    ) -> Option<Vec3> {
        let matches_request = self.repath_routes.get(&entity).is_some_and(|route| {
            route.destination == destination
                && route.agent_type == agent_type
                && route.area_mask == area_mask
        });
        if !matches_request {
            self.repath_routes.remove(&entity);
            return None;
        }

        let route = self
            .repath_routes
            .get_mut(&entity)
            .expect("matching repath route must remain present");
        while route.next_waypoint < route.waypoints.len()
            && distance_xz(current, route.waypoints[route.next_waypoint])
                <= stopping_distance.max(0.0)
        {
            route.next_waypoint += 1;
        }
        if let Some(target) = route.waypoints.get(route.next_waypoint).copied() {
            return Some(target);
        }
        self.repath_routes.remove(&entity);
        None
    }

    pub(super) fn record_repath_query(&mut self) {
        self.repath_queries = self.repath_queries.saturating_add(1);
    }

    pub(super) fn store_repath_route(
        &mut self,
        entity: u64,
        destination: Vec3,
        agent_type: String,
        area_mask: u64,
        waypoints: Vec<Vec3>,
    ) {
        if waypoints.is_empty() {
            self.repath_routes.remove(&entity);
            return;
        }
        self.repath_routes.insert(
            entity,
            RuntimeRepathRoute {
                destination,
                agent_type,
                area_mask,
                waypoints,
                next_waypoint: 0,
            },
        );
    }

    pub(super) fn clear_repath_route(&mut self, entity: u64) {
        self.repath_routes.remove(&entity);
    }

    pub(super) fn generated_snapshot(
        &self,
        surface_entity: Option<u64>,
    ) -> NavigationGeneratedBakeSnapshot {
        self.generated
            .get(&surface_entity)
            .or_else(|| {
                surface_entity
                    .is_none()
                    .then(|| {
                        self.generated
                            .iter()
                            .min_by_key(|(surface, _)| **surface)
                            .map(|(_, state)| state)
                    })
                    .flatten()
            })
            .map(|state| state.snapshot.clone())
            .unwrap_or_else(|| NavigationGeneratedBakeSnapshot::empty(surface_entity))
    }

    pub(super) fn replace_generated_snapshot(&mut self, snapshot: NavigationGeneratedBakeSnapshot) {
        let key = snapshot.surface_entity;
        if let Some(previous) = self.generated.remove(&key) {
            if let Some(handle) = previous.loaded_handle {
                self.loaded.remove(&handle);
            }
        }
        let loaded_handle = snapshot
            .asset
            .as_ref()
            .filter(|asset| !asset.is_empty())
            .map(|asset| {
                let handle = NavMeshHandle(self.next_handle);
                self.next_handle = self.next_handle.saturating_add(1);
                self.loaded
                    .insert(handle, Arc::new(BakedNavMesh::new(asset.clone())));
                handle
            });
        if snapshot.asset.is_some() {
            self.generated.insert(
                key,
                BuiltinGeneratedBakeState {
                    snapshot,
                    loaded_handle,
                },
            );
        }
        self.stats.loaded_nav_meshes = self.loaded.len();
    }

    pub(super) fn selected_mesh_snapshot(
        &self,
        query_handle: Option<NavMeshHandle>,
    ) -> Result<Arc<BakedNavMesh>, NavigationError> {
        let handle = query_handle
            .or_else(|| self.loaded.keys().copied().min_by_key(|handle| handle.0))
            .ok_or_else(|| NavigationError::missing_nav_mesh("no nav mesh is loaded"))?;
        self.loaded.get(&handle).cloned().ok_or_else(|| {
            NavigationError::missing_nav_mesh(format!("nav mesh {:?} is not loaded", handle))
        })
    }
}

#[derive(Debug)]
struct RuntimeRepathRoute {
    destination: Vec3,
    agent_type: String,
    area_mask: u64,
    waypoints: Vec<Vec3>,
    next_waypoint: usize,
}
