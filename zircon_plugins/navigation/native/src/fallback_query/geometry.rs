use zircon_runtime::asset::{NavMeshAsset, NavMeshPolygonAsset};
use zircon_runtime::core::framework::navigation::NavQueryFilter;
use zircon_runtime::core::math::Real;

pub(super) fn area_allowed(
    asset: &NavMeshAsset,
    mask: u64,
    filter: &NavQueryFilter,
    area: u8,
) -> bool {
    if area >= 64 || (mask & (1_u64 << area)) == 0 || !filter.allows_area(area) {
        return false;
    }
    asset
        .area_costs
        .iter()
        .find(|cost| cost.area == area)
        .map(|cost| cost.walkable)
        .unwrap_or(area != 0)
}

pub(super) fn area_cost(filter: &NavQueryFilter, area: u8) -> Real {
    filter
        .area_costs
        .get(area as usize)
        .copied()
        .filter(|cost| cost.is_finite() && *cost > 0.0)
        .unwrap_or(1.0)
}

pub(super) fn nearest_allowed_polygon(
    asset: &NavMeshAsset,
    position: [Real; 3],
    mask: u64,
    filter: &NavQueryFilter,
) -> Option<usize> {
    let mut best_inside = None;
    let mut best_distance = Real::INFINITY;
    for (index, polygon) in asset.polygons.iter().enumerate() {
        if !area_allowed(asset, mask, filter, polygon.area) {
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

pub(super) fn closest_point_on_polygon_xz(
    asset: &NavMeshAsset,
    polygon: &NavMeshPolygonAsset,
    point: [Real; 3],
) -> Option<[Real; 3]> {
    let indices = polygon_indices(asset, polygon);
    let mut best = None;
    let mut best_distance = Real::INFINITY;
    for triangle in indices.chunks(3).filter(|triangle| triangle.len() == 3) {
        let Some(sample) = closest_point_on_triangle_xz(asset, triangle, point) else {
            continue;
        };
        let distance = distance(point, sample);
        if distance < best_distance {
            best_distance = distance;
            best = Some(sample);
        }
    }
    best
}

pub(super) fn shared_vertex_count(
    asset: &NavMeshAsset,
    left: &NavMeshPolygonAsset,
    right: &NavMeshPolygonAsset,
) -> usize {
    let left_indices = unique_polygon_indices(asset, left);
    let right_indices = unique_polygon_indices(asset, right);
    left_indices
        .iter()
        .filter(|index| right_indices.contains(index))
        .count()
}

pub(super) fn point_in_polygon_xz(
    asset: &NavMeshAsset,
    polygon: &NavMeshPolygonAsset,
    point: [Real; 3],
) -> bool {
    let indices = polygon_indices(asset, polygon);
    indices
        .chunks(3)
        .any(|triangle| triangle.len() == 3 && point_in_triangle_xz(asset, triangle, point))
}

pub(super) fn polygon_centroid(
    asset: &NavMeshAsset,
    polygon: &NavMeshPolygonAsset,
) -> Option<[Real; 3]> {
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

pub(super) fn distance(from: [Real; 3], to: [Real; 3]) -> Real {
    let delta = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
    (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt()
}

pub(super) fn lerp(from: [Real; 3], to: [Real; 3], t: Real) -> [Real; 3] {
    [
        from[0] + (to[0] - from[0]) * t,
        from[1] + (to[1] - from[1]) * t,
        from[2] + (to[2] - from[2]) * t,
    ]
}

fn closest_point_on_triangle_xz(
    asset: &NavMeshAsset,
    indices: &[usize],
    point: [Real; 3],
) -> Option<[Real; 3]> {
    let a = asset.vertices.get(indices[0]).copied()?;
    let b = asset.vertices.get(indices[1]).copied()?;
    let c = asset.vertices.get(indices[2]).copied()?;
    if point_in_triangle_xz(asset, indices, point) {
        let weights = barycentric_xz(a, b, c, point)?;
        return Some(interpolate_triangle(a, b, c, weights));
    }
    [
        closest_point_on_segment_xz(a, b, point),
        closest_point_on_segment_xz(b, c, point),
        closest_point_on_segment_xz(c, a, point),
    ]
    .into_iter()
    .min_by(|left, right| distance(point, *left).total_cmp(&distance(point, *right)))
}

fn closest_point_on_segment_xz(a: [Real; 3], b: [Real; 3], point: [Real; 3]) -> [Real; 3] {
    let ab = [b[0] - a[0], b[2] - a[2]];
    let ap = [point[0] - a[0], point[2] - a[2]];
    let length_sq = ab[0] * ab[0] + ab[1] * ab[1];
    let t = if length_sq <= Real::EPSILON {
        0.0
    } else {
        ((ap[0] * ab[0] + ap[1] * ab[1]) / length_sq).clamp(0.0, 1.0)
    };
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn unique_polygon_indices(asset: &NavMeshAsset, polygon: &NavMeshPolygonAsset) -> Vec<usize> {
    let mut unique = Vec::new();
    for index in polygon_indices(asset, polygon) {
        if !unique.contains(&index) {
            unique.push(index);
        }
    }
    unique
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
    let Some((u, v, w)) = barycentric_xz(a, b, c, point) else {
        return false;
    };
    u >= -Real::EPSILON && v >= -Real::EPSILON && w >= -Real::EPSILON
}

fn barycentric_xz(
    a: [Real; 3],
    b: [Real; 3],
    c: [Real; 3],
    point: [Real; 3],
) -> Option<(Real, Real, Real)> {
    let p = [point[0], point[2]];
    let a = [a[0], a[2]];
    let b = [b[0], b[2]];
    let c = [c[0], c[2]];
    let denominator = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
    if denominator.abs() <= Real::EPSILON {
        return None;
    }
    let u = ((b[1] - c[1]) * (p[0] - c[0]) + (c[0] - b[0]) * (p[1] - c[1])) / denominator;
    let v = ((c[1] - a[1]) * (p[0] - c[0]) + (a[0] - c[0]) * (p[1] - c[1])) / denominator;
    let w = 1.0 - u - v;
    Some((u, v, w))
}

fn interpolate_triangle(
    a: [Real; 3],
    b: [Real; 3],
    c: [Real; 3],
    (u, v, w): (Real, Real, Real),
) -> [Real; 3] {
    [
        a[0] * u + b[0] * v + c[0] * w,
        a[1] * u + b[1] * v + c[1] * w,
        a[2] * u + b[2] * v + c[2] * w,
    ]
}

fn polygon_indices(asset: &NavMeshAsset, polygon: &NavMeshPolygonAsset) -> Vec<usize> {
    let start = polygon.first_index as usize;
    let end = start.saturating_add(polygon.index_count as usize);
    asset.indices[start.min(asset.indices.len())..end.min(asset.indices.len())]
        .iter()
        .map(|index| *index as usize)
        .collect()
}

fn distance_xz(from: [Real; 3], to: [Real; 3]) -> Real {
    let delta = [to[0] - from[0], to[2] - from[2]];
    (delta[0] * delta[0] + delta[1] * delta[1]).sqrt()
}
