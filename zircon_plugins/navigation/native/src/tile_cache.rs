use std::os::raw::c_void;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime::core::framework::navigation::NavMeshAsset;
use zircon_runtime::core::framework::navigation::{NavPathQuery, NavPathResult, NavQueryFilter};
use zircon_runtime::core::math::Real;

use crate::asset_ffi::{
    asset_query_filter, detour_area_costs, detour_off_mesh_links, detour_polygons, flat_vertices,
};
use crate::detour_result::convert_path_result;
use crate::ffi::{
    self, ZrNavDetourPathResult, ZrNavDetourTileCacheCommandResult,
    ZrNavDetourTileCacheCreateResult, ZrNavDetourTileCacheObstacle,
};

const ZR_NAV_DETOUR_OK: u32 = 1;
const ZR_NAV_DETOUR_NO_PATH: u32 = 2;
const ZR_NAV_TILE_CACHE_MAX_PENDING_REQUESTS: usize = 64;
const ZR_NAV_TILE_CACHE_SHAPE_CYLINDER: u8 = 0;
const ZR_NAV_TILE_CACHE_SHAPE_BOX: u8 = 1;
static NEXT_TILE_CACHE_ID: AtomicU64 = AtomicU64::new(1);

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
    let tile_cache = RecastTileCache::from_asset_with_obstacles(asset, obstacles)?;
    Some(tile_cache.find_path(query))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RecastTileCacheObstacleHandle {
    cache_id: u64,
    obstacle_ref: u64,
}

impl RecastTileCacheObstacleHandle {
    pub const INVALID: Self = Self {
        cache_id: 0,
        obstacle_ref: 0,
    };
}

/// Owns a mutable Detour TileCache world. Mutation requires `&mut self`; the native object is
/// never shared concurrently and can therefore move between manager-owned worker contexts.
pub struct RecastTileCache {
    handle: NonNull<c_void>,
    cache_id: u64,
    asset_filter: NavQueryFilter,
    pending_requests: usize,
}

impl std::fmt::Debug for RecastTileCache {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RecastTileCache")
            .finish_non_exhaustive()
    }
}

unsafe impl Send for RecastTileCache {}

impl RecastTileCache {
    pub fn from_asset(asset: &NavMeshAsset) -> Option<Self> {
        Self::from_asset_with_obstacles(asset, &[])
    }

    fn from_asset_with_obstacles(
        asset: &NavMeshAsset,
        obstacles: &[RecastNavigationObstacle],
    ) -> Option<Self> {
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
        let obstacles = obstacles
            .iter()
            .copied()
            .map(detour_obstacle)
            .collect::<Vec<_>>();
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
        Some(Self {
            handle,
            cache_id: next_tile_cache_id(),
            asset_filter: asset_query_filter(asset),
            pending_requests: 0,
        })
    }

    pub fn add_obstacle(
        &mut self,
        obstacle: RecastNavigationObstacle,
    ) -> Option<RecastTileCacheObstacleHandle> {
        self.flush_pending_requests_if_full().ok()?;
        let obstacle = detour_obstacle(obstacle);
        let mut result = ZrNavDetourTileCacheCommandResult::default();
        unsafe {
            ffi::zr_nav_tile_cache_add_obstacle(self.handle.as_ptr(), &obstacle, &mut result);
        }
        if result.status == ZR_NAV_DETOUR_OK && result.obstacle_ref != 0 {
            self.pending_requests += 1;
            Some(RecastTileCacheObstacleHandle {
                cache_id: self.cache_id,
                obstacle_ref: result.obstacle_ref,
            })
        } else {
            None
        }
    }

    pub fn remove_obstacle(
        &mut self,
        obstacle: RecastTileCacheObstacleHandle,
    ) -> Result<(), &'static str> {
        if obstacle.cache_id != self.cache_id || obstacle.obstacle_ref == 0 {
            return Err("Detour TileCache obstacle handle belongs to another cache");
        }
        self.flush_pending_requests_if_full()?;
        let mut result = ZrNavDetourTileCacheCommandResult::default();
        unsafe {
            ffi::zr_nav_tile_cache_remove_obstacle(
                self.handle.as_ptr(),
                obstacle.obstacle_ref,
                &mut result,
            );
        }
        if result.status == ZR_NAV_DETOUR_OK {
            self.pending_requests += 1;
            Ok(())
        } else {
            Err("Detour TileCache could not remove obstacle")
        }
    }

    pub fn update(&mut self) -> Result<(), &'static str> {
        let mut result = ZrNavDetourTileCacheCommandResult::default();
        unsafe {
            ffi::zr_nav_tile_cache_update(self.handle.as_ptr(), &mut result);
        }
        if result.status == ZR_NAV_DETOUR_OK {
            self.pending_requests = 0;
            Ok(())
        } else {
            Err("Detour TileCache update failed")
        }
    }

    pub fn find_path(&self, query: &NavPathQuery) -> NavPathResult {
        self.find_path_with_filter(query, &self.asset_filter)
    }

    pub fn find_path_with_filter(
        &self,
        query: &NavPathQuery,
        filter: &NavQueryFilter,
    ) -> NavPathResult {
        let mut result = ZrNavDetourPathResult::default();
        let filter = crate::detour::detour_query_filter(filter);
        unsafe {
            ffi::zr_nav_tile_cache_find_path(
                self.handle.as_ptr(),
                query.start.as_ptr(),
                query.end.as_ptr(),
                query.area_mask,
                &filter,
                &mut result,
            );
        }
        let converted = match result.status {
            ZR_NAV_DETOUR_OK => convert_path_result(&result),
            ZR_NAV_DETOUR_NO_PATH => Some(NavPathResult::no_path()),
            _ => None,
        }
        .unwrap_or_else(NavPathResult::no_path);
        unsafe {
            ffi::zr_nav_detour_free_path_result(&mut result);
        }
        converted
    }

    fn flush_pending_requests_if_full(&mut self) -> Result<(), &'static str> {
        if self.pending_requests >= ZR_NAV_TILE_CACHE_MAX_PENDING_REQUESTS {
            self.update()?;
        }
        Ok(())
    }
}

fn next_tile_cache_id() -> u64 {
    loop {
        let id = NEXT_TILE_CACHE_ID.fetch_add(1, Ordering::Relaxed);
        if id != 0 {
            return id;
        }
    }
}

impl Drop for RecastTileCache {
    fn drop(&mut self) {
        unsafe {
            ffi::zr_nav_tile_cache_free_query(self.handle.as_ptr());
        }
    }
}

fn detour_obstacle(obstacle: RecastNavigationObstacle) -> ZrNavDetourTileCacheObstacle {
    ZrNavDetourTileCacheObstacle {
        center: obstacle.center,
        half_extents: obstacle.half_extents,
        radius: obstacle.radius,
        height: obstacle.height,
        shape: match obstacle.shape {
            RecastNavigationObstacleShape::Cylinder => ZR_NAV_TILE_CACHE_SHAPE_CYLINDER,
            RecastNavigationObstacleShape::Box => ZR_NAV_TILE_CACHE_SHAPE_BOX,
        },
    }
}
