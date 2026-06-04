use serde::{Deserialize, Serialize};

use crate::core::math::Real;

use super::constants::{NavAreaId, NavAreaMask, DEFAULT_AGENT_TYPE, DEFAULT_AREA_MASK};
use super::handle::NavMeshHandle;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavPathQuery {
    pub nav_mesh: Option<NavMeshHandle>,
    pub start: [Real; 3],
    pub end: [Real; 3],
    pub agent_type: String,
    pub area_mask: NavAreaMask,
}

impl NavPathQuery {
    pub fn new(start: [Real; 3], end: [Real; 3]) -> Self {
        Self {
            nav_mesh: None,
            start,
            end,
            agent_type: DEFAULT_AGENT_TYPE.to_string(),
            area_mask: DEFAULT_AREA_MASK,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavPathStatus {
    Complete,
    Partial,
    NoPath,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavPathPoint {
    pub position: [Real; 3],
    pub area: NavAreaId,
    pub flags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavPathResult {
    pub status: NavPathStatus,
    pub points: Vec<NavPathPoint>,
    pub length: Real,
    pub visited_nodes: usize,
}

impl NavPathResult {
    pub fn no_path() -> Self {
        Self {
            status: NavPathStatus::NoPath,
            points: Vec::new(),
            length: 0.0,
            visited_nodes: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavSampleQuery {
    pub nav_mesh: Option<NavMeshHandle>,
    pub position: [Real; 3],
    pub extents: [Real; 3],
    pub agent_type: String,
    pub area_mask: NavAreaMask,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavSampleHit {
    pub position: [Real; 3],
    pub distance: Real,
    pub area: NavAreaId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavRaycastQuery {
    pub nav_mesh: Option<NavMeshHandle>,
    pub start: [Real; 3],
    pub end: [Real; 3],
    pub agent_type: String,
    pub area_mask: NavAreaMask,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavRaycastResult {
    pub hit: bool,
    pub position: [Real; 3],
    pub normal: [Real; 3],
    pub distance: Real,
}
