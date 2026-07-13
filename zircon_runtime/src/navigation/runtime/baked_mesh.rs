use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::core::framework::navigation::{
    NavMeshAsset, NavMeshPolygonAsset, NavPathPoint, NavPathQuery, NavPathResult, NavPathStatus,
    NavRaycastQuery, NavRaycastResult, NavSampleHit, NavSampleQuery, AREA_WALKABLE,
};
use crate::core::math::{Real, Vec3};

use super::math::distance_xz;

#[derive(Clone, Debug)]
pub(super) struct BakedNavMesh {
    asset: NavMeshAsset,
    polygons: Vec<BakedPolygon>,
    adjacency: Vec<Vec<usize>>,
}

impl BakedNavMesh {
    pub(super) fn new(asset: NavMeshAsset) -> Self {
        let polygons = asset
            .polygons
            .iter()
            .map(|polygon| BakedPolygon::from_asset(&asset, polygon))
            .collect::<Vec<_>>();
        let adjacency = build_adjacency(&polygons);
        Self {
            asset,
            polygons,
            adjacency,
        }
    }

    pub(super) fn find_path(&self, query: NavPathQuery) -> NavPathResult {
        if self.polygons.is_empty() {
            return NavPathResult::no_path();
        }
        let start = Vec3::from_array(query.start);
        let end = Vec3::from_array(query.end);
        let Some(start_polygon) = self.best_polygon(start, query.area_mask) else {
            return NavPathResult::no_path();
        };
        let Some(end_polygon) = self.best_polygon(end, query.area_mask) else {
            return NavPathResult::no_path();
        };
        if start_polygon == end_polygon {
            let points = vec![
                self.path_point(start, start_polygon),
                self.path_point(self.polygons[start_polygon].project_point(end), end_polygon),
            ];
            return path_result(NavPathStatus::Complete, points, 1);
        }
        let Some(poly_path) = self.graph_path(start_polygon, end_polygon, end) else {
            let nearest = self.nearest_reachable_polygon(start_polygon, end, query.area_mask);
            let points = vec![
                self.path_point(start, start_polygon),
                self.path_point(self.polygons[nearest].center, nearest),
            ];
            return path_result(NavPathStatus::Partial, points, self.polygons.len());
        };
        let mut points = Vec::with_capacity(poly_path.len() + 1);
        points.push(self.path_point(start, start_polygon));
        for polygon_index in poly_path
            .iter()
            .skip(1)
            .take(poly_path.len().saturating_sub(2))
        {
            points.push(self.path_point(self.polygons[*polygon_index].center, *polygon_index));
        }
        points.push(self.path_point(self.polygons[end_polygon].project_point(end), end_polygon));
        path_result(
            NavPathStatus::Complete,
            deduplicate_path_points(points),
            poly_path.len(),
        )
    }

    pub(super) fn sample_position(&self, query: NavSampleQuery) -> Option<NavSampleHit> {
        let position = Vec3::from_array(query.position);
        let max_distance = Vec3::from_array(query.extents).abs().length().max(0.05);
        self.polygons
            .iter()
            .filter(|polygon| area_allowed(polygon.area, query.area_mask))
            .map(|polygon| {
                let projected = polygon.project_point(position);
                let distance = distance_xz(position, projected);
                (polygon, projected, distance)
            })
            .filter(|(_, _, distance)| *distance <= max_distance)
            .min_by(|left, right| left.2.total_cmp(&right.2))
            .map(|(polygon, projected, distance)| NavSampleHit {
                position: projected.to_array(),
                distance,
                area: polygon.area,
            })
    }

    pub(super) fn raycast(&self, query: NavRaycastQuery) -> NavRaycastResult {
        let end = Vec3::from_array(query.end);
        let hit = self
            .best_polygon(end, query.area_mask)
            .map(|polygon| self.polygons[polygon].contains_xz(end))
            .unwrap_or(false);
        NavRaycastResult {
            hit: !hit,
            position: query.end,
            normal: [0.0, 1.0, 0.0],
            distance: distance_xz(Vec3::from_array(query.start), end),
        }
    }

    fn graph_path(&self, start: usize, end: usize, end_position: Vec3) -> Option<Vec<usize>> {
        let mut open = BinaryHeap::new();
        let mut best_cost = vec![Real::INFINITY; self.polygons.len()];
        let mut previous = vec![None; self.polygons.len()];
        best_cost[start] = 0.0;
        open.push(PathOpenNode {
            polygon: start,
            estimated_total: distance_xz(self.polygons[start].center, end_position),
        });
        while let Some(node) = open.pop() {
            if node.polygon == end {
                break;
            }
            let current_cost = best_cost[node.polygon];
            for next in &self.adjacency[node.polygon] {
                let step_cost = distance_xz(
                    self.polygons[node.polygon].center,
                    self.polygons[*next].center,
                ) * self.area_cost(self.polygons[*next].area);
                let tentative = current_cost + step_cost;
                if tentative >= best_cost[*next] {
                    continue;
                }
                best_cost[*next] = tentative;
                previous[*next] = Some(node.polygon);
                open.push(PathOpenNode {
                    polygon: *next,
                    estimated_total: tentative
                        + distance_xz(self.polygons[*next].center, end_position),
                });
            }
        }
        if !best_cost[end].is_finite() {
            return None;
        }
        let mut path = vec![end];
        let mut cursor = end;
        while cursor != start {
            cursor = previous[cursor]?;
            path.push(cursor);
        }
        path.reverse();
        Some(path)
    }

    fn nearest_reachable_polygon(&self, start: usize, end: Vec3, area_mask: u64) -> usize {
        let mut stack = vec![start];
        let mut visited = vec![false; self.polygons.len()];
        let mut best = start;
        let mut best_distance = distance_xz(self.polygons[start].center, end);
        while let Some(current) = stack.pop() {
            if visited[current] {
                continue;
            }
            visited[current] = true;
            let distance = distance_xz(self.polygons[current].center, end);
            if distance < best_distance {
                best = current;
                best_distance = distance;
            }
            for next in &self.adjacency[current] {
                if !visited[*next] && area_allowed(self.polygons[*next].area, area_mask) {
                    stack.push(*next);
                }
            }
        }
        best
    }

    fn best_polygon(&self, position: Vec3, area_mask: u64) -> Option<usize> {
        self.polygons
            .iter()
            .enumerate()
            .filter(|(_, polygon)| area_allowed(polygon.area, area_mask))
            .map(|(index, polygon)| {
                let projected = polygon.project_point(position);
                let penalty = if polygon.contains_xz(position) {
                    0.0
                } else {
                    10_000.0
                };
                (index, distance_xz(position, projected) + penalty)
            })
            .min_by(|left, right| left.1.total_cmp(&right.1))
            .map(|(index, _)| index)
    }

    fn path_point(&self, position: Vec3, polygon: usize) -> NavPathPoint {
        NavPathPoint {
            position: position.to_array(),
            area: self.polygons[polygon].area,
            off_mesh_link_id: None,
            flags: Vec::new(),
        }
    }

    fn area_cost(&self, area: u8) -> Real {
        self.asset
            .area_costs
            .iter()
            .find(|cost| cost.area == area)
            .map(|cost| cost.cost.max(0.01))
            .unwrap_or(1.0)
    }
}

#[derive(Clone, Debug)]
struct BakedPolygon {
    area: u8,
    center: Vec3,
    min: Vec3,
    max: Vec3,
    index_set: Vec<u32>,
}

impl BakedPolygon {
    fn from_asset(asset: &NavMeshAsset, polygon: &NavMeshPolygonAsset) -> Self {
        let start = polygon.first_index as usize;
        let end = start
            .saturating_add(polygon.index_count as usize)
            .min(asset.indices.len());
        let index_set = asset.indices[start.min(asset.indices.len())..end].to_vec();
        let mut vertices = index_set
            .iter()
            .filter_map(|index| asset.vertices.get(*index as usize).copied())
            .map(Vec3::from_array)
            .collect::<Vec<_>>();
        vertices.sort_by(|left, right| {
            left.x
                .total_cmp(&right.x)
                .then(left.z.total_cmp(&right.z))
                .then(left.y.total_cmp(&right.y))
        });
        vertices.dedup_by(|left, right| {
            (left.x - right.x).abs() < 0.001 && (left.z - right.z).abs() < 0.001
        });
        let (min, max) = bounds_for_vertices(&vertices);
        let center = if vertices.is_empty() {
            Vec3::ZERO
        } else {
            vertices
                .iter()
                .copied()
                .fold(Vec3::ZERO, |sum, vertex| sum + vertex)
                / vertices.len() as Real
        };
        Self {
            area: polygon.area,
            center,
            min,
            max,
            index_set,
        }
    }

    fn contains_xz(&self, point: Vec3) -> bool {
        point.x >= self.min.x - 0.001
            && point.x <= self.max.x + 0.001
            && point.z >= self.min.z - 0.001
            && point.z <= self.max.z + 0.001
    }

    fn project_point(&self, point: Vec3) -> Vec3 {
        let x = point.x.clamp(self.min.x, self.max.x);
        let z = point.z.clamp(self.min.z, self.max.z);
        let y = self.center.y;
        Vec3::new(x, y, z)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PathOpenNode {
    polygon: usize,
    estimated_total: Real,
}

impl Eq for PathOpenNode {}

impl Ord for PathOpenNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .estimated_total
            .total_cmp(&self.estimated_total)
            .then_with(|| other.polygon.cmp(&self.polygon))
    }
}

impl PartialOrd for PathOpenNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn build_adjacency(polygons: &[BakedPolygon]) -> Vec<Vec<usize>> {
    let mut adjacency = vec![Vec::new(); polygons.len()];
    for left in 0..polygons.len() {
        for right in (left + 1)..polygons.len() {
            if polygons_share_edge(&polygons[left], &polygons[right]) {
                adjacency[left].push(right);
                adjacency[right].push(left);
            }
        }
    }
    adjacency
}

fn polygons_share_edge(left: &BakedPolygon, right: &BakedPolygon) -> bool {
    let shared_indices = left
        .index_set
        .iter()
        .filter(|index| right.index_set.contains(index))
        .count();
    if shared_indices >= 2 {
        return true;
    }
    rectangles_touch_or_overlap(left, right)
}

fn rectangles_touch_or_overlap(left: &BakedPolygon, right: &BakedPolygon) -> bool {
    let x_overlap = left.max.x >= right.min.x - 0.001 && right.max.x >= left.min.x - 0.001;
    let z_overlap = left.max.z >= right.min.z - 0.001 && right.max.z >= left.min.z - 0.001;
    x_overlap && z_overlap
}

fn bounds_for_vertices(vertices: &[Vec3]) -> (Vec3, Vec3) {
    let Some(first) = vertices.first().copied() else {
        return (Vec3::ZERO, Vec3::ZERO);
    };
    let mut min = first;
    let mut max = first;
    for vertex in vertices.iter().copied().skip(1) {
        min = min.min(vertex);
        max = max.max(vertex);
    }
    (min, max)
}

fn area_allowed(area: u8, mask: u64) -> bool {
    area == AREA_WALKABLE || (mask & (1_u64 << area.min(63))) != 0
}

fn path_result(
    status: NavPathStatus,
    points: Vec<NavPathPoint>,
    visited_nodes: usize,
) -> NavPathResult {
    let length = points
        .windows(2)
        .map(|window| {
            distance_xz(
                Vec3::from_array(window[0].position),
                Vec3::from_array(window[1].position),
            )
        })
        .sum();
    NavPathResult {
        status,
        points,
        length,
        visited_nodes,
    }
}

fn deduplicate_path_points(points: Vec<NavPathPoint>) -> Vec<NavPathPoint> {
    let mut result = Vec::new();
    for point in points {
        let duplicate = result.last().is_some_and(|previous: &NavPathPoint| {
            distance_xz(
                Vec3::from_array(previous.position),
                Vec3::from_array(point.position),
            ) < 0.05
        });
        if !duplicate {
            result.push(point);
        }
    }
    result
}
