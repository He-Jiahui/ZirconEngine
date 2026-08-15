mod declarations;
mod influence;
mod residency;
mod statistics;
mod synchronize;

pub(in crate::hybrid_gi) use declarations::{
    GLOBAL_SDF_CLIPMAP_COUNT, GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT, GLOBAL_SDF_PAGES_PER_EDGE,
    HybridGiGlobalSdfClipmapBounds, HybridGiGlobalSdfPageBuildRequest, HybridGiGlobalSdfPageKey,
    HybridGiGlobalSdfSceneState,
};
pub(in crate::hybrid_gi) use influence::GLOBAL_SDF_MAX_PAGE_CANDIDATES;

#[cfg(test)]
use synchronize::aabb_intersects;

#[cfg(test)]
mod tests;
