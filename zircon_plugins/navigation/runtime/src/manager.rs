pub(crate) mod agent_motion;
mod bake;
mod query;
mod state;
mod stats;
pub(crate) mod tick;
mod traversal;

use std::sync::MutexGuard;
use std::sync::{Arc, Mutex};

use zircon_plugin_navigation_recast::RecastBackend;
use zircon_runtime::asset::{NavMeshAsset, NavigationSettingsAsset};
use zircon_runtime::core::framework::navigation::{
    NavAgentTickReport, NavMeshBakeReport, NavMeshBakeRequest, NavMeshHandle, NavPathQuery,
    NavPathResult, NavQueryFilter, NavRaycastQuery, NavRaycastResult, NavSampleHit, NavSampleQuery,
    NavigationError, NavigationManager, NavigationRuntimeStats, DEFAULT_AGENT_TYPE,
};
use zircon_runtime::core::framework::tasks::TaskPoolDescriptor;
use zircon_runtime::core::math::Real;
use zircon_runtime::core::runtime::tasks::TaskPool;
use zircon_runtime::scene::World;

pub use self::bake::{
    NavMeshBakeTaskHandle, NavMeshBakeTaskState, NavMeshDirtyBakeReport, NavMeshDirtyBounds,
};
use self::state::NavigationRuntimeState;

#[derive(Clone, Debug)]
pub struct DefaultNavigationManager {
    pub(crate) backend: RecastBackend,
    pub(in crate::manager) bake_pool: TaskPool,
    pub(in crate::manager) state: Arc<Mutex<NavigationRuntimeState>>,
}

impl DefaultNavigationManager {
    pub(crate) fn lock_state(&self) -> MutexGuard<'_, NavigationRuntimeState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn new() -> Self {
        Self {
            backend: RecastBackend,
            bake_pool: TaskPool::new(
                TaskPoolDescriptor::async_compute().with_thread_name("navigation-bake"),
            ),
            state: Arc::new(Mutex::new(NavigationRuntimeState::default())),
        }
    }

    pub fn active_settings(&self) -> NavigationSettingsAsset {
        self.lock_state().settings.clone()
    }

    pub fn find_path_with_filter(
        &self,
        query: NavPathQuery,
        filter: &NavQueryFilter,
    ) -> Result<NavPathResult, NavigationError> {
        query::find_path_with_filter(self, query, filter)
    }

    pub(super) fn selected_asset(
        &self,
        query_handle: Option<NavMeshHandle>,
    ) -> Result<NavMeshAsset, NavigationError> {
        self.selected_handle_asset(query_handle)
            .map(|(_, asset)| asset)
    }

    pub(super) fn selected_handle_asset(
        &self,
        query_handle: Option<NavMeshHandle>,
    ) -> Result<(NavMeshHandle, NavMeshAsset), NavigationError> {
        let state = self.lock_state();
        let handle = query_handle
            .or_else(|| state.loaded.keys().copied().min_by_key(|handle| handle.0))
            .ok_or_else(|| NavigationError::missing_nav_mesh("no nav mesh is loaded"))?;
        state
            .loaded
            .get(&handle)
            .cloned()
            .map(|asset| (handle, asset))
            .ok_or_else(|| {
                NavigationError::missing_nav_mesh(format!("nav mesh {:?} is not loaded", handle))
            })
    }

    pub(crate) fn loaded_assets(&self) -> Vec<(NavMeshHandle, NavMeshAsset)> {
        let state = self.lock_state();
        let mut loaded = state
            .loaded
            .iter()
            .map(|(handle, asset)| (*handle, asset.clone()))
            .collect::<Vec<_>>();
        loaded.sort_by_key(|(handle, _)| handle.0);
        loaded
    }

    pub(in crate::manager) fn begin_bake_generation(&self, surface: Option<u64>) -> u64 {
        let mut state = self.lock_state();
        state.advance_bake_context(surface)
    }

    pub(in crate::manager) fn publish_bake(
        &self,
        surface: Option<u64>,
        generation: u64,
        tiled_bake: Option<(
            state::TiledBakeIdentity,
            zircon_plugin_navigation_recast::RecastTiledBakePlan,
            NavMeshAsset,
        )>,
        diagnostics: Vec<zircon_runtime::core::framework::navigation::NavMeshBakeDiagnostic>,
        counts: (usize, usize, usize),
    ) -> Result<(), NavigationError> {
        let mut state = self.lock_state();
        let context = state.bake_contexts.entry(surface).or_default();
        if context.current_generation != generation {
            return Err(NavigationError::new(
                zircon_runtime::core::framework::navigation::NavigationErrorKind::InvalidConfiguration,
                "navigation bake result was superseded by a newer request",
            ));
        }
        context.last_tiled_bake = tiled_bake.map(|(identity, plan, asset)| state::LastTiledBake {
            identity,
            plan,
            asset,
        });
        state.bake_diagnostics = diagnostics;
        state.stats.active_obstacles = counts.0;
        state.stats.active_off_mesh_links = counts.1;
        state.stats.active_off_mesh_bridges = counts.2;
        Ok(())
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
        let mut state = self.lock_state();
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
        let mut state = self.lock_state();
        state.settings = settings;
        state.crowds.clear();
        state.obstacle_worlds.clear();
        state.crowd_handle_cursor = 0;
        for context in state.bake_contexts.values_mut() {
            let generation = context.next_generation;
            context.next_generation = context.next_generation.saturating_add(1);
            context.current_generation = generation;
            context.last_tiled_bake = None;
        }
        state.bake_tasks.clear();
        state.dirty_bake_tasks.clear();
        Ok(())
    }

    fn find_path(&self, query: NavPathQuery) -> Result<NavPathResult, NavigationError> {
        query::find_path(self, query)
    }

    fn find_path_with_filter(
        &self,
        query: NavPathQuery,
        filter: &NavQueryFilter,
    ) -> Result<NavPathResult, NavigationError> {
        query::find_path_with_filter(self, query, filter)
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
        crate::agent::tick_world_agents(self, world, dt_seconds)
    }

    fn stats(&self) -> NavigationRuntimeStats {
        let state = self.lock_state();
        state.stats.clone()
    }
}

pub fn count_navigation_components(world: &World) -> NavigationRuntimeStats {
    stats::count_navigation_components(world)
}

pub fn default_agent_type() -> &'static str {
    DEFAULT_AGENT_TYPE
}
