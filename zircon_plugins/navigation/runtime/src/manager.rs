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
use zircon_runtime::asset::ProjectAssetManagerAccess;
use zircon_runtime::core::framework::navigation::{
    NavAgentTickReport, NavMeshAsset, NavMeshBakeReport, NavMeshBakeRequest, NavMeshHandle,
    NavPathQuery, NavPathResult, NavQueryFilter, NavRaycastQuery, NavRaycastResult, NavSampleHit,
    NavSampleQuery, NavigationError, NavigationGeneratedBakeSnapshot, NavigationManager,
    NavigationRuntimeStats, NavigationSettingsAsset, DEFAULT_AGENT_TYPE,
};
use zircon_runtime::core::math::Real;
use zircon_runtime::core::runtime::tasks::TaskPool;
use zircon_runtime::scene::{LevelSystem, SceneNavigationRuntime, World};

pub use self::bake::{
    NavMeshBakeTaskHandle, NavMeshBakeTaskState, NavMeshDirtyBakeReport, NavMeshDirtyBounds,
};
use self::state::{BakeGenerationToken, NavigationRuntimeState};
use crate::NavigationOverlayFrame;

#[derive(Clone, Debug)]
/// 管理器集中持有已加载网格、烘焙代次与代理状态；场景 trait 和插件驱动共享这份状态。
pub struct DefaultNavigationManager {
    pub(crate) backend: RecastBackend,
    pub(in crate::manager) bake_pool: TaskPool,
    pub(in crate::manager) project_assets: ProjectAssetManagerAccess,
    pub(in crate::manager) state: Arc<Mutex<NavigationRuntimeState>>,
}

impl DefaultNavigationManager {
    pub(crate) fn lock_state(&self) -> MutexGuard<'_, NavigationRuntimeState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn new(bake_pool: TaskPool, project_assets: ProjectAssetManagerAccess) -> Self {
        Self {
            backend: RecastBackend,
            bake_pool,
            project_assets,
            state: Arc::new(Mutex::new(NavigationRuntimeState::default())),
        }
    }

    pub(in crate::manager) fn project_asset_access(&self) -> &ProjectAssetManagerAccess {
        &self.project_assets
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
            .or_else(|| state.loaded.keys().next().copied().map(NavMeshHandle))
            .ok_or_else(|| NavigationError::missing_nav_mesh("no nav mesh is loaded"))?;
        state
            .loaded
            .get(&handle.0)
            .map(|asset| (handle, asset.as_ref().clone()))
            .ok_or_else(|| {
                NavigationError::missing_nav_mesh(format!("nav mesh {:?} is not loaded", handle))
            })
    }

    pub(crate) fn loaded_assets(&self) -> Vec<(NavMeshHandle, Arc<NavMeshAsset>)> {
        let state = self.lock_state();
        state
            .loaded
            .iter()
            .map(|(handle, asset)| (NavMeshHandle(*handle), Arc::clone(asset)))
            .collect()
    }

    pub fn navigation_overlay_frame(
        &self,
        tick_report: NavAgentTickReport,
    ) -> NavigationOverlayFrame {
        let state = self.lock_state();
        NavigationOverlayFrame::from_assets(
            state.overlay_generation,
            state.loaded.values().map(Arc::as_ref),
            tick_report,
        )
    }

    pub(in crate::manager) fn begin_bake_generation(
        &self,
        surface: Option<u64>,
    ) -> Result<u64, NavigationError> {
        let mut state = self.lock_state();
        state.try_advance_bake_context(surface)
    }

    pub(in crate::manager) fn capture_bake_generation(
        &self,
        surface: Option<u64>,
    ) -> BakeGenerationToken {
        self.lock_state().bake_generation_token(surface)
    }

    pub(in crate::manager) fn capture_generated_mutation_epoch(&self) -> u64 {
        self.lock_state().generated_mutation_epoch()
    }

    pub(in crate::manager) fn publish_bake(
        &self,
        surface: Option<u64>,
        generation: u64,
        generated_mutation_epoch: u64,
        tiled_bake: Option<(
            state::TiledBakeIdentity,
            zircon_plugin_navigation_recast::RecastTiledBakePlan,
            NavMeshAsset,
        )>,
        generated_snapshot: NavigationGeneratedBakeSnapshot,
        diagnostics: Vec<zircon_runtime::core::framework::navigation::NavMeshBakeDiagnostic>,
        counts: (usize, usize, usize),
    ) -> Result<(), NavigationError> {
        let mut state = self.lock_state();
        if state.generated_mutation_epoch != generated_mutation_epoch {
            return Err(NavigationError::new(
                zircon_runtime::core::framework::navigation::NavigationErrorKind::InvalidConfiguration,
                "navigation bake result was superseded by a generated-state mutation",
            ));
        }
        state.ensure_generated_mutation_epoch_available()?;
        {
            let context = state.bake_contexts.entry(surface).or_default();
            if context.current_generation != generation {
                return Err(NavigationError::new(
                    zircon_runtime::core::framework::navigation::NavigationErrorKind::InvalidConfiguration,
                    "navigation bake result was superseded by a newer request",
                ));
            }
            if context.next_generation == u64::MAX {
                return Err(NavigationError::new(
                    zircon_runtime::core::framework::navigation::NavigationErrorKind::InvalidConfiguration,
                    "navigation bake generation exhausted before publication",
                ));
            }
            context.last_tiled_bake =
                tiled_bake.map(|(identity, plan, asset)| state::LastTiledBake {
                    identity,
                    plan,
                    asset,
                });
        }
        state.replace_generated_snapshot(generated_snapshot)?;
        state.bake_diagnostics = diagnostics;
        state.stats.active_obstacles = counts.0;
        state.stats.active_off_mesh_links = counts.1;
        state.stats.active_off_mesh_bridges = counts.2;
        Ok(())
    }

    pub(in crate::manager) fn publish_operation_bake(
        &self,
        token: BakeGenerationToken,
        expected_before: &NavigationGeneratedBakeSnapshot,
        tiled_bake: Option<(
            state::TiledBakeIdentity,
            zircon_plugin_navigation_recast::RecastTiledBakePlan,
            NavMeshAsset,
        )>,
        generated_snapshot: NavigationGeneratedBakeSnapshot,
        diagnostics: Vec<zircon_runtime::core::framework::navigation::NavMeshBakeDiagnostic>,
        counts: (usize, usize, usize),
    ) -> Result<(), NavigationError> {
        let mut state = self.lock_state();
        let current = state.bake_generation_token(token.surface);
        if current != token {
            return Err(NavigationError::new(
                zircon_runtime::core::framework::navigation::NavigationErrorKind::InvalidConfiguration,
                "navigation bake result was superseded by a newer request or manager state change",
            ));
        }
        if state.generated_snapshot(expected_before.surface_entity) != *expected_before {
            return Err(NavigationError::new(
                zircon_runtime::core::framework::navigation::NavigationErrorKind::InvalidConfiguration,
                "navigation generated bake state changed before owner apply",
            ));
        }
        if state
            .bake_contexts
            .get(&token.surface)
            .is_some_and(|context| context.next_generation == u64::MAX)
        {
            return Err(NavigationError::new(
                zircon_runtime::core::framework::navigation::NavigationErrorKind::InvalidConfiguration,
                "navigation bake generation exhausted before owner apply",
            ));
        }
        state.ensure_generated_mutation_epoch_available()?;
        let generation = state.try_advance_bake_context(token.surface)?;
        if generation != token.next_generation {
            return Err(NavigationError::new(
                zircon_runtime::core::framework::navigation::NavigationErrorKind::InvalidConfiguration,
                "navigation bake generation changed before owner apply",
            ));
        }
        let context = state.bake_contexts.entry(token.surface).or_default();
        context.last_tiled_bake = tiled_bake.map(|(identity, plan, asset)| state::LastTiledBake {
            identity,
            plan,
            asset,
        });
        state.replace_generated_snapshot(generated_snapshot)?;
        state.bake_diagnostics = diagnostics;
        state.stats.active_obstacles = counts.0;
        state.stats.active_off_mesh_links = counts.1;
        state.stats.active_off_mesh_bridges = counts.2;
        Ok(())
    }
}

impl DefaultNavigationManager {
    pub fn bake_surface(
        &self,
        level: &LevelSystem,
        request: NavMeshBakeRequest,
    ) -> Result<NavMeshBakeReport, NavigationError> {
        bake::bake_surface(self, level, request)
    }

    fn load_nav_mesh(&self, asset: NavMeshAsset) -> Result<NavMeshHandle, NavigationError> {
        let mut state = self.lock_state();
        let handle = NavMeshHandle(state.next_handle);
        state.next_handle += 1;
        state.loaded.insert(handle.0, Arc::new(asset));
        state.stats.loaded_nav_meshes = state.loaded.len();
        state.advance_overlay_generation();
        Ok(handle)
    }

    fn load_navigation_settings(
        &self,
        settings: NavigationSettingsAsset,
    ) -> Result<(), NavigationError> {
        crate::settings_validation::validate_navigation_settings(&settings)?;
        let mut state = self.lock_state();
        state.ensure_generated_mutation_epoch_available()?;
        state.ensure_bake_generations_available()?;
        state.settings = settings;
        state.clear_generated_snapshots()?;
        state.crowds.clear();
        state.obstacle_worlds.clear();
        state.off_mesh_traversal = traversal::OffMeshTraversalRuntime::default();
        state.crowd_handle_cursor = 0;
        for context in state.bake_contexts.values_mut() {
            let generation = context.next_generation;
            context.next_generation = context
                .next_generation
                .checked_add(1)
                .expect("bake generation availability was checked above");
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

    fn sample_position(
        &self,
        query: NavSampleQuery,
    ) -> Result<Option<NavSampleHit>, NavigationError> {
        query::sample_position(self, query)
    }

    fn raycast(&self, query: NavRaycastQuery) -> Result<NavRaycastResult, NavigationError> {
        query::raycast(self, query)
    }

    pub fn tick_world_agents(
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

impl NavigationManager for DefaultNavigationManager {
    fn load_nav_mesh(&self, asset: NavMeshAsset) -> Result<NavMeshHandle, NavigationError> {
        DefaultNavigationManager::load_nav_mesh(self, asset)
    }

    fn load_navigation_settings(
        &self,
        settings: NavigationSettingsAsset,
    ) -> Result<(), NavigationError> {
        DefaultNavigationManager::load_navigation_settings(self, settings)
    }

    fn find_path(&self, query: NavPathQuery) -> Result<NavPathResult, NavigationError> {
        DefaultNavigationManager::find_path(self, query)
    }

    fn find_path_with_filter(
        &self,
        query: NavPathQuery,
        filter: &NavQueryFilter,
    ) -> Result<NavPathResult, NavigationError> {
        DefaultNavigationManager::find_path_with_filter(self, query, filter)
    }

    fn sample_position(
        &self,
        query: NavSampleQuery,
    ) -> Result<Option<NavSampleHit>, NavigationError> {
        DefaultNavigationManager::sample_position(self, query)
    }

    fn raycast(&self, query: NavRaycastQuery) -> Result<NavRaycastResult, NavigationError> {
        DefaultNavigationManager::raycast(self, query)
    }

    fn stats(&self) -> NavigationRuntimeStats {
        DefaultNavigationManager::stats(self)
    }
}

impl SceneNavigationRuntime for DefaultNavigationManager {
    fn bake_surface(
        &self,
        level: &LevelSystem,
        request: NavMeshBakeRequest,
    ) -> Result<NavMeshBakeReport, NavigationError> {
        DefaultNavigationManager::bake_surface(self, level, request)
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
        self.lock_state().replace_generated_snapshot(snapshot)
    }

    fn generated_bake_mutation_epoch(&self, _surface_entity: Option<u64>) -> u64 {
        self.capture_generated_mutation_epoch()
    }

    fn capture_bake_operation(
        &self,
        source: &zircon_runtime::scene::WorldPublicationSource,
        request: NavMeshBakeRequest,
    ) -> Result<
        (
            Box<dyn std::any::Any + Send>,
            usize,
            NavigationGeneratedBakeSnapshot,
        ),
        NavigationError,
    > {
        bake::capture_bake_operation(self, source, request)
    }

    fn prepare_bake_operation(
        &self,
        snapshot: Box<dyn std::any::Any + Send>,
    ) -> Result<
        (
            Box<dyn std::any::Any + Send>,
            NavMeshBakeReport,
            NavigationGeneratedBakeSnapshot,
            usize,
        ),
        NavigationError,
    > {
        bake::prepare_bake_operation(self, snapshot)
    }

    fn apply_bake_operation(
        &self,
        source: &zircon_runtime::scene::WorldPublicationSource,
        expected_before: &NavigationGeneratedBakeSnapshot,
        prepared: Box<dyn std::any::Any + Send>,
    ) -> Result<(), NavigationError> {
        bake::apply_bake_operation(self, source, expected_before, prepared)
    }

    fn tick_world_agents(
        &self,
        world: &mut World,
        dt_seconds: Real,
    ) -> Result<NavAgentTickReport, NavigationError> {
        DefaultNavigationManager::tick_world_agents(self, world, dt_seconds)
    }
}

pub fn count_navigation_components(world: &World) -> NavigationRuntimeStats {
    stats::count_navigation_components(world)
}

pub fn default_agent_type() -> &'static str {
    DEFAULT_AGENT_TYPE
}
