use zircon_runtime::asset::NavMeshAsset;
use zircon_runtime::core::framework::navigation::{
    NavPathPoint, NavPathQuery, NavPathResult, NavPathStatus, NavQueryFilter, AREA_WALKABLE,
};
use zircon_runtime::core::math::Real;

use crate::ffi;

use super::geometry::{distance, nearest_allowed_polygon, polygon_centroid};
use super::graph::{build_polygon_graph, shortest_polygon_route, EdgeTraversal, RouteStep};

pub(crate) fn find_path(
    asset: &NavMeshAsset,
    query: &NavPathQuery,
    filter: &NavQueryFilter,
) -> NavPathResult {
    let Some(start_polygon) = nearest_allowed_polygon(asset, query.start, query.area_mask, filter)
    else {
        return NavPathResult::no_path();
    };
    let Some(end_polygon) = nearest_allowed_polygon(asset, query.end, query.area_mask, filter)
    else {
        return NavPathResult::no_path();
    };
    let graph = build_polygon_graph(asset, query.area_mask, filter, true);
    let Some(route) = shortest_polygon_route(&graph, start_polygon, end_polygon) else {
        return NavPathResult::no_path();
    };
    let points = path_points_from_route(asset, query.start, query.end, &route);
    let length = native_polyline_length(&points);
    NavPathResult {
        status: NavPathStatus::Complete,
        points,
        length,
        visited_nodes: route.len().max(1),
    }
}

fn path_points_from_route(
    asset: &NavMeshAsset,
    start: [Real; 3],
    end: [Real; 3],
    route: &[RouteStep],
) -> Vec<NavPathPoint> {
    let mut points = vec![NavPathPoint {
        position: start,
        area: route
            .first()
            .and_then(|step| asset.polygons.get(step.polygon))
            .map(|polygon| polygon.area)
            .unwrap_or(AREA_WALKABLE),
        off_mesh_link_id: None,
        flags: Vec::new(),
    }];
    for index in 1..route.len() {
        let step = &route[index];
        if let Some(EdgeTraversal::OffMeshLink {
            id,
            start,
            end,
            area,
        }) = &step.traversal_from_previous
        {
            points.push(NavPathPoint {
                position: *start,
                area: *area,
                off_mesh_link_id: Some(*id),
                flags: vec!["off_mesh_link".to_string()],
            });
            points.push(NavPathPoint {
                position: *end,
                area: *area,
                off_mesh_link_id: Some(*id),
                flags: vec!["off_mesh_link".to_string()],
            });
        } else if index + 1 < route.len() {
            if let Some(centroid) = asset
                .polygons
                .get(step.polygon)
                .and_then(|polygon| polygon_centroid(asset, polygon))
            {
                points.push(NavPathPoint {
                    position: centroid,
                    area: asset.polygons[step.polygon].area,
                    off_mesh_link_id: None,
                    flags: Vec::new(),
                });
            }
        }
    }
    points.push(NavPathPoint {
        position: end,
        area: route
            .last()
            .and_then(|step| asset.polygons.get(step.polygon))
            .map(|polygon| polygon.area)
            .unwrap_or(AREA_WALKABLE),
        off_mesh_link_id: None,
        flags: Vec::new(),
    });
    points
}

fn polyline_length(points: &[NavPathPoint]) -> Real {
    points
        .windows(2)
        .map(|window| distance(window[0].position, window[1].position))
        .sum()
}

fn native_polyline_length(points: &[NavPathPoint]) -> Real {
    let mut coordinates = Vec::with_capacity(points.len() * 3);
    for point in points {
        coordinates.extend_from_slice(&point.position);
    }
    let length =
        unsafe { ffi::zr_nav_recast_polyline_length(coordinates.as_ptr(), points.len() as u64) };
    if length.is_finite() {
        length
    } else {
        polyline_length(points)
    }
}
