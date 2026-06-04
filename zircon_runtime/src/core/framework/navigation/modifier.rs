use serde::{Deserialize, Serialize};

use super::constants::{NavAreaId, AREA_WALKABLE};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavMeshModifierMode {
    Add,
    Modify,
    Remove,
}

impl Default for NavMeshModifierMode {
    fn default() -> Self {
        Self::Modify
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NavMeshModifierDescriptor {
    pub mode: NavMeshModifierMode,
    pub affected_agents: Vec<String>,
    pub apply_to_children: bool,
    pub override_area: bool,
    pub area: NavAreaId,
    pub override_generate_links: bool,
    pub generate_links: bool,
}

impl Default for NavMeshModifierDescriptor {
    fn default() -> Self {
        Self {
            mode: NavMeshModifierMode::Modify,
            affected_agents: Vec::new(),
            apply_to_children: true,
            override_area: false,
            area: AREA_WALKABLE,
            override_generate_links: false,
            generate_links: true,
        }
    }
}
