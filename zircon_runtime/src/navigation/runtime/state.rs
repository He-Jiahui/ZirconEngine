use std::collections::HashMap;

use crate::asset::NavigationSettingsAsset;
use crate::core::framework::navigation::{NavMeshHandle, NavigationError, NavigationRuntimeStats};

use super::baked_mesh::BakedNavMesh;

#[derive(Debug)]
pub(super) struct BuiltinNavigationState {
    pub(super) next_handle: u64,
    pub(super) loaded: HashMap<NavMeshHandle, BakedNavMesh>,
    pub(super) settings: NavigationSettingsAsset,
    pub(super) stats: NavigationRuntimeStats,
}

impl Default for BuiltinNavigationState {
    fn default() -> Self {
        Self {
            next_handle: 1,
            loaded: HashMap::new(),
            settings: NavigationSettingsAsset::default(),
            stats: NavigationRuntimeStats::default(),
        }
    }
}

impl BuiltinNavigationState {
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
