use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use zircon_plugin_navigation_recast::RecastTiledBakePlan;
use zircon_runtime::core::framework::navigation::NavMeshBakeDiagnostic;
use zircon_runtime::core::framework::navigation::{
    NavMeshAsset, NavigationGeneratedBakeSnapshot, NavigationSettingsAsset,
};
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
pub(super) struct GeneratedBakeState {
    pub(super) snapshot: NavigationGeneratedBakeSnapshot,
    pub(super) loaded_handle: Option<NavMeshHandle>,
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
    pub(super) overlay_generation: u64,
    pub(super) loaded: BTreeMap<u64, Arc<NavMeshAsset>>,
    pub(super) generated_bakes: HashMap<Option<u64>, GeneratedBakeState>,
    pub(super) settings: NavigationSettingsAsset,
    pub(crate) stats: NavigationRuntimeStats,
    pub(super) agent_motion: HashMap<u64, NavigationAgentMotionState>,
    pub(crate) crowds: HashMap<NavMeshHandle, crate::agent::NavigationCrowdRuntime>,
    pub(crate) obstacle_worlds:
        HashMap<NavMeshHandle, crate::runtime_obstacles::NavigationObstacleWorld>,
    pub(super) off_mesh_traversal: super::traversal::OffMeshTraversalRuntime,
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
            overlay_generation: 0,
            loaded: BTreeMap::new(),
            generated_bakes: HashMap::new(),
            settings: NavigationSettingsAsset::default(),
            stats: NavigationRuntimeStats::default(),
            agent_motion: HashMap::new(),
            crowds: HashMap::new(),
            obstacle_worlds: HashMap::new(),
            off_mesh_traversal: super::traversal::OffMeshTraversalRuntime::default(),
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
    pub(super) fn advance_overlay_generation(&mut self) {
        self.overlay_generation = self.overlay_generation.saturating_add(1);
    }

    pub(super) fn generated_snapshot(
        &self,
        surface_entity: Option<u64>,
    ) -> NavigationGeneratedBakeSnapshot {
        self.generated_bakes
            .get(&surface_entity)
            .or_else(|| {
                surface_entity
                    .is_none()
                    .then(|| {
                        self.generated_bakes
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
        if let Some(previous) = self.generated_bakes.remove(&key) {
            if let Some(handle) = previous.loaded_handle {
                self.loaded.remove(&handle.0);
            }
        }
        let loaded_handle = snapshot
            .asset
            .as_ref()
            .filter(|asset| !asset.is_empty())
            .map(|asset| {
                let handle = NavMeshHandle(self.next_handle);
                self.next_handle = self.next_handle.saturating_add(1);
                self.loaded.insert(handle.0, Arc::new(asset.clone()));
                handle
            });
        if snapshot.asset.is_some() {
            self.generated_bakes.insert(
                key,
                GeneratedBakeState {
                    snapshot,
                    loaded_handle,
                },
            );
        }
        self.stats.loaded_nav_meshes = self.loaded.len();
        self.advance_overlay_generation();
    }

    pub(super) fn clear_generated_snapshots(&mut self) {
        let handles = self
            .generated_bakes
            .drain()
            .filter_map(|(_, generated)| generated.loaded_handle)
            .collect::<Vec<_>>();
        for handle in handles {
            self.loaded.remove(&handle.0);
        }
        self.stats.loaded_nav_meshes = self.loaded.len();
        self.advance_overlay_generation();
    }

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
