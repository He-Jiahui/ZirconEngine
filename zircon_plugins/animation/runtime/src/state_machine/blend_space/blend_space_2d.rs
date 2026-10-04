use std::collections::BTreeMap;

use spade::{DelaunayTriangulation, HasPosition, Point2, Triangulation};
use zircon_runtime::core::framework::animation::compiler::state_machine::AnimationCompiledBlendSpace2DSample;
use zircon_runtime::core::math::{Real, Vec2};

use super::geometry::{barycentric, inside, project_to_segment};
use super::{BlendSpaceCompileError, BlendSpaceWeights3};

#[derive(Clone, Copy, Debug, PartialEq)]
struct PreparedPoint2D {
    position: Vec2,
    sample: u32,
}

#[derive(Clone, Copy, Debug)]
struct TopologyVertex {
    position: Point2<f64>,
    point: usize,
}

enum TriangleWalk {
    Inside { triangle: usize, weights: [Real; 3] },
    OutsideHull { triangle: usize },
    Failed,
}

impl HasPosition for TopologyVertex {
    type Scalar = f64;

    fn position(&self) -> Point2<Self::Scalar> {
        self.position
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlendSpace2D {
    points: Box<[PreparedPoint2D]>,
    triangles: Box<[[usize; 3]]>,
    neighbors: Box<[[Option<usize>; 3]]>,
    hull_edges: Box<[[usize; 2]]>,
}

impl BlendSpace2D {
    pub(in crate::state_machine) fn from_compiled(
        samples: &[AnimationCompiledBlendSpace2DSample],
    ) -> Result<Self, BlendSpaceCompileError> {
        let points = samples
            .iter()
            .enumerate()
            .map(|(sample, source)| {
                Ok(PreparedPoint2D {
                    position: source.position,
                    sample: u32::try_from(sample)
                        .map_err(|_| BlendSpaceCompileError::CapacityExceeded)?,
                })
            })
            .collect::<Result<Vec<_>, BlendSpaceCompileError>>()?;
        let (triangles, neighbors, hull_edges) = compile_topology(&points)?;
        Ok(Self {
            points: points.into_boxed_slice(),
            triangles: triangles.into_boxed_slice(),
            neighbors: neighbors.into_boxed_slice(),
            hull_edges: hull_edges.into_boxed_slice(),
        })
    }

    #[cfg(test)]
    fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    pub fn sample(&self, point: Vec2) -> Option<BlendSpaceWeights3> {
        self.sample_with_hint(point, None)
            .map(|(weights, _)| weights)
    }

    /// Walks from retained location and reserves the triangle scan for abnormal topology failure.
    pub(crate) fn sample_with_hint(
        &self,
        point: Vec2,
        hint: Option<usize>,
    ) -> Option<(BlendSpaceWeights3, Option<usize>)> {
        if !point.is_finite() {
            return None;
        }
        match self.walk_from_hint(point, hint) {
            TriangleWalk::Inside { triangle, weights } => Some((
                self.weights(self.triangles[triangle], weights),
                Some(triangle),
            )),
            TriangleWalk::OutsideHull { triangle } => self
                .sample_hull(point)
                .map(|weights| (weights, Some(triangle))),
            TriangleWalk::Failed => self.sample_after_failed_walk(point),
        }
    }

    fn sample_after_failed_walk(&self, point: Vec2) -> Option<(BlendSpaceWeights3, Option<usize>)> {
        for (index, triangle) in self.triangles.iter().copied().enumerate() {
            let positions = triangle.map(|point_index| self.points[point_index].position);
            let Some(weights) = barycentric(point, positions[0], positions[1], positions[2]) else {
                continue;
            };
            if inside(weights) {
                return Some((self.weights(triangle, weights), Some(index)));
            }
        }
        let weights = self.sample_hull(point)?;
        Some((weights, None))
    }

    fn walk_from_hint(&self, point: Vec2, hint: Option<usize>) -> TriangleWalk {
        let Some(mut current) = hint
            .filter(|index| *index < self.triangles.len())
            .or_else(|| (!self.triangles.is_empty()).then_some(self.triangles.len() / 2))
        else {
            return TriangleWalk::Failed;
        };
        let mut previous = None;
        for _ in 0..self.triangles.len() {
            let triangle = self.triangles[current];
            let positions = triangle.map(|index| self.points[index].position);
            let Some(weights) = barycentric(point, positions[0], positions[1], positions[2]) else {
                return TriangleWalk::Failed;
            };
            if inside(weights) {
                return TriangleWalk::Inside {
                    triangle: current,
                    weights,
                };
            }
            let Some(outside_edge) = weights
                .iter()
                .enumerate()
                .min_by(|left, right| left.1.total_cmp(right.1))
                .map(|(index, _)| index)
            else {
                return TriangleWalk::Failed;
            };
            let Some(next) = self.neighbors[current][outside_edge] else {
                return TriangleWalk::OutsideHull { triangle: current };
            };
            if Some(next) == previous {
                return TriangleWalk::Failed;
            }
            previous = Some(current);
            current = next;
        }
        TriangleWalk::Failed
    }

    fn sample_hull(&self, point: Vec2) -> Option<BlendSpaceWeights3> {
        self.hull_edges
            .iter()
            .copied()
            .map(|[a, b]| {
                let (distance, target) =
                    project_to_segment(point, self.points[a].position, self.points[b].position);
                (distance, a, b, target)
            })
            .min_by(|left, right| left.0.total_cmp(&right.0))
            .map(|(_, a, b, target)| {
                BlendSpaceWeights3::new([
                    (self.points[a].sample, 1.0 - target),
                    (self.points[b].sample, target),
                    (self.points[b].sample, 0.0),
                ])
            })
    }

    fn weights(&self, triangle: [usize; 3], weights: [Real; 3]) -> BlendSpaceWeights3 {
        let samples = triangle.map(|index| self.points[index].sample);
        BlendSpaceWeights3::new([
            (samples[0], weights[0]),
            (samples[1], weights[1]),
            (samples[2], weights[2]),
        ])
    }
}

fn compile_topology(
    points: &[PreparedPoint2D],
) -> Result<(Vec<[usize; 3]>, Vec<[Option<usize>; 3]>, Vec<[usize; 2]>), BlendSpaceCompileError> {
    let vertices = points
        .iter()
        .enumerate()
        .map(|(point, source)| TopologyVertex {
            position: Point2::new(source.position.x as f64, source.position.y as f64),
            point,
        })
        .collect();
    let triangulation = DelaunayTriangulation::<TopologyVertex>::bulk_load_stable(vertices)
        .map_err(|_| BlendSpaceCompileError::TopologyFailure)?;
    let mut triangles = triangulation
        .inner_faces()
        .map(|face| face.vertices().map(|vertex| vertex.data().point))
        .collect::<Vec<_>>();
    for triangle in &mut triangles {
        triangle.sort_unstable();
    }
    triangles.sort_unstable();
    if triangles.is_empty() {
        return Err(BlendSpaceCompileError::CollinearPoints);
    }

    let mut neighbors = vec![[None; 3]; triangles.len()];
    let mut unmatched_edges = BTreeMap::<(usize, usize), (usize, usize)>::new();
    let mut hull_edges = Vec::new();
    for (triangle_index, triangle) in triangles.iter().copied().enumerate() {
        for (opposite, edge) in [
            [triangle[1], triangle[2]],
            [triangle[2], triangle[0]],
            [triangle[0], triangle[1]],
        ]
        .into_iter()
        .enumerate()
        {
            let edge = ordered_edge(edge[0], edge[1]);
            if let Some((other_triangle, other_opposite)) = unmatched_edges.remove(&edge) {
                neighbors[triangle_index][opposite] = Some(other_triangle);
                neighbors[other_triangle][other_opposite] = Some(triangle_index);
            } else {
                unmatched_edges.insert(edge, (triangle_index, opposite));
            }
        }
    }
    hull_edges.extend(unmatched_edges.into_keys().map(|(a, b)| [a, b]));
    Ok((triangles, neighbors, hull_edges))
}

fn ordered_edge(a: usize, b: usize) -> (usize, usize) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

#[cfg(test)]
#[path = "tests/blend_space_2d.rs"]
mod tests;
