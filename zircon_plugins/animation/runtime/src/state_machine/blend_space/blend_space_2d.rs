use std::collections::BTreeMap;

use zircon_runtime::core::math::{Real, Vec2};

use super::geometry::{
    barycentric, circumcircle_contains, inside, project_to_segment, triangles_overlap,
};
use super::{BlendSpaceCompileError, BlendSpacePoint2D, BlendSpaceWeights3};

#[derive(Clone, Debug, PartialEq)]
pub struct BlendSpace2D {
    points: Box<[BlendSpacePoint2D]>,
    triangles: Box<[[usize; 3]]>,
}

impl BlendSpace2D {
    pub fn compile(
        points: impl IntoIterator<Item = BlendSpacePoint2D>,
    ) -> Result<Self, BlendSpaceCompileError> {
        let mut points = points.into_iter().collect::<Vec<_>>();
        validate_points(&points)?;
        points.sort_by(|left, right| {
            left.position
                .x
                .total_cmp(&right.position.x)
                .then(left.position.y.total_cmp(&right.position.y))
        });
        let triangles = triangulate(&points)?;
        Ok(Self {
            points: points.into_boxed_slice(),
            triangles: triangles.into_boxed_slice(),
        })
    }

    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    pub fn sample(&self, point: Vec2) -> Option<BlendSpaceWeights3> {
        if !point.is_finite() {
            return None;
        }
        for triangle in &self.triangles {
            let positions = triangle.map(|index| self.points[index].position);
            let weights = barycentric(point, positions[0], positions[1], positions[2])?;
            if inside(weights) {
                return Some(self.weights(*triangle, weights));
            }
        }
        self.sample_hull(point)
    }

    fn sample_hull(&self, point: Vec2) -> Option<BlendSpaceWeights3> {
        let mut edges = BTreeMap::<(usize, usize), usize>::new();
        for triangle in &self.triangles {
            for [a, b] in [
                [triangle[0], triangle[1]],
                [triangle[1], triangle[2]],
                [triangle[2], triangle[0]],
            ] {
                *edges.entry(ordered_edge(a, b)).or_default() += 1;
            }
        }
        edges
            .into_iter()
            .filter(|(_, count)| *count == 1)
            .map(|((a, b), _)| {
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

fn validate_points(points: &[BlendSpacePoint2D]) -> Result<(), BlendSpaceCompileError> {
    if points.len() < 3 {
        return Err(BlendSpaceCompileError::Empty);
    }
    if points.iter().any(|point| !point.position.is_finite()) {
        return Err(BlendSpaceCompileError::NonFinitePoint);
    }
    for (index, point) in points.iter().enumerate() {
        if points[index + 1..]
            .iter()
            .any(|other| other.position == point.position)
        {
            return Err(BlendSpaceCompileError::DuplicatePoint);
        }
    }
    Ok(())
}

fn triangulate(points: &[BlendSpacePoint2D]) -> Result<Vec<[usize; 3]>, BlendSpaceCompileError> {
    let mut triangles = Vec::new();
    for a in 0..points.len() {
        for b in a + 1..points.len() {
            for c in b + 1..points.len() {
                let triangle = [a, b, c];
                if barycentric(
                    points[a].position,
                    points[a].position,
                    points[b].position,
                    points[c].position,
                )
                .is_none()
                {
                    continue;
                }
                if !points.iter().enumerate().any(|(index, point)| {
                    !triangle.contains(&index)
                        && circumcircle_contains(point.position, triangle, points)
                }) {
                    triangles.push(triangle);
                }
            }
        }
    }
    let mut selected = Vec::with_capacity(triangles.len());
    for triangle in triangles {
        if selected
            .iter()
            .all(|existing| !triangles_overlap(*existing, triangle, points))
        {
            selected.push(triangle);
        }
    }
    if selected.is_empty() {
        return Err(BlendSpaceCompileError::CollinearPoints);
    }
    Ok(selected)
}

fn ordered_edge(a: usize, b: usize) -> (usize, usize) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}
