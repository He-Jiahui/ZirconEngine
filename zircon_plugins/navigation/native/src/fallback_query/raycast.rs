use zircon_runtime::core::framework::navigation::NavMeshAsset;
use zircon_runtime::core::framework::navigation::{
    NavQueryFilter, NavRaycastQuery, NavRaycastResult,
};
use zircon_runtime::core::math::Real;

use super::geometry::{area_allowed, distance, lerp, point_in_polygon_xz};
use super::graph::{build_polygon_graph, shortest_polygon_route};

pub(crate) fn blocked_raycast_result(query: &NavRaycastQuery) -> NavRaycastResult {
    NavRaycastResult {
        hit: true,
        position: query.start,
        normal: [0.0, 1.0, 0.0],
        distance: 0.0,
    }
}

pub(crate) fn containing_allowed_polygon(
    asset: &NavMeshAsset,
    position: [Real; 3],
    mask: u64,
) -> Option<usize> {
    asset
        .polygons
        .iter()
        .enumerate()
        .find(|(_, polygon)| {
            area_allowed(asset, mask, &NavQueryFilter::default(), polygon.area)
                && point_in_polygon_xz(asset, polygon, position)
        })
        .map(|(index, _)| index)
}

pub(crate) fn raycast_from_polygon(
    asset: &NavMeshAsset,
    query: &NavRaycastQuery,
    start_polygon: usize,
) -> NavRaycastResult {
    if let Some(hit) = first_straight_line_block(asset, query, start_polygon) {
        return NavRaycastResult {
            hit: true,
            position: hit,
            normal: [0.0, 1.0, 0.0],
            distance: distance(query.start, hit),
        };
    }
    NavRaycastResult {
        hit: false,
        position: query.end,
        normal: [0.0, 1.0, 0.0],
        distance: distance(query.start, query.end),
    }
}

fn first_straight_line_block(
    asset: &NavMeshAsset,
    query: &NavRaycastQuery,
    start_polygon: usize,
) -> Option<[Real; 3]> {
    const STEPS: usize = 32;
    let mut previous_polygon = start_polygon;
    for step in 1..=STEPS {
        let t = step as Real / STEPS as Real;
        let point = lerp(query.start, query.end, t);
        let Some(current_polygon) = containing_allowed_polygon(asset, point, query.area_mask)
        else {
            return Some(point);
        };
        if current_polygon != previous_polygon {
            let graph =
                build_polygon_graph(asset, query.area_mask, &NavQueryFilter::default(), false);
            if shortest_polygon_route(&graph, previous_polygon, current_polygon).is_none() {
                return Some(point);
            }
            previous_polygon = current_polygon;
        }
    }
    None
}
