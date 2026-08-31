use zircon_runtime::core::math::{Real, Vec2};

pub(super) fn barycentric(point: Vec2, a: Vec2, b: Vec2, c: Vec2) -> Option<[Real; 3]> {
    let denominator = cross(delta(b, a), delta(c, a));
    (denominator != 0.0).then(|| {
        let u = cross(delta(b, point), delta(c, point)) / denominator;
        let v = cross(delta(c, point), delta(a, point)) / denominator;
        [u as Real, v as Real, (1.0 - u - v) as Real]
    })
}

pub(super) fn inside(weights: [Real; 3]) -> bool {
    weights.iter().all(|weight| *weight >= -Real::EPSILON)
}

pub(super) fn project_to_segment(point: Vec2, a: Vec2, b: Vec2) -> (f64, Real) {
    let edge = delta(b, a);
    let point_from_a = delta(point, a);
    let denominator = dot(edge, edge);
    let target = if denominator > 0.0 {
        (dot(point_from_a, edge) / denominator).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let projected_delta = [
        point_from_a[0] - edge[0] * target,
        point_from_a[1] - edge[1] * target,
    ];
    (dot(projected_delta, projected_delta), target as Real)
}

fn delta(left: Vec2, right: Vec2) -> [f64; 2] {
    [
        f64::from(left.x) - f64::from(right.x),
        f64::from(left.y) - f64::from(right.y),
    ]
}

fn cross(left: [f64; 2], right: [f64; 2]) -> f64 {
    left[0] * right[1] - left[1] * right[0]
}

fn dot(left: [f64; 2], right: [f64; 2]) -> f64 {
    left[0] * right[0] + left[1] * right[1]
}
