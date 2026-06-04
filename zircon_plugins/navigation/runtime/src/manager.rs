mod bake;
mod query;
mod state;
mod stats;
mod tick;

use std::sync::Mutex;

use zircon_plugin_navigation_recast::RecastBackend;
use zircon_runtime::asset::{NavMeshAsset, NavigationSettingsAsset};
use zircon_runtime::core::framework::navigation::{
    NavAgentTickReport, NavMeshBakeReport, NavMeshBakeRequest, NavMeshHandle, NavPathQuery,
    NavPathResult, NavRaycastQuery, NavRaycastResult, NavSampleHit, NavSampleQuery,
    NavigationError, NavigationManager, NavigationRuntimeStats, DEFAULT_AGENT_TYPE,
};
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::World;

use self::state::NavigationRuntimeState;

#[derive(Debug)]
pub struct DefaultNavigationManager {
    pub(super) backend: RecastBackend,
    pub(super) state: Mutex<NavigationRuntimeState>,
}

impl DefaultNavigationManager {
    pub fn new() -> Self {
        Self {
            backend: RecastBackend,
            state: Mutex::new(NavigationRuntimeState::default()),
        }
    }

    pub fn active_settings(&self) -> NavigationSettingsAsset {
        self.state
            .lock()
            .expect("navigation state lock poisoned")
            .settings
            .clone()
    }

    pub(super) fn selected_asset(
        &self,
        query_handle: Option<NavMeshHandle>,
    ) -> Result<NavMeshAsset, NavigationError> {
        let state = self.state.lock().expect("navigation state lock poisoned");
        let handle = query_handle
            .or_else(|| state.loaded.keys().copied().min_by_key(|handle| handle.0))
            .ok_or_else(|| NavigationError::missing_nav_mesh("no nav mesh is loaded"))?;
        state.loaded.get(&handle).cloned().ok_or_else(|| {
            NavigationError::missing_nav_mesh(format!("nav mesh {:?} is not loaded", handle))
        })
    }

    pub(super) fn record_bake_counts(
        &self,
        active_obstacles: usize,
        active_off_mesh_links: usize,
        active_off_mesh_bridges: usize,
    ) {
        let mut state = self.state.lock().expect("navigation state lock poisoned");
        state.stats.active_obstacles = active_obstacles;
        state.stats.active_off_mesh_links = active_off_mesh_links;
        state.stats.active_off_mesh_bridges = active_off_mesh_bridges;
    }
}

impl Default for DefaultNavigationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationManager for DefaultNavigationManager {
    fn bake_surface(
        &self,
        world: &World,
        request: NavMeshBakeRequest,
    ) -> Result<NavMeshBakeReport, NavigationError> {
        bake::bake_surface(self, world, request)
    }

    fn load_nav_mesh(&self, asset: NavMeshAsset) -> Result<NavMeshHandle, NavigationError> {
        let mut state = self.state.lock().expect("navigation state lock poisoned");
        let handle = NavMeshHandle(state.next_handle);
        state.next_handle += 1;
        state.loaded.insert(handle, asset);
        state.stats.loaded_nav_meshes = state.loaded.len();
        Ok(handle)
    }

    fn load_navigation_settings(
        &self,
        settings: NavigationSettingsAsset,
    ) -> Result<(), NavigationError> {
        crate::settings_validation::validate_navigation_settings(&settings)?;
        let mut state = self.state.lock().expect("navigation state lock poisoned");
        state.settings = settings;
        Ok(())
    }

    fn find_path(&self, query: NavPathQuery) -> Result<NavPathResult, NavigationError> {
        query::find_path(self, query)
    }

    fn sample_position(
        &self,
        query: NavSampleQuery,
    ) -> Result<Option<NavSampleHit>, NavigationError> {
        query::sample_position(self, query)
    }

    fn raycast(&self, query: NavRaycastQuery) -> Result<NavRaycastResult, NavigationError> {
        query::raycast(self, query)
    }

    fn tick_world_agents(
        &self,
        world: &mut World,
        dt_seconds: Real,
    ) -> Result<NavAgentTickReport, NavigationError> {
        tick::tick_world_agents(self, world, dt_seconds)
    }

    fn stats(&self) -> NavigationRuntimeStats {
        let state = self.state.lock().expect("navigation state lock poisoned");
        state.stats.clone()
    }
}

pub fn count_navigation_components(world: &World) -> NavigationRuntimeStats {
    stats::count_navigation_components(world)
}

pub fn default_agent_type() -> &'static str {
    DEFAULT_AGENT_TYPE
}
