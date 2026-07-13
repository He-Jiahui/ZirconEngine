use zircon_runtime::core::framework::navigation::NavMeshAsset;
use zircon_runtime::core::framework::navigation::{
    NavQueryFilter, NavSampleHit, NavSampleQuery, AREA_WALKABLE,
};
use zircon_runtime::core::math::Real;

use super::geometry::{area_allowed, closest_point_on_polygon_xz, distance};

pub(crate) fn sample_position(
    asset: &NavMeshAsset,
    query: &NavSampleQuery,
) -> Option<NavSampleHit> {
    let (polygon, position, distance) =
        nearest_allowed_polygon_sample(asset, query.position, query.area_mask)?;
    let inside_extents = distance <= query.extents[0].max(query.extents[1]).max(query.extents[2]);
    inside_extents.then_some(NavSampleHit {
        position,
        distance,
        area: asset
            .polygons
            .get(polygon)
            .map(|polygon| polygon.area)
            .unwrap_or(AREA_WALKABLE),
    })
}

fn nearest_allowed_polygon_sample(
    asset: &NavMeshAsset,
    position: [Real; 3],
    mask: u64,
) -> Option<(usize, [Real; 3], Real)> {
    let mut best = None;
    let mut best_distance = Real::INFINITY;
    let filter = NavQueryFilter::default();
    for (index, polygon) in asset.polygons.iter().enumerate() {
        if !area_allowed(asset, mask, &filter, polygon.area) {
            continue;
        }
        if let Some(sample) = closest_point_on_polygon_xz(asset, polygon, position) {
            let distance = distance(position, sample);
            if distance < best_distance {
                best_distance = distance;
                best = Some((index, sample, distance));
            }
        }
    }
    best
}
