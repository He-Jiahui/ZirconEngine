use crate::core::math::{Real, Vec3};

use super::super::math::distance_xz;
use super::{area_allowed, BakedPolygon, MeshQueryWork};

const POLYGON_SPATIAL_INDEX_LEAF_SIZE: usize = 8;

#[derive(Clone, Copy, Debug)]
struct PolygonSpatialIndexNode {
    min: Vec3,
    max: Vec3,
    left: Option<usize>,
    right: Option<usize>,
    start: usize,
    end: usize,
}

impl PolygonSpatialIndexNode {
    fn leaf(min: Vec3, max: Vec3, start: usize, end: usize) -> Self {
        Self {
            min,
            max,
            left: None,
            right: None,
            start,
            end,
        }
    }

    fn branch(min: Vec3, max: Vec3, left: usize, right: usize) -> Self {
        Self {
            min,
            max,
            left: Some(left),
            right: Some(right),
            start: 0,
            end: 0,
        }
    }

    fn contains_xz(&self, position: Vec3) -> bool {
        position.x >= self.min.x - 0.001
            && position.x <= self.max.x + 0.001
            && position.z >= self.min.z - 0.001
            && position.z <= self.max.z + 0.001
    }

    fn distance_xz(&self, position: Vec3) -> Real {
        let x = if position.x < self.min.x {
            self.min.x - position.x
        } else if position.x > self.max.x {
            position.x - self.max.x
        } else {
            0.0
        };
        let z = if position.z < self.min.z {
            self.min.z - position.z
        } else if position.z > self.max.z {
            position.z - self.max.z
        } else {
            0.0
        };
        (x * x + z * z).sqrt()
    }

    fn lower_bound_score(&self, position: Vec3) -> Real {
        if self.contains_xz(position) {
            0.0
        } else {
            10_000.0 + self.distance_xz(position)
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct PolygonSpatialIndex {
    nodes: Vec<PolygonSpatialIndexNode>,
    polygon_order: Vec<usize>,
}

impl PolygonSpatialIndex {
    pub(super) fn new(polygons: &[BakedPolygon]) -> Self {
        let mut index = Self::default();
        if polygons.is_empty() {
            return index;
        }
        let mut polygon_indices = (0..polygons.len()).collect::<Vec<_>>();
        index.build_node(&mut polygon_indices, polygons);
        index
    }

    fn build_node(&mut self, polygon_indices: &mut [usize], polygons: &[BakedPolygon]) -> usize {
        let (min, max) = polygon_bounds(polygon_indices, polygons);
        let node_index = self.nodes.len();
        self.nodes
            .push(PolygonSpatialIndexNode::leaf(min, max, 0, 0));
        if polygon_indices.len() <= POLYGON_SPATIAL_INDEX_LEAF_SIZE {
            let start = self.polygon_order.len();
            self.polygon_order.extend_from_slice(polygon_indices);
            let end = self.polygon_order.len();
            self.nodes[node_index] = PolygonSpatialIndexNode::leaf(min, max, start, end);
            return node_index;
        }

        let extent = max - min;
        if extent.x >= extent.z {
            polygon_indices.sort_unstable_by(|left, right| {
                polygons[*left]
                    .center
                    .x
                    .total_cmp(&polygons[*right].center.x)
                    .then(left.cmp(right))
            });
        } else {
            polygon_indices.sort_unstable_by(|left, right| {
                polygons[*left]
                    .center
                    .z
                    .total_cmp(&polygons[*right].center.z)
                    .then(left.cmp(right))
            });
        }
        let middle = polygon_indices.len() / 2;
        let (left_indices, right_indices) = polygon_indices.split_at_mut(middle);
        let left = self.build_node(left_indices, polygons);
        let right = self.build_node(right_indices, polygons);
        self.nodes[node_index] = PolygonSpatialIndexNode::branch(min, max, left, right);
        node_index
    }

    pub(super) fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub(super) fn visit_nearest(
        &self,
        polygons: &[BakedPolygon],
        position: Vec3,
        area_mask: u64,
        best: &mut Option<(usize, Real)>,
        work: &mut MeshQueryWork,
    ) {
        if self.nodes.is_empty() {
            return;
        }
        self.visit_nearest_node(0, polygons, position, area_mask, best, work);
    }

    fn visit_nearest_node(
        &self,
        node_index: usize,
        polygons: &[BakedPolygon],
        position: Vec3,
        area_mask: u64,
        best: &mut Option<(usize, Real)>,
        work: &mut MeshQueryWork,
    ) {
        let node = self.nodes[node_index];
        work.bvh_nodes = work.bvh_nodes.saturating_add(1);
        if best
            .as_ref()
            .is_some_and(|(_, score)| node.lower_bound_score(position) > *score)
        {
            return;
        }
        match (node.left, node.right) {
            (Some(left), Some(right)) => {
                let left_score = self.nodes[left].lower_bound_score(position);
                let right_score = self.nodes[right].lower_bound_score(position);
                if left_score <= right_score {
                    self.visit_nearest_node(left, polygons, position, area_mask, best, work);
                    self.visit_nearest_node(right, polygons, position, area_mask, best, work);
                } else {
                    self.visit_nearest_node(right, polygons, position, area_mask, best, work);
                    self.visit_nearest_node(left, polygons, position, area_mask, best, work);
                }
            }
            (None, None) => {
                for polygon_index in &self.polygon_order[node.start..node.end] {
                    let polygon = &polygons[*polygon_index];
                    if !area_allowed(polygon.area, area_mask) {
                        continue;
                    }
                    work.polygon_candidates = work.polygon_candidates.saturating_add(1);
                    let score = polygon_score(polygon, position);
                    let replace = best.as_ref().is_none_or(|(current, current_score)| {
                        score < *current_score
                            || (score == *current_score && *polygon_index < *current)
                    });
                    if replace {
                        *best = Some((*polygon_index, score));
                    }
                }
            }
            _ => unreachable!("spatial index nodes are either branches or leaves"),
        }
    }

    pub(super) fn visit_sample(
        &self,
        polygons: &[BakedPolygon],
        position: Vec3,
        max_distance: Real,
        area_mask: u64,
        best: &mut Option<(usize, Vec3, Real)>,
        work: &mut MeshQueryWork,
    ) {
        if self.nodes.is_empty() {
            return;
        }
        self.visit_sample_node(0, polygons, position, max_distance, area_mask, best, work);
    }

    fn visit_sample_node(
        &self,
        node_index: usize,
        polygons: &[BakedPolygon],
        position: Vec3,
        max_distance: Real,
        area_mask: u64,
        best: &mut Option<(usize, Vec3, Real)>,
        work: &mut MeshQueryWork,
    ) {
        let node = self.nodes[node_index];
        let node_distance = node.distance_xz(position);
        work.bvh_nodes = work.bvh_nodes.saturating_add(1);
        if node_distance > max_distance
            || best
                .as_ref()
                .is_some_and(|(_, _, distance)| node_distance > *distance)
        {
            return;
        }
        match (node.left, node.right) {
            (Some(left), Some(right)) => {
                let left_distance = self.nodes[left].distance_xz(position);
                let right_distance = self.nodes[right].distance_xz(position);
                if left_distance <= right_distance {
                    self.visit_sample_node(
                        left,
                        polygons,
                        position,
                        max_distance,
                        area_mask,
                        best,
                        work,
                    );
                    self.visit_sample_node(
                        right,
                        polygons,
                        position,
                        max_distance,
                        area_mask,
                        best,
                        work,
                    );
                } else {
                    self.visit_sample_node(
                        right,
                        polygons,
                        position,
                        max_distance,
                        area_mask,
                        best,
                        work,
                    );
                    self.visit_sample_node(
                        left,
                        polygons,
                        position,
                        max_distance,
                        area_mask,
                        best,
                        work,
                    );
                }
            }
            (None, None) => {
                for polygon_index in &self.polygon_order[node.start..node.end] {
                    let polygon = &polygons[*polygon_index];
                    if !area_allowed(polygon.area, area_mask) {
                        continue;
                    }
                    work.polygon_candidates = work.polygon_candidates.saturating_add(1);
                    let projected = polygon.project_point(position);
                    let distance = distance_xz(position, projected);
                    if distance > max_distance {
                        continue;
                    }
                    let replace = best.as_ref().is_none_or(|(current, _, current_distance)| {
                        distance < *current_distance
                            || (distance == *current_distance && *polygon_index < *current)
                    });
                    if replace {
                        *best = Some((*polygon_index, projected, distance));
                    }
                }
            }
            _ => unreachable!("spatial index nodes are either branches or leaves"),
        }
    }
}

fn polygon_bounds(indices: &[usize], polygons: &[BakedPolygon]) -> (Vec3, Vec3) {
    let first = &polygons[indices[0]];
    indices
        .iter()
        .copied()
        .skip(1)
        .fold((first.min, first.max), |(min, max), index| {
            (min.min(polygons[index].min), max.max(polygons[index].max))
        })
}

fn polygon_score(polygon: &BakedPolygon, position: Vec3) -> Real {
    let penalty = if polygon.contains_xz(position) {
        0.0
    } else {
        10_000.0
    };
    penalty + distance_xz(position, polygon.project_point(position))
}
