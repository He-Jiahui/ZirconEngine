use zircon_runtime::core::framework::navigation::NavQueryFilter;
use zircon_runtime::core::framework::navigation::AREA_WALKABLE;
use zircon_runtime::core::framework::navigation::{NavMeshAsset, NavMeshLinkAsset};
use zircon_runtime::core::math::Real;

use super::geometry::{
    area_allowed, area_cost, distance, nearest_allowed_polygon, polygon_centroid,
    shared_vertex_count,
};

#[derive(Clone, Debug)]
pub(super) struct PolygonEdge {
    to: usize,
    cost: Real,
    traversal: EdgeTraversal,
}

#[derive(Clone, Debug)]
pub(super) enum EdgeTraversal {
    SharedEdge,
    OffMeshLink {
        id: u32,
        start: [Real; 3],
        end: [Real; 3],
        area: u8,
    },
}

#[derive(Clone, Debug)]
pub(super) struct RouteStep {
    pub(super) polygon: usize,
    pub(super) traversal_from_previous: Option<EdgeTraversal>,
}

pub(super) fn build_polygon_graph(
    asset: &NavMeshAsset,
    mask: u64,
    filter: &NavQueryFilter,
    include_off_mesh_links: bool,
) -> Vec<Vec<PolygonEdge>> {
    let mut graph = vec![Vec::new(); asset.polygons.len()];
    for (left_index, left) in asset.polygons.iter().enumerate() {
        if !area_allowed(asset, mask, filter, left.area) {
            continue;
        }
        for (right_index, right) in asset.polygons.iter().enumerate().skip(left_index + 1) {
            if !area_allowed(asset, mask, filter, right.area) {
                continue;
            }
            if shared_vertex_count(asset, left, right) >= 2 {
                let left_to_right = polygon_edge_cost(asset, filter, left_index, right_index, None);
                let right_to_left = polygon_edge_cost(asset, filter, right_index, left_index, None);
                graph[left_index].push(PolygonEdge {
                    to: right_index,
                    cost: left_to_right,
                    traversal: EdgeTraversal::SharedEdge,
                });
                graph[right_index].push(PolygonEdge {
                    to: left_index,
                    cost: right_to_left,
                    traversal: EdgeTraversal::SharedEdge,
                });
            }
        }
    }
    if include_off_mesh_links {
        for link in &asset.off_mesh_links {
            add_off_mesh_link_edges(asset, mask, filter, &mut graph, link);
        }
    }
    graph
}

fn add_off_mesh_link_edges(
    asset: &NavMeshAsset,
    mask: u64,
    filter: &NavQueryFilter,
    graph: &mut [Vec<PolygonEdge>],
    link: &NavMeshLinkAsset,
) {
    if !area_allowed(asset, mask, filter, link.area) {
        return;
    }
    let Some(start_polygon) = nearest_allowed_polygon(asset, link.start, mask, filter) else {
        return;
    };
    let Some(end_polygon) = nearest_allowed_polygon(asset, link.end, mask, filter) else {
        return;
    };
    if start_polygon == end_polygon {
        return;
    }
    let cost = link
        .cost_override
        .unwrap_or_else(|| distance(link.start, link.end) * area_cost(filter, link.area));
    graph[start_polygon].push(PolygonEdge {
        to: end_polygon,
        cost,
        traversal: EdgeTraversal::OffMeshLink {
            id: link.id,
            start: link.start,
            end: link.end,
            area: link.area,
        },
    });
    if link.bidirectional {
        graph[end_polygon].push(PolygonEdge {
            to: start_polygon,
            cost,
            traversal: EdgeTraversal::OffMeshLink {
                id: link.id,
                start: link.end,
                end: link.start,
                area: link.area,
            },
        });
    }
}

pub(super) fn shortest_polygon_route(
    graph: &[Vec<PolygonEdge>],
    start: usize,
    end: usize,
) -> Option<Vec<RouteStep>> {
    if start >= graph.len() || end >= graph.len() {
        return None;
    }
    let mut distances = vec![Real::INFINITY; graph.len()];
    let mut visited = vec![false; graph.len()];
    let mut parents: Vec<Option<(usize, EdgeTraversal)>> = vec![None; graph.len()];
    distances[start] = 0.0;

    loop {
        let current = (0..graph.len())
            .filter(|index| !visited[*index])
            .min_by(|left, right| distances[*left].total_cmp(&distances[*right]))?;
        if distances[current] == Real::INFINITY {
            return None;
        }
        if current == end {
            break;
        }
        visited[current] = true;
        for edge in &graph[current] {
            let candidate = distances[current] + edge.cost;
            if candidate < distances[edge.to] {
                distances[edge.to] = candidate;
                parents[edge.to] = Some((current, edge.traversal.clone()));
            }
        }
    }

    let mut reversed = Vec::new();
    let mut current = end;
    reversed.push(RouteStep {
        polygon: current,
        traversal_from_previous: None,
    });
    while current != start {
        let (parent, traversal) = parents[current].clone()?;
        reversed.last_mut().unwrap().traversal_from_previous = Some(traversal);
        current = parent;
        reversed.push(RouteStep {
            polygon: current,
            traversal_from_previous: None,
        });
    }
    reversed.reverse();
    Some(reversed)
}

fn polygon_edge_cost(
    asset: &NavMeshAsset,
    filter: &NavQueryFilter,
    source_index: usize,
    target_index: usize,
    override_cost: Option<Real>,
) -> Real {
    override_cost.unwrap_or_else(|| {
        let source = asset
            .polygons
            .get(source_index)
            .and_then(|polygon| polygon_centroid(asset, polygon));
        let target = asset
            .polygons
            .get(target_index)
            .and_then(|polygon| polygon_centroid(asset, polygon));
        source
            .zip(target)
            .map(|(source, target)| {
                let source_area = asset
                    .polygons
                    .get(source_index)
                    .map(|polygon| polygon.area)
                    .unwrap_or(AREA_WALKABLE);
                distance(source, target) * area_cost(filter, source_area)
            })
            .unwrap_or(1.0)
    })
}
