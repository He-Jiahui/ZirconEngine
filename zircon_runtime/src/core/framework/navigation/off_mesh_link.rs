use serde::{Deserialize, Serialize};

use crate::core::math::Real;

use super::constants::{NavAreaId, AREA_JUMP, DEFAULT_AGENT_TYPE};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavLinkTraversalMode {
    Automatic,
    Manual,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavLinkMotion {
    Linear,
    #[default]
    Parabolic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OffMeshTraversePhase {
    Approach,
    Traverse,
    Exit,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OffMeshTraverseState {
    pub agent_entity: u64,
    pub nav_mesh: super::handle::NavMeshHandle,
    pub link_id: u32,
    pub owner_entity: u64,
    pub phase: OffMeshTraversePhase,
    pub progress: Real,
    pub start: [Real; 3],
    pub end: [Real; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OffMeshTraverseEventKind {
    Started,
    Completed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OffMeshTraverseEvent {
    pub kind: OffMeshTraverseEventKind,
    pub agent_entity: u64,
    pub nav_mesh: super::handle::NavMeshHandle,
    pub link_id: u32,
    pub owner_entity: u64,
    pub start: [Real; 3],
    pub end: [Real; 3],
}

impl OffMeshTraverseEvent {
    pub fn started(state: &OffMeshTraverseState) -> Self {
        Self::from_state(OffMeshTraverseEventKind::Started, state)
    }

    pub fn completed(state: &OffMeshTraverseState) -> Self {
        Self::from_state(OffMeshTraverseEventKind::Completed, state)
    }

    fn from_state(kind: OffMeshTraverseEventKind, state: &OffMeshTraverseState) -> Self {
        Self {
            kind,
            agent_entity: state.agent_entity,
            nav_mesh: state.nav_mesh,
            link_id: state.link_id,
            owner_entity: state.owner_entity,
            start: state.start,
            end: state.end,
        }
    }
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
    pub motion: NavLinkMotion,
    pub arc_height: Real,
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
            motion: NavLinkMotion::Parabolic,
            arc_height: 1.0,
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
    pub motion: NavLinkMotion,
    pub arc_height: Real,
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
            motion: NavLinkMotion::Linear,
            arc_height: 0.0,
        }
    }
}
