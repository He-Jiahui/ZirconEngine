#[path = "../../sdf_state/declarations.rs"]
mod declarations;
#[path = "../../sdf_state/influence.rs"]
mod influence;
#[path = "../../sdf_state/residency.rs"]
mod residency;
#[path = "../../sdf_state/statistics.rs"]
mod statistics;
#[path = "../../sdf_state/synchronize.rs"]
mod synchronize;

pub(in crate::hybrid_gi) use declarations::{
    HybridGiGlobalSdfClipmapBounds, HybridGiGlobalSdfPageBuildRequest, HybridGiGlobalSdfPageKey,
    HybridGiGlobalSdfSceneState, GLOBAL_SDF_CLIPMAP_COUNT, GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT,
    GLOBAL_SDF_PAGES_PER_EDGE,
};
pub(in crate::hybrid_gi) use influence::GLOBAL_SDF_MAX_PAGE_CANDIDATES;

#[cfg(test)]
use synchronize::aabb_intersects;

#[cfg(test)]
#[path = "../../sdf_state/tests/cases.rs"]
mod tests;
