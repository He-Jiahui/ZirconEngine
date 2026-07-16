use serde::{Deserialize, Serialize};

use super::{NavMeshAsset, NavMeshBakeReport};

pub const NAVIGATION_BAKE_SCENE_OPERATION: &str = "navigation.bake.scene";
pub const NAVIGATION_BAKE_SURFACE_OPERATION: &str = "navigation.bake.surface";
pub const NAVIGATION_CLEAR_SURFACE_OPERATION: &str = "navigation.bake.clear_surface";
pub const NAVIGATION_RESTORE_BAKE_OPERATION: &str = "navigation.bake.restore_snapshot";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NavigationClearBakeRequest {
    pub surface_entity: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NavigationGeneratedBakeSnapshot {
    pub surface_entity: Option<u64>,
    pub asset: Option<NavMeshAsset>,
    pub output_asset: Option<String>,
}

impl NavigationGeneratedBakeSnapshot {
    pub fn empty(surface_entity: Option<u64>) -> Self {
        Self {
            surface_entity,
            asset: None,
            output_asset: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NavigationGeneratedBakeChange {
    pub before: NavigationGeneratedBakeSnapshot,
    pub after: NavigationGeneratedBakeSnapshot,
    pub report: Option<NavMeshBakeReport>,
}
