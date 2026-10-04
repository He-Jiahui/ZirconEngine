use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::core::framework::navigation::{
    NavMeshAsset, NavMeshPolygonAsset, NavPathPoint, NavPathQuery, NavPathResult, NavPathStatus,
    NavRaycastQuery, NavRaycastResult, NavSampleHit, NavSampleQuery, AREA_WALKABLE, MAX_NAV_AREAS,
};
use crate::core::math::{Real, Vec3};

use super::math::distance_xz;

mod query_scratch;
mod spatial_index;

use query_scratch::{FallbackQueryScratch, PathOpenNode};
use spatial_index::PolygonSpatialIndex;

#[derive(Clone, Debug)]
pub(super) struct BakedNavMesh {
    polygons: Vec<BakedPolygon>,
    adjacency: Vec<Vec<usize>>,
    area_costs: [Real; MAX_NAV_AREAS],
    spatial_index: PolygonSpatialIndex,
    query_scratch: Arc<Mutex<FallbackQueryScratch>>,
}

impl BakedNavMesh {
    pub(super) fn new(asset: NavMeshAsset) -> Self {
        let polygons = asset
            .polygons
            .iter()
            .map(|polygon| BakedPolygon::from_asset(&asset, polygon))
            .collect::<Vec<_>>();
        let adjacency = build_adjacency(&polygons);
        let area_costs = build_area_costs(&asset);
        let spatial_index = PolygonSpatialIndex::new(&polygons);
        Self {
            polygons,
            adjacency,
            area_costs,
            spatial_index,
            query_scratch: Arc::new(Mutex::new(FallbackQueryScratch::default())),
        }
    }

    pub(super) fn find_path(&self, query: NavPathQuery) -> NavPathResult {
        if self.polygons.is_empty() {
            return NavPathResult::no_path();
        }
        let mut scratch = self
            .query_scratch
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        scratch.prepare(self.polygons.len());
        self.find_path_with_scratch(query, &mut scratch)
    }

    fn find_path_with_scratch(
        &self,
        query: NavPathQuery,
        scratch: &mut FallbackQueryScratch,
    ) -> NavPathResult {
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
        if !self.graph_path(start_polygon, end_polygon, end, scratch) {
            let nearest =
                self.nearest_reachable_polygon(start_polygon, end, query.area_mask, scratch);
            let points = vec![
                self.path_point(start, start_polygon),
                self.path_point(self.polygons[nearest].center, nearest),
            ];
            return path_result(NavPathStatus::Partial, points, self.polygons.len());
        }
        let mut points = Vec::with_capacity(scratch.path.len() + 1);
        points.push(self.path_point(start, start_polygon));
        for polygon_index in scratch
            .path
            .iter()
            .skip(1)
            .take(scratch.path.len().saturating_sub(2))
        {
            points.push(self.path_point(self.polygons[*polygon_index].center, *polygon_index));
        }
        points.push(self.path_point(self.polygons[end_polygon].project_point(end), end_polygon));
        path_result(
            NavPathStatus::Complete,
            deduplicate_path_points(points),
            scratch.path.len(),
        )
    }

    pub(super) fn sample_position(&self, query: NavSampleQuery) -> Option<NavSampleHit> {
        let position = Vec3::from_array(query.position);
        let max_distance = Vec3::from_array(query.extents).abs().length().max(0.05);
        let mut work = MeshQueryWork::default();
        let result =
            self.sample_position_with_work(position, max_distance, query.area_mask, &mut work);
        debug_assert!(work.bvh_nodes <= self.spatial_index.node_count());
        debug_assert!(work.polygon_candidates <= self.polygons.len());
        result
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

    fn graph_path(
        &self,
        start: usize,
        end: usize,
        end_position: Vec3,
        scratch: &mut FallbackQueryScratch,
    ) -> bool {
        scratch.set_best_cost(start, 0.0);
        scratch.open.push(PathOpenNode {
            polygon: start,
            estimated_total: distance_xz(self.polygons[start].center, end_position),
        });
        while let Some(node) = scratch.open.pop() {
            if node.polygon == end {
                break;
            }
            let current_cost = scratch.best_cost(node.polygon);
            for next in &self.adjacency[node.polygon] {
                let step_cost = distance_xz(
                    self.polygons[node.polygon].center,
                    self.polygons[*next].center,
                ) * self.area_cost(self.polygons[*next].area);
                let tentative = current_cost + step_cost;
                if tentative >= scratch.best_cost(*next) {
                    continue;
                }
                scratch.set_best_cost(*next, tentative);
                scratch.set_previous(*next, node.polygon);
                scratch.open.push(PathOpenNode {
                    polygon: *next,
                    estimated_total: tentative
                        + distance_xz(self.polygons[*next].center, end_position),
                });
            }
        }
        if !scratch.best_cost(end).is_finite() {
            return false;
        }
        scratch.path.push(end);
        let mut cursor = end;
        while cursor != start {
            let Some(previous) = scratch.previous(cursor) else {
                scratch.path.clear();
                return false;
            };
            cursor = previous;
            scratch.path.push(cursor);
        }
        scratch.path.reverse();
        true
    }

    fn nearest_reachable_polygon(
        &self,
        start: usize,
        end: Vec3,
        area_mask: u64,
        scratch: &mut FallbackQueryScratch,
    ) -> usize {
        scratch.reset_traversal();
        scratch.traversal.push(start);
        let mut best = start;
        let mut best_distance = distance_xz(self.polygons[start].center, end);
        while let Some(current) = scratch.traversal.pop() {
            if !scratch.mark_visited(current) {
                continue;
            }
            let distance = distance_xz(self.polygons[current].center, end);
            if distance < best_distance {
                best = current;
                best_distance = distance;
            }
            for next in &self.adjacency[current] {
                if !scratch.is_visited(*next) && area_allowed(self.polygons[*next].area, area_mask)
                {
                    scratch.traversal.push(*next);
                }
            }
        }
        best
    }

    fn best_polygon(&self, position: Vec3, area_mask: u64) -> Option<usize> {
        let mut work = MeshQueryWork::default();
        let result = self.best_polygon_with_work(position, area_mask, &mut work);
        debug_assert!(work.bvh_nodes <= self.spatial_index.node_count());
        debug_assert!(work.polygon_candidates <= self.polygons.len());
        result
    }

    fn best_polygon_with_work(
        &self,
        position: Vec3,
        area_mask: u64,
        work: &mut MeshQueryWork,
    ) -> Option<usize> {
        let mut best = None;
        self.spatial_index
            .visit_nearest(&self.polygons, position, area_mask, &mut best, work);
        best.map(|(polygon, _)| polygon)
    }

    fn sample_position_with_work(
        &self,
        position: Vec3,
        max_distance: Real,
        area_mask: u64,
        work: &mut MeshQueryWork,
    ) -> Option<NavSampleHit> {
        let mut best = None;
        self.spatial_index.visit_sample(
            &self.polygons,
            position,
            max_distance,
            area_mask,
            &mut best,
            work,
        );
        best.map(|(polygon, projected, distance)| NavSampleHit {
            position: projected.to_array(),
            distance,
            area: self.polygons[polygon].area,
        })
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
        self.area_costs[usize::from(area.min((MAX_NAV_AREAS - 1) as u8))]
    }
}

#[derive(Debug, Default)]
pub(super) struct MeshQueryWork {
    pub(super) bvh_nodes: usize,
    pub(super) polygon_candidates: usize,
}

fn build_area_costs(asset: &NavMeshAsset) -> [Real; MAX_NAV_AREAS] {
    let mut area_costs = [1.0; MAX_NAV_AREAS];
    let mut configured = [false; MAX_NAV_AREAS];
    for cost in &asset.area_costs {
        let index = usize::from(cost.area.min((MAX_NAV_AREAS - 1) as u8));
        if configured[index] {
            continue;
        }
        area_costs[index] = cost.cost.max(0.01);
        configured[index] = true;
    }
    area_costs
}

#[derive(Clone, Debug)]
pub(super) struct BakedPolygon {
    pub(super) area: u8,
    pub(super) center: Vec3,
    pub(super) min: Vec3,
    pub(super) max: Vec3,
    edge_keys: Vec<PolygonEdgeKey>,
}

type PolygonEdgeKey = (u32, u32);

impl BakedPolygon {
    fn from_asset(asset: &NavMeshAsset, polygon: &NavMeshPolygonAsset) -> Self {
        let start = polygon.first_index as usize;
        let end = start
            .saturating_add(polygon.index_count as usize)
            .min(asset.indices.len());
        let index_set = &asset.indices[start.min(asset.indices.len())..end];
        let edge_keys = polygon_edge_keys(index_set);
        let mut vertices = polygon_vertices(asset, index_set);
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
            edge_keys,
        }
    }

    pub(super) fn contains_xz(&self, point: Vec3) -> bool {
        point.x >= self.min.x - 0.001
            && point.x <= self.max.x + 0.001
            && point.z >= self.min.z - 0.001
            && point.z <= self.max.z + 0.001
    }

    pub(super) fn project_point(&self, point: Vec3) -> Vec3 {
        let x = point.x.clamp(self.min.x, self.max.x);
        let z = point.z.clamp(self.min.z, self.max.z);
        let y = self.center.y;
        Vec3::new(x, y, z)
    }
}

fn polygon_vertices(asset: &NavMeshAsset, index_set: &[u32]) -> Vec<Vec3> {
    let mut vertices = Vec::with_capacity(index_set.len());
    vertices.extend(
        index_set
            .iter()
            .filter_map(|index| asset.vertices.get(*index as usize).copied())
            .map(Vec3::from_array),
    );
    vertices
}

fn build_adjacency(polygons: &[BakedPolygon]) -> Vec<Vec<usize>> {
    // Topology is established from canonical triangle edges, never from spatial overlap.
    let mut edge_to_polygons = BTreeMap::<PolygonEdgeKey, Vec<usize>>::new();
    for (polygon_index, polygon) in polygons.iter().enumerate() {
        for edge_key in &polygon.edge_keys {
            edge_to_polygons
                .entry(*edge_key)
                .or_default()
                .push(polygon_index);
        }
    }

    let mut adjacency = vec![Vec::new(); polygons.len()];
    for polygon_indices in edge_to_polygons.values_mut() {
        polygon_indices.sort_unstable();
        polygon_indices.dedup();
        for left_offset in 0..polygon_indices.len() {
            for right_offset in (left_offset + 1)..polygon_indices.len() {
                let left = polygon_indices[left_offset];
                let right = polygon_indices[right_offset];
                adjacency[left].push(right);
                adjacency[right].push(left);
            }
        }
    }

    for neighbors in &mut adjacency {
        neighbors.sort_unstable();
        neighbors.dedup();
    }
    adjacency
}

fn polygon_edge_keys(indices: &[u32]) -> Vec<PolygonEdgeKey> {
    let mut edge_keys = Vec::with_capacity(indices.len());
    for triangle in indices.chunks_exact(3) {
        edge_keys.push(polygon_edge_key(triangle[0], triangle[1]));
        edge_keys.push(polygon_edge_key(triangle[1], triangle[2]));
        edge_keys.push(polygon_edge_key(triangle[2], triangle[0]));
    }
    edge_keys.sort_unstable();
    edge_keys.dedup();
    edge_keys
}

fn polygon_edge_key(left: u32, right: u32) -> PolygonEdgeKey {
    (left.min(right), left.max(right))
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

fn deduplicate_path_points(mut points: Vec<NavPathPoint>) -> Vec<NavPathPoint> {
    points.dedup_by(|current, previous| {
        distance_xz(
            Vec3::from_array(previous.position),
            Vec3::from_array(current.position),
        ) < 0.05
    });
    points
}

#[cfg(test)]
#[path = "tests/baked_mesh_performance_contract_tests.rs"]
mod performance_contract_tests;
