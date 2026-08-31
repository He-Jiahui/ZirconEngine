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
    pub(super) fn from_compiled(
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
mod tests {
    use std::f32::consts::TAU;

    use zircon_runtime::asset::{AssetReference, AssetUri};
    use zircon_runtime::core::framework::animation::compiler::state_machine::AnimationCompiledBlendSpace2DSample;

    use super::*;

    #[test]
    fn stable_delaunay_prepares_complete_cocircular_hull() {
        const SAMPLE_COUNT: usize = 96;
        let samples = (0..SAMPLE_COUNT)
            .map(|index| {
                let angle = TAU * index as f32 / SAMPLE_COUNT as f32;
                sample([angle.cos(), angle.sin()], index)
            })
            .collect::<Vec<_>>();

        let blend = BlendSpace2D::from_compiled(&samples).unwrap();

        assert_eq!(blend.triangle_count(), SAMPLE_COUNT - 2);
        assert_eq!(blend.hull_edges.len(), SAMPLE_COUNT);
        assert_eq!(
            blend
                .neighbors
                .iter()
                .flat_map(|neighbors| neighbors.iter())
                .filter(|neighbor| neighbor.is_some())
                .count(),
            2 * (SAMPLE_COUNT - 3)
        );
        let weights = blend.sample(Vec2::ZERO).unwrap();
        assert!((weights.weight_sum() - 1.0).abs() <= 1.0e-5);
    }

    #[test]
    fn prepared_hull_sampling_does_not_rebuild_an_edge_map() {
        let source = include_str!("blend_space_2d.rs");

        assert!(source.contains("hull_edges: Box<[[usize; 2]]>"));
        assert!(!source.contains("fn sample_hull(&self, point: Vec2) -> Option<BlendSpaceWeights3> {\n        let mut edges"));
    }

    #[test]
    fn outside_hull_sampling_retains_boundary_triangle_hint() {
        let samples = [
            sample([-1.0, -1.0], 0),
            sample([1.0, -1.0], 1),
            sample([1.0, 1.0], 2),
            sample([-1.0, 1.0], 3),
        ];
        let blend = BlendSpace2D::from_compiled(&samples).unwrap();

        let (weights, hint) = blend.sample_with_hint(Vec2::new(2.0, 0.25), None).unwrap();
        let hint = hint.expect("outside-hull sampling retains the boundary triangle");
        let (_, repeated_hint) = blend
            .sample_with_hint(Vec2::new(2.0, 0.25), Some(hint))
            .unwrap();

        assert!((weights.weight_sum() - 1.0).abs() <= 1.0e-5);
        assert_eq!(repeated_hint, Some(hint));
    }

    #[test]
    fn retained_walk_matches_exhaustive_sampling_inside_and_outside_the_hull() {
        let samples = [
            sample([-1.0, -1.0], 0),
            sample([0.0, -0.8], 1),
            sample([1.0, -1.0], 2),
            sample([-0.9, 0.1], 3),
            sample([0.1, 0.0], 4),
            sample([0.8, 0.3], 5),
            sample([-0.7, 1.0], 6),
            sample([0.2, 0.9], 7),
            sample([1.0, 1.0], 8),
        ];
        let blend = BlendSpace2D::from_compiled(&samples).unwrap();
        let mut hint = None;
        for y in -16..=16 {
            for x in -16..=16 {
                let point = Vec2::new(x as Real * 0.1, y as Real * 0.1);
                let (walked, next_hint) = blend.sample_with_hint(point, hint).unwrap();
                let (exhaustive, _) = blend.sample_after_failed_walk(point).unwrap();

                assert_weight_maps_close(walked, exhaustive, samples.len(), point);
                hint = next_hint;
            }
        }
    }

    fn assert_weight_maps_close(
        actual: BlendSpaceWeights3,
        expected: BlendSpaceWeights3,
        sample_count: usize,
        point: Vec2,
    ) {
        let mut actual_weights = vec![0.0; sample_count];
        let mut expected_weights = vec![0.0; sample_count];
        for (sample, weight) in actual.as_pairs() {
            actual_weights[sample as usize] += weight;
        }
        for (sample, weight) in expected.as_pairs() {
            expected_weights[sample as usize] += weight;
        }
        for (actual, expected) in actual_weights.into_iter().zip(expected_weights) {
            assert!(
                (actual - expected).abs() <= 1.0e-5,
                "walked and exhaustive weights differ at {point:?}: {actual} != {expected}"
            );
        }
    }

    fn sample(position: [f32; 2], index: usize) -> AnimationCompiledBlendSpace2DSample {
        AnimationCompiledBlendSpace2DSample {
            position: Vec2::from_array(position),
            graph: AssetReference::from_locator(
                AssetUri::parse(&format!("res://animation/direction-{index}.zranim")).unwrap(),
            ),
        }
    }
}
