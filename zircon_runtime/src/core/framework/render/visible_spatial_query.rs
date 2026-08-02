use std::fmt;
use std::sync::Arc;

use crate::core::framework::scene::EntityId;
use crate::core::math::{Real, Vec3};

use super::{RenderViewportHandle, RenderWorldSnapshotHandle};

/// Conservative world-space sphere used by renderer-neutral spatial queries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderSpatialBounds {
    pub center: Vec3,
    pub radius: Real,
}

impl RenderSpatialBounds {
    pub const fn new(center: Vec3, radius: Real) -> Self {
        Self { center, radius }
    }
}

/// Conservative world-space ray used by renderer-visible spatial queries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderSpatialRay {
    pub origin: Vec3,
    pub direction: Vec3,
    pub max_distance: Real,
}

impl RenderSpatialRay {
    pub const fn new(origin: Vec3, direction: Vec3, max_distance: Real) -> Self {
        Self {
            origin,
            direction,
            max_distance,
        }
    }
}

/// View whose renderer-visible spatial set backs a query snapshot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderVisibleSpatialQueryView {
    #[default]
    MainCamera,
}

/// Identity required to reject a query result from an obsolete world or rendered frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderVisibleSpatialQuerySnapshotId {
    pub world: RenderWorldSnapshotHandle,
    pub viewport: RenderViewportHandle,
    pub frame_generation: u64,
    pub view: RenderVisibleSpatialQueryView,
}

impl RenderVisibleSpatialQuerySnapshotId {
    pub const fn new(
        world: RenderWorldSnapshotHandle,
        viewport: RenderViewportHandle,
        frame_generation: u64,
        view: RenderVisibleSpatialQueryView,
    ) -> Self {
        Self {
            world,
            viewport,
            frame_generation,
            view,
        }
    }
}

/// Work performed by a visible spatial query. Counts are per query, not scene totals.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderVisibleSpatialQueryStats {
    pub visited_node_count: usize,
    pub candidate_count: usize,
    pub hit_count: usize,
}

/// Stable entity owners returned from a renderer-visible spatial query.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVisibleSpatialQueryResult {
    /// Sorted and deduplicated runtime entities. Multiple render primitives may share an owner.
    pub entities: Vec<EntityId>,
    pub stats: RenderVisibleSpatialQueryStats,
}

/// Opaque renderer-owned query implementation. Consumers only observe immutable results.
pub trait RenderVisibleSpatialQuery: Send + Sync {
    fn query_bounds(&self, bounds: RenderSpatialBounds) -> RenderVisibleSpatialQueryResult;

    fn query_ray(&self, _ray: RenderSpatialRay) -> RenderVisibleSpatialQueryResult {
        RenderVisibleSpatialQueryResult::default()
    }
}

/// Immutable, generation-bound view of the spatial set accepted by renderer visibility.
#[derive(Clone)]
pub struct RenderVisibleSpatialQuerySnapshot {
    identity: RenderVisibleSpatialQuerySnapshotId,
    query: Arc<dyn RenderVisibleSpatialQuery>,
}

impl RenderVisibleSpatialQuerySnapshot {
    pub(crate) fn new(
        identity: RenderVisibleSpatialQuerySnapshotId,
        query: Arc<dyn RenderVisibleSpatialQuery>,
    ) -> Self {
        Self { identity, query }
    }

    pub const fn identity(&self) -> RenderVisibleSpatialQuerySnapshotId {
        self.identity
    }

    pub fn query_bounds(&self, bounds: RenderSpatialBounds) -> RenderVisibleSpatialQueryResult {
        self.query.query_bounds(bounds)
    }

    pub fn query_ray(&self, ray: RenderSpatialRay) -> RenderVisibleSpatialQueryResult {
        self.query.query_ray(ray)
    }
}

impl fmt::Debug for RenderVisibleSpatialQuerySnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RenderVisibleSpatialQuerySnapshot")
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedQuery;

    impl RenderVisibleSpatialQuery for FixedQuery {
        fn query_bounds(&self, _bounds: RenderSpatialBounds) -> RenderVisibleSpatialQueryResult {
            RenderVisibleSpatialQueryResult {
                entities: vec![2, 7],
                stats: RenderVisibleSpatialQueryStats {
                    visited_node_count: 3,
                    candidate_count: 2,
                    hit_count: 2,
                },
            }
        }

        fn query_ray(&self, _ray: RenderSpatialRay) -> RenderVisibleSpatialQueryResult {
            RenderVisibleSpatialQueryResult {
                entities: vec![7],
                stats: RenderVisibleSpatialQueryStats {
                    visited_node_count: 2,
                    candidate_count: 1,
                    hit_count: 1,
                },
            }
        }
    }

    #[test]
    fn visible_spatial_snapshot_keeps_generation_bound_identity_and_opaque_query() {
        let identity = RenderVisibleSpatialQuerySnapshotId::new(
            RenderWorldSnapshotHandle::new(4),
            RenderViewportHandle::new(9),
            12,
            RenderVisibleSpatialQueryView::MainCamera,
        );
        let snapshot = RenderVisibleSpatialQuerySnapshot::new(identity, Arc::new(FixedQuery));

        assert_eq!(snapshot.identity(), identity);
        assert_eq!(
            snapshot.query_bounds(RenderSpatialBounds::new(Vec3::ZERO, 1.0)),
            RenderVisibleSpatialQueryResult {
                entities: vec![2, 7],
                stats: RenderVisibleSpatialQueryStats {
                    visited_node_count: 3,
                    candidate_count: 2,
                    hit_count: 2,
                },
            }
        );
        assert_eq!(
            snapshot.query_ray(RenderSpatialRay::new(Vec3::ZERO, Vec3::NEG_Z, 10.0)),
            RenderVisibleSpatialQueryResult {
                entities: vec![7],
                stats: RenderVisibleSpatialQueryStats {
                    visited_node_count: 2,
                    candidate_count: 1,
                    hit_count: 1,
                },
            }
        );
    }
}
