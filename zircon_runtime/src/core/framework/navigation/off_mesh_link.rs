use serde::{Deserialize, Serialize};

use crate::core::math::Real;

use super::constants::{NavAreaId, AREA_JUMP, DEFAULT_AGENT_TYPE};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavLinkTraversalMode {
    Automatic,
    Manual,
}

impl Default for NavLinkTraversalMode {
    fn default() -> Self {
        Self::Automatic
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NavMeshOffMeshLinkDescriptor {
    pub start_entity: Option<u64>,
    pub end_entity: Option<u64>,
    pub start_local_point: [Real; 3],
    pub end_local_point: [Real; 3],
    pub width: Real,
    pub bidirectional: bool,
    pub activated: bool,
    pub auto_update_positions: bool,
    pub cost_override: Option<Real>,
    pub area_type: NavAreaId,
    pub agent_type: String,
    pub traversal_mode: NavLinkTraversalMode,
}

impl Default for NavMeshOffMeshLinkDescriptor {
    fn default() -> Self {
        Self {
            start_entity: None,
            end_entity: None,
            start_local_point: [0.0, 0.0, 0.0],
            end_local_point: [0.0, 0.0, 1.0],
            width: 0.0,
            bidirectional: true,
            activated: true,
            auto_update_positions: true,
            cost_override: None,
            area_type: AREA_JUMP,
            agent_type: DEFAULT_AGENT_TYPE.to_string(),
            traversal_mode: NavLinkTraversalMode::Automatic,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NavMeshOffMeshBridgeDescriptor {
    pub start_entity: Option<u64>,
    pub end_entity: Option<u64>,
    pub start_local_point: [Real; 3],
    pub end_local_point: [Real; 3],
    pub width: Real,
    pub lane_count: u32,
    pub bidirectional: bool,
    pub activated: bool,
    pub cost_override: Option<Real>,
    pub area_type: NavAreaId,
    pub agent_type: String,
    pub traversal_mode: NavLinkTraversalMode,
}

impl Default for NavMeshOffMeshBridgeDescriptor {
    fn default() -> Self {
        Self {
            start_entity: None,
            end_entity: None,
            start_local_point: [-0.5, 0.0, 0.0],
            end_local_point: [0.5, 0.0, 0.0],
            width: 1.0,
            lane_count: 1,
            bidirectional: true,
            activated: true,
            cost_override: None,
            area_type: AREA_JUMP,
            agent_type: DEFAULT_AGENT_TYPE.to_string(),
            traversal_mode: NavLinkTraversalMode::Automatic,
        }
    }
}
