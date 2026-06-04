use std::os::raw::c_void;
use std::ptr::NonNull;

use zircon_runtime::asset::NavMeshAsset;
use zircon_runtime::core::framework::navigation::{NavPathQuery, NavPathResult};
use zircon_runtime::core::math::Real;

use crate::asset_ffi::{detour_area_costs, detour_off_mesh_links, detour_polygons, flat_vertices};
use crate::detour_result::convert_path_result;
use crate::ffi::{
    self, ZrNavDetourPathResult, ZrNavDetourTileCacheCreateResult, ZrNavDetourTileCacheObstacle,
};

const ZR_NAV_DETOUR_OK: u32 = 1;
const ZR_NAV_DETOUR_NO_PATH: u32 = 2;
const ZR_NAV_TILE_CACHE_SHAPE_CYLINDER: u8 = 0;
const ZR_NAV_TILE_CACHE_SHAPE_BOX: u8 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecastNavigationObstacleShape {
    Cylinder,
    Box,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RecastNavigationObstacle {
    pub shape: RecastNavigationObstacleShape,
    pub center: [Real; 3],
    pub half_extents: [Real; 3],
    pub radius: Real,
    pub height: Real,
}

impl RecastNavigationObstacle {
    pub fn cylinder(center: [Real; 3], radius: Real, height: Real) -> Self {
        Self {
            shape: RecastNavigationObstacleShape::Cylinder,
            center,
            half_extents: [radius.max(0.05), height.max(0.05) * 0.5, radius.max(0.05)],
            radius,
            height,
        }
    }

    pub fn box_obstacle(center: [Real; 3], half_extents: [Real; 3]) -> Self {
        Self {
            shape: RecastNavigationObstacleShape::Box,
            center,
            half_extents,
            radius: half_extents[0].abs().max(half_extents[2].abs()),
            height: half_extents[1].abs() * 2.0,
        }
    }
}

pub(crate) fn find_path(
    asset: &NavMeshAsset,
    query: &NavPathQuery,
    obstacles: &[RecastNavigationObstacle],
) -> Option<NavPathResult> {
    let tile_cache = TileCacheQuery::from_asset(asset, obstacles)?;
    tile_cache.find_path(query)
}

struct TileCacheQuery {
    handle: NonNull<c_void>,
}

impl TileCacheQuery {
    fn from_asset(asset: &NavMeshAsset, obstacles: &[RecastNavigationObstacle]) -> Option<Self> {
        if asset.is_empty()
            || obstacles.is_empty()
            || !asset.off_mesh_links.is_empty()
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
        let obstacles = detour_obstacles(obstacles);
        let mut result = ZrNavDetourTileCacheCreateResult::default();
        unsafe {
            ffi::zr_nav_tile_cache_create_query(
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
                obstacles.as_ptr(),
                obstacles.len() as u32,
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
            ffi::zr_nav_tile_cache_find_path(
                self.handle.as_ptr(),
                query.start.as_ptr(),
                query.end.as_ptr(),
                query.area_mask,
                &mut result,
            );
        }
        let converted = match result.status {
            ZR_NAV_DETOUR_OK => convert_path_result(&result),
            ZR_NAV_DETOUR_NO_PATH => Some(NavPathResult::no_path()),
            _ => None,
        };
        unsafe {
            ffi::zr_nav_detour_free_path_result(&mut result);
        }
        converted
    }
}

impl Drop for TileCacheQuery {
    fn drop(&mut self) {
        unsafe {
            ffi::zr_nav_tile_cache_free_query(self.handle.as_ptr());
        }
    }
}

fn detour_obstacles(obstacles: &[RecastNavigationObstacle]) -> Vec<ZrNavDetourTileCacheObstacle> {
    obstacles
        .iter()
        .map(|obstacle| ZrNavDetourTileCacheObstacle {
            center: obstacle.center,
            half_extents: obstacle.half_extents,
            radius: obstacle.radius,
            height: obstacle.height,
            shape: match obstacle.shape {
                RecastNavigationObstacleShape::Cylinder => ZR_NAV_TILE_CACHE_SHAPE_CYLINDER,
                RecastNavigationObstacleShape::Box => ZR_NAV_TILE_CACHE_SHAPE_BOX,
            },
        })
        .collect()
}
