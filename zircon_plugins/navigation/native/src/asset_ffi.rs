use zircon_runtime::asset::NavMeshAsset;
use zircon_runtime::core::math::Real;

use crate::ffi::{ZrNavDetourAreaCost, ZrNavDetourOffMeshLink, ZrNavRecastBakePolygon};

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
            start: link.start,
            end: link.end,
            radius: link.width.max(0.05),
            bidirectional: u8::from(link.bidirectional),
            area: link.area,
        })
        .collect()
}
