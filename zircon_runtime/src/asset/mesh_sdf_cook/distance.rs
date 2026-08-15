use crate::core::math::Vec3;

pub(super) fn point_triangle_distance_squared(point: Vec3, a: Vec3, b: Vec3, c: Vec3) -> f32 {
    let ab = b - a;
    let ac = c - a;
    let ap = point - a;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return ap.length_squared();
    }

    let bp = point - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return bp.length_squared();
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let projection = d1 / (d1 - d3);
        return (point - (a + projection * ab)).length_squared();
    }

    let cp = point - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return cp.length_squared();
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let projection = d2 / (d2 - d6);
        return (point - (a + projection * ac)).length_squared();
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let projection = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return (point - (b + projection * (c - b))).length_squared();
    }

    let denominator = 1.0 / (va + vb + vc);
    let v = vb * denominator;
    let w = vc * denominator;
    (point - (a + ab * v + ac * w)).length_squared()
}

pub(super) fn positive_x_ray_intersects_triangle(origin: Vec3, a: Vec3, b: Vec3, c: Vec3) -> bool {
    const EPSILON: f32 = 1.0e-7;
    let direction = Vec3::X;
    let edge_ab = b - a;
    let edge_ac = c - a;
    let cross = direction.cross(edge_ac);
    let determinant = edge_ab.dot(cross);
    if determinant.abs() <= EPSILON {
        return false;
    }
    let inverse = determinant.recip();
    let offset = origin - a;
    let u = inverse * offset.dot(cross);
    if !(-EPSILON..=1.0 + EPSILON).contains(&u) {
        return false;
    }
    let q = offset.cross(edge_ab);
    let v = inverse * direction.dot(q);
    if v < -EPSILON || u + v > 1.0 + EPSILON {
        return false;
    }
    inverse * edge_ac.dot(q) > EPSILON
}
