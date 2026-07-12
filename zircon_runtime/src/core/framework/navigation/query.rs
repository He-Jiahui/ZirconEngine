use serde::de::{Error as _, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

use crate::core::math::Real;

use super::constants::{
    NavAreaId, NavAreaMask, DEFAULT_AGENT_TYPE, DEFAULT_AREA_MASK, MAX_NAV_AREAS,
};
use super::handle::NavMeshHandle;

/// Per-query Detour filter state. Area costs are deliberately query-owned so AI actors can
/// choose different routes over the same immutable baked navmesh.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NavQueryFilter {
    #[serde(
        serialize_with = "serialize_area_costs",
        deserialize_with = "deserialize_area_costs"
    )]
    pub area_costs: [Real; MAX_NAV_AREAS],
    pub include_flags: u16,
    pub exclude_flags: u16,
}

fn serialize_area_costs<S>(
    area_costs: &[Real; MAX_NAV_AREAS],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut sequence = serializer.serialize_seq(Some(MAX_NAV_AREAS))?;
    for cost in area_costs {
        sequence.serialize_element(cost)?;
    }
    sequence.end()
}

fn deserialize_area_costs<'de, D>(deserializer: D) -> Result<[Real; MAX_NAV_AREAS], D::Error>
where
    D: Deserializer<'de>,
{
    struct AreaCostsVisitor;

    impl<'de> Visitor<'de> for AreaCostsVisitor {
        type Value = [Real; MAX_NAV_AREAS];

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                formatter,
                "exactly {MAX_NAV_AREAS} finite positive area costs"
            )
        }

        fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut area_costs = [1.0; MAX_NAV_AREAS];
            for (index, cost) in area_costs.iter_mut().enumerate() {
                let value = sequence
                    .next_element::<Real>()?
                    .ok_or_else(|| A::Error::invalid_length(index, &self))?;
                if !value.is_finite() || value <= 0.0 {
                    return Err(A::Error::custom(format_args!(
                        "area cost at index {index} must be finite and greater than zero"
                    )));
                }
                *cost = value;
            }
            if sequence.next_element::<serde::de::IgnoredAny>()?.is_some() {
                return Err(A::Error::invalid_length(MAX_NAV_AREAS + 1, &self));
            }
            Ok(area_costs)
        }
    }

    deserializer.deserialize_seq(AreaCostsVisitor)
}

impl NavQueryFilter {
    pub fn with_area_cost(mut self, area: NavAreaId, cost: Real) -> Self {
        if let Some(slot) = self.area_costs.get_mut(area as usize) {
            if cost.is_finite() && cost > 0.0 {
                *slot = cost;
            }
        }
        self
    }

    pub fn allows_area(&self, area: NavAreaId) -> bool {
        let flags = nav_area_flag(area);
        flags != 0 && flags & self.include_flags != 0 && flags & self.exclude_flags == 0
    }
}

impl Default for NavQueryFilter {
    fn default() -> Self {
        Self {
            area_costs: [1.0; MAX_NAV_AREAS],
            include_flags: u16::MAX,
            exclude_flags: 0,
        }
    }
}

/// Maps the 64 area identifiers onto Detour's 16-bit polygon flag space. The final flag is the
/// explicit overflow group, keeping high custom areas filterable without truncating them to zero.
pub const fn nav_area_flag(area: NavAreaId) -> u16 {
    match area {
        0 => 0,
        1..=15 => 1_u16 << (area - 1),
        _ => 1_u16 << 15,
    }
}

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
    #[serde(default)]
    pub off_mesh_link_id: Option<u32>,
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
