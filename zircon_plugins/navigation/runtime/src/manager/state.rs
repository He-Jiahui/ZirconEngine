use std::collections::HashMap;

use zircon_runtime::asset::{NavMeshAsset, NavigationSettingsAsset};
use zircon_runtime::core::framework::navigation::{NavMeshHandle, NavigationRuntimeStats};

#[derive(Debug)]
pub(super) struct NavigationRuntimeState {
    pub(super) next_handle: u64,
    pub(super) loaded: HashMap<NavMeshHandle, NavMeshAsset>,
    pub(super) settings: NavigationSettingsAsset,
    pub(super) stats: NavigationRuntimeStats,
}

impl Default for NavigationRuntimeState {
    fn default() -> Self {
        Self {
            next_handle: 1,
            loaded: HashMap::new(),
            settings: NavigationSettingsAsset::default(),
            stats: NavigationRuntimeStats::default(),
        }
    }
}
