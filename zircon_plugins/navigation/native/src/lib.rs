use zircon_runtime::asset::{NavMeshAsset, NavMeshLinkAsset, NavMeshPolygonAsset};
use zircon_runtime::core::framework::navigation::{
    NavPathPoint, NavPathQuery, NavPathResult, NavPathStatus, NavRaycastQuery, NavRaycastResult,
    NavSampleHit, NavSampleQuery, NavigationError, NavigationErrorKind, AREA_WALKABLE,
};
use zircon_runtime::core::math::Real;

mod bake;
mod detour;
mod ffi;
mod tile_cache;

pub use bake::{RecastBakeInput, RecastBakeMeshInput};
pub use tile_cache::{RecastNavigationObstacle, RecastNavigationObstacleShape};

pub fn native_backend_version() -> u32 {
    unsafe { ffi::zr_nav_recast_bridge_version() }
}

pub fn native_runtime_modules_available() -> bool {
    unsafe { ffi::zr_nav_recast_runtime_modules_smoke() == 1 }
}

#[derive(Clone, Debug, Default)]
pub struct RecastBackend;

impl RecastBackend {
    pub fn find_path(
        &self,
        asset: &NavMeshAsset,
        query: &NavPathQuery,
    ) -> Result<NavPathResult, NavigationError> {
        validate_query_agent(asset, &query.agent_type)?;
        if asset.is_empty() {
            return Ok(NavPathResult::no_path());
        }
        if let Some(result) = detour::find_path(asset, query) {
            return Ok(result);
        }
        let Some(start_polygon) = nearest_allowed_polygon(asset, query.start, query.area_mask)
        else {
            return Ok(NavPathResult::no_path());
        };
        let Some(end_polygon) = nearest_allowed_polygon(asset, query.end, query.area_mask) else {
            return Ok(NavPathResult::no_path());
        };
        let graph = build_polygon_graph(asset, query.area_mask, true);
        let Some(route) = shortest_polygon_route(&graph, start_polygon, end_polygon) else {
            return Ok(NavPathResult::no_path());
        };
        let points = path_points_from_route(asset, query.start, query.end, &route);
        let length = native_polyline_length(&points);
        Ok(NavPathResult {
            status: NavPathStatus::Complete,
            points,
            length,
            visited_nodes: route.len().max(1),
        })
    }

    pub fn find_path_with_obstacles(
        &self,
        asset: &NavMeshAsset,
        query: &NavPathQuery,
        obstacles: &[RecastNavigationObstacle],
    ) -> Result<NavPathResult, NavigationError> {
        if obstacles.is_empty() {
            return self.find_path(asset, query);
        }
        validate_query_agent(asset, &query.agent_type)?;
        if asset.is_empty() {
            return Ok(NavPathResult::no_path());
        }
        if let Some(result) = tile_cache::find_path(asset, query, obstacles) {
            return Ok(result);
        }
        self.find_path(asset, query)
    }

    pub fn sample_position(
        &self,
        asset: &NavMeshAsset,
        query: &NavSampleQuery,
    ) -> Result<Option<NavSampleHit>, NavigationError> {
        validate_query_agent(asset, &query.agent_type)?;
        if asset.is_empty() {
            return Ok(None);
        }
        if let Some(result) = detour::sample_position(asset, query) {
            return Ok(result);
        }
        let Some((polygon, position, distance)) =
            nearest_allowed_polygon_sample(asset, query.position, query.area_mask)
        else {
            return Ok(None);
        };
        let inside_extents =
            distance <= query.extents[0].max(query.extents[1]).max(query.extents[2]);
        Ok(inside_extents.then_some(NavSampleHit {
            position,
            distance,
            area: asset
                .polygons
                .get(polygon)
                .map(|polygon| polygon.area)
                .unwrap_or(AREA_WALKABLE),
        }))
    }

    pub fn raycast(
        &self,
        asset: &NavMeshAsset,
        query: &NavRaycastQuery,
    ) -> Result<NavRaycastResult, NavigationError> {
        validate_query_agent(asset, &query.agent_type)?;
        if asset.is_empty() {
            return Ok(NavRaycastResult {
                hit: true,
                position: query.start,
                normal: [0.0, 1.0, 0.0],
                distance: 0.0,
            });
        }
        let Some(start_polygon) = containing_allowed_polygon(asset, query.start, query.area_mask)
        else {
            return Ok(NavRaycastResult {
                hit: true,
                position: query.start,
                normal: [0.0, 1.0, 0.0],
                distance: 0.0,
            });
        };
        if let Some(result) = detour::raycast(asset, query) {
            return Ok(result);
        }
        if let Some(hit) = first_straight_line_block(asset, query, start_polygon) {
            return Ok(NavRaycastResult {
                hit: true,
                position: hit,
                normal: [0.0, 1.0, 0.0],
                distance: distance(query.start, hit),
            });
        }
        Ok(NavRaycastResult {
            hit: false,
            position: query.end,
            normal: [0.0, 1.0, 0.0],
            distance: distance(query.start, query.end),
        })
    }
}

fn validate_query_agent(asset: &NavMeshAsset, agent_type: &str) -> Result<(), NavigationError> {
    if asset.agent_type == agent_type {
        return Ok(());
    }
    Err(NavigationError::new(
        NavigationErrorKind::InvalidConfiguration,
        format!(
            "query agent type `{agent_type}` does not match navmesh agent type `{}`",
            asset.agent_type
        ),
    ))
}

fn area_allowed(asset: &NavMeshAsset, mask: u64, area: u8) -> bool {
    if area >= 64 || (mask & (1_u64 << area)) == 0 {
        return false;
    }
    asset
        .area_costs
        .iter()
        .find(|cost| cost.area == area)
        .map(|cost| cost.walkable)
        .unwrap_or(area != 0)
}

fn area_cost(asset: &NavMeshAsset, area: u8) -> Real {
    asset
        .area_costs
        .iter()
        .find(|cost| cost.area == area)
        .map(|cost| cost.cost.max(0.001))
        .unwrap_or(1.0)
}

#[derive(Clone, Debug)]
struct PolygonEdge {
    to: usize,
    cost: Real,
    traversal: EdgeTraversal,
}

#[derive(Clone, Debug)]
enum EdgeTraversal {
    SharedEdge,
    OffMeshLink {
        start: [Real; 3],
        end: [Real; 3],
        area: u8,
    },
}

#[derive(Clone, Debug)]
struct RouteStep {
    polygon: usize,
    traversal_from_previous: Option<EdgeTraversal>,
}

fn nearest_allowed_polygon(asset: &NavMeshAsset, position: [Real; 3], mask: u64) -> Option<usize> {
    let mut best_inside = None;
    let mut best_distance = Real::INFINITY;
    for (index, polygon) in asset.polygons.iter().enumerate() {
        if !area_allowed(asset, mask, polygon.area) {
            continue;
        }
        if point_in_polygon_xz(asset, polygon, position) {
            return Some(index);
        }
        if let Some(centroid) = polygon_centroid(asset, polygon) {
            let distance = distance_xz(position, centroid);
            if distance < best_distance {
                best_distance = distance;
                best_inside = Some(index);
            }
        }
    }
    best_inside
}

fn containing_allowed_polygon(
    asset: &NavMeshAsset,
    position: [Real; 3],
    mask: u64,
) -> Option<usize> {
    asset
        .polygons
        .iter()
        .enumerate()
        .find(|(_, polygon)| {
            area_allowed(asset, mask, polygon.area) && point_in_polygon_xz(asset, polygon, position)
        })
        .map(|(index, _)| index)
}

fn build_polygon_graph(
    asset: &NavMeshAsset,
    mask: u64,
    include_off_mesh_links: bool,
) -> Vec<Vec<PolygonEdge>> {
    let mut graph = vec![Vec::new(); asset.polygons.len()];
    for (left_index, left) in asset.polygons.iter().enumerate() {
        if !area_allowed(asset, mask, left.area) {
            continue;
        }
        for (right_index, right) in asset.polygons.iter().enumerate().skip(left_index + 1) {
            if !area_allowed(asset, mask, right.area) {
                continue;
            }
            if shared_vertex_count(asset, left, right) >= 2 {
                let cost = polygon_edge_cost(asset, left_index, right_index, None);
                graph[left_index].push(PolygonEdge {
                    to: right_index,
                    cost,
                    traversal: EdgeTraversal::SharedEdge,
                });
                graph[right_index].push(PolygonEdge {
                    to: left_index,
                    cost,
                    traversal: EdgeTraversal::SharedEdge,
                });
            }
        }
    }
    if include_off_mesh_links {
        for link in &asset.off_mesh_links {
            add_off_mesh_link_edges(asset, mask, &mut graph, link);
        }
    }
    graph
}

fn add_off_mesh_link_edges(
    asset: &NavMeshAsset,
    mask: u64,
    graph: &mut [Vec<PolygonEdge>],
    link: &NavMeshLinkAsset,
) {
    if !area_allowed(asset, mask, link.area) {
        return;
    }
    let Some(start_polygon) = nearest_allowed_polygon(asset, link.start, mask) else {
        return;
    };
    let Some(end_polygon) = nearest_allowed_polygon(asset, link.end, mask) else {
        return;
    };
    if start_polygon == end_polygon {
        return;
    }
    let cost = link
        .cost_override
        .unwrap_or_else(|| distance(link.start, link.end) * area_cost(asset, link.area));
    graph[start_polygon].push(PolygonEdge {
        to: end_polygon,
        cost,
        traversal: EdgeTraversal::OffMeshLink {
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
                start: link.end,
                end: link.start,
                area: link.area,
            },
        });
    }
}

fn shortest_polygon_route(
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
        flags: Vec::new(),
    }];
    for index in 1..route.len() {
        let step = &route[index];
        if let Some(EdgeTraversal::OffMeshLink { start, end, area }) = &step.traversal_from_previous
        {
            points.push(NavPathPoint {
                position: *start,
                area: *area,
                flags: vec!["off_mesh_link".to_string()],
            });
            points.push(NavPathPoint {
                position: *end,
                area: *area,
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
        flags: Vec::new(),
    });
    points
}

fn polygon_edge_cost(
    asset: &NavMeshAsset,
    left_index: usize,
    right_index: usize,
    override_cost: Option<Real>,
) -> Real {
    override_cost.unwrap_or_else(|| {
        let left = asset
            .polygons
            .get(left_index)
            .and_then(|polygon| polygon_centroid(asset, polygon));
        let right = asset
            .polygons
            .get(right_index)
            .and_then(|polygon| polygon_centroid(asset, polygon));
        left.zip(right)
            .map(|(left, right)| {
                let right_area = asset
                    .polygons
                    .get(right_index)
                    .map(|polygon| polygon.area)
                    .unwrap_or(AREA_WALKABLE);
                distance(left, right) * area_cost(asset, right_area)
            })
            .unwrap_or(1.0)
    })
}

fn nearest_allowed_polygon_sample(
    asset: &NavMeshAsset,
    position: [Real; 3],
    mask: u64,
) -> Option<(usize, [Real; 3], Real)> {
    let mut best = None;
    let mut best_distance = Real::INFINITY;
    for (index, polygon) in asset.polygons.iter().enumerate() {
        if !area_allowed(asset, mask, polygon.area) {
            continue;
        }
        if let Some(sample) = closest_point_on_polygon_xz(asset, polygon, position) {
            let distance = distance(position, sample);
            if distance < best_distance {
                best_distance = distance;
                best = Some((index, sample, distance));
            }
        }
    }
    best
}

fn closest_point_on_polygon_xz(
    asset: &NavMeshAsset,
    polygon: &NavMeshPolygonAsset,
    point: [Real; 3],
) -> Option<[Real; 3]> {
    let indices = polygon_indices(asset, polygon);
    let mut best = None;
    let mut best_distance = Real::INFINITY;
    for triangle in indices.chunks(3).filter(|triangle| triangle.len() == 3) {
        let Some(sample) = closest_point_on_triangle_xz(asset, triangle, point) else {
            continue;
        };
        let distance = distance(point, sample);
        if distance < best_distance {
            best_distance = distance;
            best = Some(sample);
        }
    }
    best
}

fn closest_point_on_triangle_xz(
    asset: &NavMeshAsset,
    indices: &[usize],
    point: [Real; 3],
) -> Option<[Real; 3]> {
    let a = asset.vertices.get(indices[0]).copied()?;
    let b = asset.vertices.get(indices[1]).copied()?;
    let c = asset.vertices.get(indices[2]).copied()?;
    if point_in_triangle_xz(asset, indices, point) {
        let weights = barycentric_xz(a, b, c, point)?;
        return Some(interpolate_triangle(a, b, c, weights));
    }
    [
        closest_point_on_segment_xz(a, b, point),
        closest_point_on_segment_xz(b, c, point),
        closest_point_on_segment_xz(c, a, point),
    ]
    .into_iter()
    .min_by(|left, right| distance(point, *left).total_cmp(&distance(point, *right)))
}

fn closest_point_on_segment_xz(a: [Real; 3], b: [Real; 3], point: [Real; 3]) -> [Real; 3] {
    let ab = [b[0] - a[0], b[2] - a[2]];
    let ap = [point[0] - a[0], point[2] - a[2]];
    let length_sq = ab[0] * ab[0] + ab[1] * ab[1];
    let t = if length_sq <= Real::EPSILON {
        0.0
    } else {
        ((ap[0] * ab[0] + ap[1] * ab[1]) / length_sq).clamp(0.0, 1.0)
    };
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn shared_vertex_count(
    asset: &NavMeshAsset,
    left: &NavMeshPolygonAsset,
    right: &NavMeshPolygonAsset,
) -> usize {
    let left_indices = unique_polygon_indices(asset, left);
    let right_indices = unique_polygon_indices(asset, right);
    left_indices
        .iter()
        .filter(|index| right_indices.contains(index))
        .count()
}

fn unique_polygon_indices(asset: &NavMeshAsset, polygon: &NavMeshPolygonAsset) -> Vec<usize> {
    let mut unique = Vec::new();
    for index in polygon_indices(asset, polygon) {
        if !unique.contains(&index) {
            unique.push(index);
        }
    }
    unique
}

fn point_in_polygon_xz(
    asset: &NavMeshAsset,
    polygon: &NavMeshPolygonAsset,
    point: [Real; 3],
) -> bool {
    let indices = polygon_indices(asset, polygon);
    indices
        .chunks(3)
        .any(|triangle| triangle.len() == 3 && point_in_triangle_xz(asset, triangle, point))
}

fn point_in_triangle_xz(asset: &NavMeshAsset, indices: &[usize], point: [Real; 3]) -> bool {
    let Some(a) = asset.vertices.get(indices[0]).copied() else {
        return false;
    };
    let Some(b) = asset.vertices.get(indices[1]).copied() else {
        return false;
    };
    let Some(c) = asset.vertices.get(indices[2]).copied() else {
        return false;
    };
    let Some((u, v, w)) = barycentric_xz(a, b, c, point) else {
        return false;
    };
    u >= -Real::EPSILON && v >= -Real::EPSILON && w >= -Real::EPSILON
}

fn barycentric_xz(
    a: [Real; 3],
    b: [Real; 3],
    c: [Real; 3],
    point: [Real; 3],
) -> Option<(Real, Real, Real)> {
    let p = [point[0], point[2]];
    let a = [a[0], a[2]];
    let b = [b[0], b[2]];
    let c = [c[0], c[2]];
    let denominator = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
    if denominator.abs() <= Real::EPSILON {
        return None;
    }
    let u = ((b[1] - c[1]) * (p[0] - c[0]) + (c[0] - b[0]) * (p[1] - c[1])) / denominator;
    let v = ((c[1] - a[1]) * (p[0] - c[0]) + (a[0] - c[0]) * (p[1] - c[1])) / denominator;
    let w = 1.0 - u - v;
    Some((u, v, w))
}

fn interpolate_triangle(
    a: [Real; 3],
    b: [Real; 3],
    c: [Real; 3],
    (u, v, w): (Real, Real, Real),
) -> [Real; 3] {
    [
        a[0] * u + b[0] * v + c[0] * w,
        a[1] * u + b[1] * v + c[1] * w,
        a[2] * u + b[2] * v + c[2] * w,
    ]
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
            let graph = build_polygon_graph(asset, query.area_mask, false);
            if shortest_polygon_route(&graph, previous_polygon, current_polygon).is_none() {
                return Some(point);
            }
            previous_polygon = current_polygon;
        }
    }
    None
}

fn lerp(from: [Real; 3], to: [Real; 3], t: Real) -> [Real; 3] {
    [
        from[0] + (to[0] - from[0]) * t,
        from[1] + (to[1] - from[1]) * t,
        from[2] + (to[2] - from[2]) * t,
    ]
}

fn polygon_centroid(asset: &NavMeshAsset, polygon: &NavMeshPolygonAsset) -> Option<[Real; 3]> {
    let indices = polygon_indices(asset, polygon);
    let mut sum = [0.0, 0.0, 0.0];
    let mut count = 0.0;
    for index in indices {
        let vertex = asset.vertices.get(index)?;
        sum[0] += vertex[0];
        sum[1] += vertex[1];
        sum[2] += vertex[2];
        count += 1.0;
    }
    (count > 0.0).then_some([sum[0] / count, sum[1] / count, sum[2] / count])
}

fn polygon_indices(asset: &NavMeshAsset, polygon: &NavMeshPolygonAsset) -> Vec<usize> {
    let start = polygon.first_index as usize;
    let end = start.saturating_add(polygon.index_count as usize);
    asset.indices[start.min(asset.indices.len())..end.min(asset.indices.len())]
        .iter()
        .map(|index| *index as usize)
        .collect()
}

fn distance(from: [Real; 3], to: [Real; 3]) -> Real {
    let delta = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
    (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt()
}

fn distance_xz(from: [Real; 3], to: [Real; 3]) -> Real {
    let delta = [to[0] - from[0], to[2] - from[2]];
    (delta[0] * delta[0] + delta[1] * delta[1]).sqrt()
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

#[cfg(test)]
mod tests;
