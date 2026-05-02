use zircon_runtime::asset::{NavMeshAsset, NavMeshLinkAsset, NavMeshPolygonAsset};
use zircon_runtime::core::framework::navigation::{
    NavAreaId, NavPathPoint, NavPathQuery, NavPathResult, NavPathStatus, NavRaycastQuery,
    NavRaycastResult, NavSampleHit, NavSampleQuery, NavigationError, NavigationErrorKind,
    AREA_WALKABLE,
};
use zircon_runtime::core::math::Real;

mod ffi {
    extern "C" {
        pub fn zr_nav_recast_bridge_version() -> u32;
        pub fn zr_nav_recast_runtime_modules_smoke() -> u32;
        pub fn zr_nav_recast_polyline_length(xyz: *const f32, point_count: u64) -> f32;
    }
}

pub fn native_backend_version() -> u32 {
    unsafe { ffi::zr_nav_recast_bridge_version() }
}

pub fn native_runtime_modules_available() -> bool {
    unsafe { ffi::zr_nav_recast_runtime_modules_smoke() == 1 }
}

#[derive(Clone, Debug, Default)]
pub struct RecastBackend;

#[derive(Clone, Debug, PartialEq)]
pub struct RecastBakeInput {
    pub agent_type: String,
    pub source_vertices: usize,
    pub source_triangles: usize,
    pub half_extent: Real,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecastBakeMeshInput {
    pub agent_type: String,
    pub vertices: Vec<[Real; 3]>,
    pub indices: Vec<u32>,
    pub triangle_areas: Vec<NavAreaId>,
    pub default_area: NavAreaId,
}

impl RecastBackend {
    pub fn bake_simple_surface(
        &self,
        input: RecastBakeInput,
    ) -> Result<NavMeshAsset, NavigationError> {
        if input.half_extent <= 0.0 || !input.half_extent.is_finite() {
            return Err(NavigationError::new(
                NavigationErrorKind::InvalidConfiguration,
                "navigation bake half_extent must be positive and finite",
            ));
        }
        Ok(NavMeshAsset::simple_quad(
            input.agent_type,
            input.half_extent,
        ))
    }

    pub fn bake_triangle_mesh(
        &self,
        input: RecastBakeMeshInput,
    ) -> Result<NavMeshAsset, NavigationError> {
        if input.vertices.is_empty() || input.indices.len() < 3 {
            return Err(NavigationError::new(
                NavigationErrorKind::BakeFailed,
                "navigation bake source mesh has no triangles",
            ));
        }
        let asset = NavMeshAsset::from_triangle_mesh_with_areas(
            input.agent_type,
            input.vertices,
            input.indices,
            input.triangle_areas,
            input.default_area,
        );
        if asset.is_empty() {
            return Err(NavigationError::new(
                NavigationErrorKind::BakeFailed,
                "navigation bake source mesh has no valid indexed triangles",
            ));
        }
        Ok(asset)
    }

    pub fn find_path(
        &self,
        asset: &NavMeshAsset,
        query: &NavPathQuery,
    ) -> Result<NavPathResult, NavigationError> {
        if asset.is_empty() {
            return Ok(NavPathResult::no_path());
        }
        let Some(start_polygon) = nearest_allowed_polygon(asset, query.start, query.area_mask)
        else {
            return Ok(NavPathResult::no_path());
        };
        let Some(end_polygon) = nearest_allowed_polygon(asset, query.end, query.area_mask) else {
            return Ok(NavPathResult::no_path());
        };
        let graph = build_polygon_graph(asset, query.area_mask);
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

    pub fn sample_position(
        &self,
        asset: &NavMeshAsset,
        query: &NavSampleQuery,
    ) -> Result<Option<NavSampleHit>, NavigationError> {
        if asset.is_empty() {
            return Ok(None);
        }
        let Some((min, max)) = asset_bounds(asset) else {
            return Ok(None);
        };
        let position = [
            query.position[0].clamp(min[0], max[0]),
            query.position[1].clamp(min[1], max[1]),
            query.position[2].clamp(min[2], max[2]),
        ];
        let distance = distance(query.position, position);
        let inside_extents =
            distance <= query.extents[0].max(query.extents[1]).max(query.extents[2]);
        Ok(inside_extents.then_some(NavSampleHit {
            position,
            distance,
            area: nearest_allowed_polygon(asset, position, query.area_mask)
                .and_then(|polygon| asset.polygons.get(polygon))
                .map(|polygon| polygon.area)
                .unwrap_or(AREA_WALKABLE),
        }))
    }

    pub fn raycast(
        &self,
        asset: &NavMeshAsset,
        query: &NavRaycastQuery,
    ) -> Result<NavRaycastResult, NavigationError> {
        if asset.is_empty() {
            return Ok(NavRaycastResult {
                hit: true,
                position: query.start,
                normal: [0.0, 1.0, 0.0],
                distance: 0.0,
            });
        }
        let path = self.find_path(
            asset,
            &NavPathQuery {
                nav_mesh: query.nav_mesh,
                start: query.start,
                end: query.end,
                agent_type: query.agent_type.clone(),
                area_mask: query.area_mask,
            },
        )?;
        if path.status == NavPathStatus::NoPath {
            return Ok(NavRaycastResult {
                hit: true,
                position: query.start,
                normal: [0.0, 1.0, 0.0],
                distance: 0.0,
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

fn area_allowed(mask: u64, area: u8) -> bool {
    area < 64 && (mask & (1_u64 << area)) != 0
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
        if !area_allowed(mask, polygon.area) {
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

fn build_polygon_graph(asset: &NavMeshAsset, mask: u64) -> Vec<Vec<PolygonEdge>> {
    let mut graph = vec![Vec::new(); asset.polygons.len()];
    for (left_index, left) in asset.polygons.iter().enumerate() {
        if !area_allowed(mask, left.area) {
            continue;
        }
        for (right_index, right) in asset.polygons.iter().enumerate().skip(left_index + 1) {
            if !area_allowed(mask, right.area) {
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
    for link in &asset.off_mesh_links {
        add_off_mesh_link_edges(asset, mask, &mut graph, link);
    }
    graph
}

fn add_off_mesh_link_edges(
    asset: &NavMeshAsset,
    mask: u64,
    graph: &mut [Vec<PolygonEdge>],
    link: &NavMeshLinkAsset,
) {
    if !area_allowed(mask, link.area) {
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
        .unwrap_or_else(|| distance(link.start, link.end));
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
            .map(|(left, right)| distance(left, right))
            .unwrap_or(1.0)
    })
}

fn shared_vertex_count(
    asset: &NavMeshAsset,
    left: &NavMeshPolygonAsset,
    right: &NavMeshPolygonAsset,
) -> usize {
    let left_indices = polygon_indices(asset, left);
    let right_indices = polygon_indices(asset, right);
    left_indices
        .iter()
        .filter(|index| right_indices.contains(index))
        .count()
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
    let p = [point[0], point[2]];
    let a = [a[0], a[2]];
    let b = [b[0], b[2]];
    let c = [c[0], c[2]];
    let denominator = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
    if denominator.abs() <= Real::EPSILON {
        return false;
    }
    let u = ((b[1] - c[1]) * (p[0] - c[0]) + (c[0] - b[0]) * (p[1] - c[1])) / denominator;
    let v = ((c[1] - a[1]) * (p[0] - c[0]) + (a[0] - c[0]) * (p[1] - c[1])) / denominator;
    let w = 1.0 - u - v;
    u >= -Real::EPSILON && v >= -Real::EPSILON && w >= -Real::EPSILON
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

fn asset_bounds(asset: &NavMeshAsset) -> Option<([Real; 3], [Real; 3])> {
    let mut vertices = asset.vertices.iter();
    let first = *vertices.next()?;
    let mut min = first;
    let mut max = first;
    for vertex in vertices {
        for axis in 0..3 {
            min[axis] = min[axis].min(vertex[axis]);
            max[axis] = max[axis].max(vertex[axis]);
        }
    }
    Some((min, max))
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
mod tests {
    use zircon_runtime::core::framework::navigation::{
        NavPathQuery, NavSampleQuery, AREA_JUMP, AREA_WALKABLE, DEFAULT_AREA_MASK,
    };

    use super::*;

    #[test]
    fn native_recast_detour_modules_are_linked() {
        assert_eq!(native_backend_version(), 1);
        assert!(native_runtime_modules_available());
    }

    #[test]
    fn simple_surface_path_uses_baked_asset() {
        let backend = RecastBackend;
        let asset = backend
            .bake_simple_surface(RecastBakeInput {
                agent_type: "humanoid".to_string(),
                source_vertices: 4,
                source_triangles: 2,
                half_extent: 5.0,
            })
            .unwrap();

        let result = backend
            .find_path(&asset, &NavPathQuery::new([0.0, 0.0, 0.0], [3.0, 0.0, 4.0]))
            .unwrap();

        assert_eq!(result.status, NavPathStatus::Complete);
        assert_eq!(result.length, 5.0);
        assert_eq!(result.points.len(), 2);
    }

    #[test]
    fn area_mask_can_block_walkable_area() {
        let backend = RecastBackend;
        let asset = NavMeshAsset::simple_quad("humanoid", 5.0);
        let mut query = NavPathQuery::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        query.area_mask = DEFAULT_AREA_MASK & !(1_u64 << AREA_WALKABLE);

        let result = backend.find_path(&asset, &query).unwrap();

        assert_eq!(result.status, NavPathStatus::NoPath);
    }

    #[test]
    fn disconnected_polygons_return_no_path_without_link() {
        let backend = RecastBackend;
        let asset = two_island_asset(false);

        let result = backend
            .find_path(&asset, &NavPathQuery::new([0.0, 0.0, 0.0], [8.0, 0.0, 0.0]))
            .unwrap();

        assert_eq!(result.status, NavPathStatus::NoPath);
    }

    #[test]
    fn off_mesh_link_bridges_disconnected_polygons() {
        let backend = RecastBackend;
        let asset = two_island_asset(true);

        let result = backend
            .find_path(&asset, &NavPathQuery::new([0.0, 0.0, 0.0], [8.0, 0.0, 0.0]))
            .unwrap();

        assert_eq!(result.status, NavPathStatus::Complete);
        assert!(result
            .points
            .iter()
            .any(|point| point.flags.iter().any(|flag| flag == "off_mesh_link")));
    }

    #[test]
    fn sample_clamps_to_asset_bounds() {
        let backend = RecastBackend;
        let asset = NavMeshAsset::simple_quad("humanoid", 5.0);

        let hit = backend
            .sample_position(
                &asset,
                &NavSampleQuery {
                    nav_mesh: None,
                    position: [10.0, 0.0, 0.0],
                    extents: [6.0, 1.0, 6.0],
                    agent_type: "humanoid".to_string(),
                    area_mask: DEFAULT_AREA_MASK,
                },
            )
            .unwrap()
            .unwrap();

        assert_eq!(hit.position, [5.0, 0.0, 0.0]);
    }

    fn two_island_asset(with_link: bool) -> NavMeshAsset {
        let mut asset = NavMeshAsset {
            version: NavMeshAsset::VERSION,
            agent_type: "humanoid".to_string(),
            settings_hash: 0,
            vertices: vec![
                [-1.0, 0.0, -1.0],
                [1.0, 0.0, -1.0],
                [1.0, 0.0, 1.0],
                [-1.0, 0.0, 1.0],
                [7.0, 0.0, -1.0],
                [9.0, 0.0, -1.0],
                [9.0, 0.0, 1.0],
                [7.0, 0.0, 1.0],
            ],
            indices: vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7],
            polygons: vec![
                NavMeshPolygonAsset {
                    first_index: 0,
                    index_count: 6,
                    area: AREA_WALKABLE,
                    tile: 0,
                },
                NavMeshPolygonAsset {
                    first_index: 6,
                    index_count: 6,
                    area: AREA_WALKABLE,
                    tile: 1,
                },
            ],
            tiles: Vec::new(),
            off_mesh_links: Vec::new(),
        };
        if with_link {
            asset.off_mesh_links.push(NavMeshLinkAsset {
                start: [1.0, 0.0, 0.0],
                end: [7.0, 0.0, 0.0],
                width: 0.5,
                bidirectional: true,
                area: AREA_JUMP,
                cost_override: None,
                traversal_mode: Default::default(),
            });
        }
        asset
    }
}
