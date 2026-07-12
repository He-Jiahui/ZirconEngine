use std::collections::HashMap;

use zircon_plugin_navigation_recast::RecastTiledBakePlan;
use zircon_runtime::asset::{NavMeshAsset, NavigationSettingsAsset};
use zircon_runtime::core::framework::navigation::NavMeshBakeDiagnostic;
use zircon_runtime::core::framework::navigation::{
    NavMeshHandle, NavMeshSurfaceDescriptor, NavigationRuntimeStats,
};

use super::agent_motion::NavigationAgentMotionState;
use super::bake::task_pool::{NavMeshBakeTaskHandle, PendingTiledBake};
use super::bake::PendingDirtyBake;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct TiledBakeIdentity {
    pub(super) surface_entity: Option<u64>,
    pub(super) agent_type: String,
    pub(super) surface: NavMeshSurfaceDescriptor,
    pub(super) settings: NavigationSettingsAsset,
}

#[derive(Clone, Debug)]
pub(super) struct LastTiledBake {
    pub(super) identity: TiledBakeIdentity,
    pub(super) plan: RecastTiledBakePlan,
    pub(super) asset: NavMeshAsset,
}

#[derive(Clone, Debug)]
pub(super) struct BakeContextState {
    pub(super) next_generation: u64,
    pub(super) current_generation: u64,
    pub(super) last_tiled_bake: Option<LastTiledBake>,
}

impl Default for BakeContextState {
    fn default() -> Self {
        Self {
            next_generation: 1,
            current_generation: 0,
            last_tiled_bake: None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct NavigationRuntimeState {
    pub(super) next_handle: u64,
    pub(super) loaded: HashMap<NavMeshHandle, NavMeshAsset>,
    pub(super) settings: NavigationSettingsAsset,
    pub(crate) stats: NavigationRuntimeStats,
    pub(super) agent_motion: HashMap<u64, NavigationAgentMotionState>,
    pub(crate) crowds: HashMap<NavMeshHandle, crate::agent::NavigationCrowdRuntime>,
    pub(crate) obstacle_worlds:
        HashMap<NavMeshHandle, crate::runtime_obstacles::NavigationObstacleWorld>,
    pub(crate) crowd_handle_cursor: usize,
    pub(super) next_bake_task: u64,
    pub(super) bake_contexts: HashMap<Option<u64>, BakeContextState>,
    pub(super) bake_tasks: HashMap<NavMeshBakeTaskHandle, PendingTiledBake>,
    pub(super) dirty_bake_tasks: HashMap<NavMeshBakeTaskHandle, PendingDirtyBake>,
    pub(super) bake_diagnostics: Vec<NavMeshBakeDiagnostic>,
}

impl Default for NavigationRuntimeState {
    fn default() -> Self {
        Self {
            next_handle: 1,
            loaded: HashMap::new(),
            settings: NavigationSettingsAsset::default(),
            stats: NavigationRuntimeStats::default(),
            agent_motion: HashMap::new(),
            crowds: HashMap::new(),
            obstacle_worlds: HashMap::new(),
            crowd_handle_cursor: 0,
            next_bake_task: 1,
            bake_contexts: HashMap::new(),
            bake_tasks: HashMap::new(),
            dirty_bake_tasks: HashMap::new(),
            bake_diagnostics: Vec::new(),
        }
    }
}

impl NavigationRuntimeState {
    pub(super) fn advance_bake_context(&mut self, surface: Option<u64>) -> u64 {
        let context = self.bake_contexts.entry(surface).or_default();
        let generation = context.next_generation;
        context.next_generation = context.next_generation.saturating_add(1);
        context.current_generation = generation;
        self.bake_tasks
            .retain(|_, task| task.surface_entity() != surface);
        self.dirty_bake_tasks
            .retain(|_, task| task.surface_entity() != surface);
        generation
    }
}
