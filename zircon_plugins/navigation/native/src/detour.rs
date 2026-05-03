use std::os::raw::c_void;
use std::ptr::NonNull;
use std::slice;

use zircon_runtime::asset::NavMeshAsset;
use zircon_runtime::core::framework::navigation::{
    NavPathPoint, NavPathQuery, NavPathResult, NavPathStatus, NavRaycastQuery, NavRaycastResult,
    NavSampleHit, NavSampleQuery,
};
use zircon_runtime::core::math::Real;

use crate::ffi::{
    self, ZrNavDetourAreaCost, ZrNavDetourOffMeshLink, ZrNavDetourPathResult,
    ZrNavDetourQueryCreateResult, ZrNavDetourRaycastResult, ZrNavDetourSampleResult,
    ZrNavRecastBakePolygon,
};

const ZR_NAV_DETOUR_OK: u32 = 1;
const ZR_NAV_DETOUR_NO_PATH: u32 = 2;
const DT_STRAIGHTPATH_OFFMESH_CONNECTION: u8 = 0x04;

pub(crate) fn find_path(asset: &NavMeshAsset, query: &NavPathQuery) -> Option<NavPathResult> {
    let detour_query = DetourQuery::from_asset(asset)?;
    detour_query.find_path(query)
}

pub(crate) fn sample_position(
    asset: &NavMeshAsset,
    query: &NavSampleQuery,
) -> Option<Option<NavSampleHit>> {
    let detour_query = DetourQuery::from_asset(asset)?;
    detour_query.sample_position(query)
}

pub(crate) fn raycast(asset: &NavMeshAsset, query: &NavRaycastQuery) -> Option<NavRaycastResult> {
    let detour_query = DetourQuery::from_asset(asset)?;
    detour_query.raycast(query)
}

struct DetourQuery {
    handle: NonNull<c_void>,
}

impl DetourQuery {
    fn from_asset(asset: &NavMeshAsset) -> Option<Self> {
        if asset.is_empty()
            || asset
                .off_mesh_links
                .iter()
                .any(|link| link.cost_override.is_some())
        {
            return None;
        }

        let vertices = flat_vertices(asset);
        let polygons = detour_polygons(asset);
        let area_costs = detour_area_costs(asset);
        let off_mesh_links = detour_off_mesh_links(asset);
        let mut result = ZrNavDetourQueryCreateResult::default();
        unsafe {
            ffi::zr_nav_detour_create_query(
                vertices.as_ptr(),
                asset.vertices.len() as u32,
                asset.indices.as_ptr(),
                asset.indices.len() as u32,
                polygons.as_ptr(),
                polygons.len() as u32,
                area_costs.as_ptr(),
                area_costs.len() as u32,
                off_mesh_links.as_ptr(),
                off_mesh_links.len() as u32,
                &mut result,
            );
        }
        if result.status != ZR_NAV_DETOUR_OK {
            return None;
        }
        let handle = NonNull::new(result.query)?;
        Some(Self { handle })
    }

    fn find_path(&self, query: &NavPathQuery) -> Option<NavPathResult> {
        let mut result = ZrNavDetourPathResult::default();
        unsafe {
            ffi::zr_nav_detour_find_path(
                self.handle.as_ptr(),
                query.start.as_ptr(),
                query.end.as_ptr(),
                query.area_mask,
                &mut result,
            );
        }
        let converted = match result.status {
            ZR_NAV_DETOUR_OK => self.convert_path_result(&result),
            ZR_NAV_DETOUR_NO_PATH => Some(NavPathResult::no_path()),
            _ => None,
        };
        unsafe {
            ffi::zr_nav_detour_free_path_result(&mut result);
        }
        converted
    }

    fn convert_path_result(&self, result: &ZrNavDetourPathResult) -> Option<NavPathResult> {
        if result.points.is_null() || result.point_count == 0 {
            return None;
        }
        let points = unsafe { slice::from_raw_parts(result.points, result.point_count as usize) }
            .iter()
            .map(|point| NavPathPoint {
                position: point.position,
                area: point.area,
                flags: path_point_flags(point.flags),
            })
            .collect::<Vec<_>>();
        Some(NavPathResult {
            status: NavPathStatus::Complete,
            points,
            length: result.length,
            visited_nodes: (result.visited_nodes as usize).max(1),
        })
    }

    fn sample_position(&self, query: &NavSampleQuery) -> Option<Option<NavSampleHit>> {
        let mut result = ZrNavDetourSampleResult::default();
        unsafe {
            ffi::zr_nav_detour_sample_position(
                self.handle.as_ptr(),
                query.position.as_ptr(),
                query.extents.as_ptr(),
                query.area_mask,
                &mut result,
            );
        }
        (result.status == ZR_NAV_DETOUR_OK).then(|| {
            (result.hit != 0).then_some(NavSampleHit {
                position: result.position,
                distance: result.distance,
                area: result.area,
            })
        })
    }

    fn raycast(&self, query: &NavRaycastQuery) -> Option<NavRaycastResult> {
        let mut result = ZrNavDetourRaycastResult::default();
        unsafe {
            ffi::zr_nav_detour_raycast(
                self.handle.as_ptr(),
                query.start.as_ptr(),
                query.end.as_ptr(),
                query.area_mask,
                &mut result,
            );
        }
        (result.status == ZR_NAV_DETOUR_OK).then_some(NavRaycastResult {
            hit: result.hit != 0,
            position: result.position,
            normal: result.normal,
            distance: result.distance,
        })
    }
}

impl Drop for DetourQuery {
    fn drop(&mut self) {
        unsafe {
            ffi::zr_nav_detour_free_query(self.handle.as_ptr());
        }
    }
}

fn flat_vertices(asset: &NavMeshAsset) -> Vec<Real> {
    let mut vertices = Vec::with_capacity(asset.vertices.len() * 3);
    for vertex in &asset.vertices {
        vertices.extend_from_slice(vertex);
    }
    vertices
}

fn detour_polygons(asset: &NavMeshAsset) -> Vec<ZrNavRecastBakePolygon> {
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

fn detour_area_costs(asset: &NavMeshAsset) -> Vec<ZrNavDetourAreaCost> {
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

fn detour_off_mesh_links(asset: &NavMeshAsset) -> Vec<ZrNavDetourOffMeshLink> {
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

fn path_point_flags(flags: u8) -> Vec<String> {
    if flags & DT_STRAIGHTPATH_OFFMESH_CONNECTION == 0 {
        return Vec::new();
    }
    vec!["off_mesh_link".to_string()]
}
