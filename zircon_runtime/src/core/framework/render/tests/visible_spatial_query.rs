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
