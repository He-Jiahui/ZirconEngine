use std::collections::HashMap;

use crate::core::framework::navigation::{
    NavMeshHandle, NavigationError, NavigationGeneratedBakeSnapshot, NavigationRuntimeStats,
    NavigationSettingsAsset,
};

use super::baked_mesh::BakedNavMesh;

#[derive(Debug)]
pub(super) struct BuiltinNavigationState {
    pub(super) next_handle: u64,
    pub(super) loaded: HashMap<NavMeshHandle, BakedNavMesh>,
    pub(super) generated: HashMap<Option<u64>, BuiltinGeneratedBakeState>,
    pub(super) settings: NavigationSettingsAsset,
    pub(super) stats: NavigationRuntimeStats,
}

impl Default for BuiltinNavigationState {
    fn default() -> Self {
        Self {
            next_handle: 1,
            loaded: HashMap::new(),
            generated: HashMap::new(),
            settings: NavigationSettingsAsset::default(),
            stats: NavigationRuntimeStats::default(),
        }
    }
}

#[derive(Debug)]
pub(super) struct BuiltinGeneratedBakeState {
    pub(super) snapshot: NavigationGeneratedBakeSnapshot,
    pub(super) loaded_handle: Option<NavMeshHandle>,
}

impl BuiltinNavigationState {
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
                self.loaded.insert(handle, BakedNavMesh::new(asset.clone()));
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

    pub(super) fn selected_mesh(
        &self,
        query_handle: Option<NavMeshHandle>,
    ) -> Result<&BakedNavMesh, NavigationError> {
        let handle = query_handle
            .or_else(|| self.loaded.keys().copied().min_by_key(|handle| handle.0))
            .ok_or_else(|| NavigationError::missing_nav_mesh("no nav mesh is loaded"))?;
        self.loaded.get(&handle).ok_or_else(|| {
            NavigationError::missing_nav_mesh(format!("nav mesh {:?} is not loaded", handle))
        })
    }
}
