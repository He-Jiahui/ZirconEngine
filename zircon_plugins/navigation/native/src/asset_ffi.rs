use zircon_runtime::core::framework::navigation::NavMeshAsset;
use zircon_runtime::core::framework::navigation::NavQueryFilter;
use zircon_runtime::core::math::Real;

use crate::ffi::{ZrNavDetourAreaCost, ZrNavDetourOffMeshLink, ZrNavRecastBakePolygon};

pub(crate) fn asset_query_filter(asset: &NavMeshAsset) -> NavQueryFilter {
    let mut filter = NavQueryFilter::default();
    for area_cost in &asset.area_costs {
        if let Some(cost) = filter.area_costs.get_mut(area_cost.area as usize) {
            if area_cost.cost.is_finite() && area_cost.cost > 0.0 {
                *cost = area_cost.cost;
            }
        }
    }
    filter
}

pub(crate) fn flat_vertices(asset: &NavMeshAsset) -> Vec<Real> {
    let mut vertices = Vec::with_capacity(asset.vertices.len() * 3);
    for vertex in &asset.vertices {
        vertices.extend_from_slice(vertex);
    }
    vertices
}

pub(crate) fn detour_polygons(asset: &NavMeshAsset) -> Vec<ZrNavRecastBakePolygon> {
    asset
        .polygons
        .iter()
        .map(|polygon| ZrNavRecastBakePolygon {
            first_index: polygon.first_index,
            index_count: polygon.index_count,
            area: polygon.area,
            tile: polygon.tile,
        })
        .collect()
}

pub(crate) fn detour_area_costs(asset: &NavMeshAsset) -> Vec<ZrNavDetourAreaCost> {
    asset
        .area_costs
        .iter()
        .map(|cost| ZrNavDetourAreaCost {
            area: cost.area,
            cost: cost.cost,
            walkable: u8::from(cost.walkable),
        })
        .collect()
}

pub(crate) fn detour_off_mesh_links(asset: &NavMeshAsset) -> Vec<ZrNavDetourOffMeshLink> {
    asset
        .off_mesh_links
        .iter()
        .map(|link| ZrNavDetourOffMeshLink {
            user_id: link.id,
            start: link.start,
            end: link.end,
            radius: link.width.max(0.05),
            bidirectional: u8::from(link.bidirectional),
            area: link.area,
        })
        .collect()
}
