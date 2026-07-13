use zircon_runtime::core::math::{Real, Vec2};

use super::BlendSpacePoint2D;

pub(super) fn barycentric(point: Vec2, a: Vec2, b: Vec2, c: Vec2) -> Option<[Real; 3]> {
    let denominator = cross(b - a, c - a);
    (denominator.abs() > Real::EPSILON).then(|| {
        let u = cross(b - point, c - point) / denominator;
        let v = cross(c - point, a - point) / denominator;
        [u, v, 1.0 - u - v]
    })
}

pub(super) fn inside(weights: [Real; 3]) -> bool {
    weights.iter().all(|weight| *weight >= -Real::EPSILON)
}

pub(super) fn circumcircle_contains(
    point: Vec2,
    triangle: [usize; 3],
    points: &[BlendSpacePoint2D],
) -> bool {
    let [a, b, c] = triangle.map(|index| points[index].position - point);
    let determinant = (a.length_squared() * cross(b, c)) - (b.length_squared() * cross(a, c))
        + (c.length_squared() * cross(a, b));
    let orientation = cross(b - a, c - a);
    if orientation > 0.0 {
        determinant > Real::EPSILON
    } else {
        determinant < -Real::EPSILON
    }
}

pub(super) fn triangles_overlap(
    left: [usize; 3],
    right: [usize; 3],
    points: &[BlendSpacePoint2D],
) -> bool {
    let shared = left
        .iter()
        .copied()
        .filter(|index| right.contains(index))
        .collect::<Vec<_>>();
    if shared.len() == 2 {
        let left_other = left.iter().copied().find(|index| !shared.contains(index));
        let right_other = right.iter().copied().find(|index| !shared.contains(index));
        let (Some(left_other), Some(right_other)) = (left_other, right_other) else {
            return true;
        };
        let edge_start = points[shared[0]].position;
        let edge = points[shared[1]].position - edge_start;
        let left_side = cross(edge, points[left_other].position - edge_start);
        let right_side = cross(edge, points[right_other].position - edge_start);
        return left_side * right_side > 0.0;
    }
    let left_positions = left.map(|index| points[index].position);
    let right_positions = right.map(|index| points[index].position);
    let left_centroid = (left_positions[0] + left_positions[1] + left_positions[2]) / 3.0;
    let right_centroid = (right_positions[0] + right_positions[1] + right_positions[2]) / 3.0;
    strictly_inside(left_centroid, right_positions)
        || strictly_inside(right_centroid, left_positions)
}

fn strictly_inside(point: Vec2, triangle: [Vec2; 3]) -> bool {
    barycentric(point, triangle[0], triangle[1], triangle[2])
        .is_some_and(|weights| weights.iter().all(|weight| *weight > Real::EPSILON))
}

pub(super) fn project_to_segment(point: Vec2, a: Vec2, b: Vec2) -> (Real, Real) {
    let edge = b - a;
    let denominator = edge.length_squared();
    let target = if denominator > Real::EPSILON {
        ((point - a).dot(edge) / denominator).clamp(0.0, 1.0)
    } else {
        0.0
    };
    ((point - a.lerp(b, target)).length_squared(), target)
}

fn cross(left: Vec2, right: Vec2) -> Real {
    left.x * right.y - left.y * right.x
}
