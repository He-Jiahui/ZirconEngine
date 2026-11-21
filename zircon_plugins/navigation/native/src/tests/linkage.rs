use std::ffi::{c_void, CStr};
use std::ptr::NonNull;

use zircon_runtime::core::framework::navigation::{
    NavMeshAsset, AREA_JUMP, AREA_WALKABLE, DEFAULT_AREA_MASK,
};

use super::support::two_island_asset;
use crate::asset_ffi::{
    asset_query_filter, detour_area_costs, detour_off_mesh_links, detour_polygons, flat_vertices,
};
use crate::detour::{detour_query_filter, DetourQuery};
use crate::ffi::{self, ZrNavDetourPathResult, ZrNavDetourQueryFilter};
use crate::{native_backend_version, native_runtime_modules_available};

#[test]
fn native_recast_detour_modules_are_linked() {
    assert_eq!(native_backend_version(), 4);
    assert!(native_runtime_modules_available());
}

#[test]
fn ordinary_detour_does_not_connect_unlinked_islands_in_either_order() {
    assert_island_routes(false, false);
}

#[test]
fn ordinary_detour_crosses_only_with_jump_in_either_order() {
    assert_island_routes(false, true);
}

#[test]
fn native_tile_cache_controls_keep_borders_and_jump_filter() {
    assert_island_routes(true, false);
    assert_island_routes(true, true);
}

#[test]
fn ordinary_detour_keeps_the_shared_edge_to_polygon_zero() {
    // Both upward triangles use the same diagonal in opposite directions as edge 0.
    let asset = NavMeshAsset::from_triangle_mesh(
        "humanoid",
        vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 2.0],
            [2.0, 0.0, 0.0],
            [0.0, 0.0, 2.0],
        ],
        vec![0, 1, 2, 1, 0, 3],
        AREA_WALKABLE,
    );
    assert_eq!(asset.polygons.len(), 2);
    let owner = NativeOwner::new(&asset, false);
    let filter = detour_query_filter(&asset_query_filter(&asset));
    for (dir, start, end) in directions([1.5, 0.0, 0.5], [0.5, 0.0, 1.5]) {
        let path = owner.path(start, end, DEFAULT_AREA_MASK, &filter);
        path.log("detour", "shared", "lr", dir);
        path.require_completed(start, end, 1e-4);
        assert_eq!(path.corridor, 2);
        assert!(path.markers.is_empty());
    }
}

fn directions(left: [f32; 3], right: [f32; 3]) -> [(&'static str, [f32; 3], [f32; 3]); 2] {
    [("lr", left, right), ("rl", right, left)]
}

fn assert_island_routes(cache: bool, with_link: bool) {
    let left = [0.0, 0.0, 0.3];
    let right = [8.0, 0.0, 0.3];
    let (backend, missing, y_slop) = if cache {
        (
            "cache",
            "TileCache path query found no complete path",
            0.051,
        )
    } else {
        ("detour", "Detour path query found no complete path", 1e-4)
    };
    let jump_mask = DEFAULT_AREA_MASK & !(1_u64 << AREA_JUMP);
    let modes: &[(&str, u64)] = if with_link {
        &[("enabled", DEFAULT_AREA_MASK), ("masked", jump_mask)]
    } else {
        &[("unlinked", DEFAULT_AREA_MASK)]
    };
    let mut topology_matches = true;
    for order in ["lr", "rl"] {
        let mut asset = two_island_asset(with_link);
        if order == "rl" {
            asset.polygons.swap(0, 1);
        }
        let owner = NativeOwner::new(&asset, cache);
        let filter = detour_query_filter(&asset_query_filter(&asset));
        for &(mode, mask) in modes {
            // Same owner and filter, in both islands; a failed snap must not count as no route.
            for (side, start) in [("l", left), ("r", right)] {
                let local = [start[0], start[1], -0.25];
                let path = owner.path(start, local, mask, &filter);
                path.log(backend, mode, order, side);
                path.require_completed(start, local, y_slop);
                assert!(path.markers.is_empty());
            }
            for (dir, start, end) in directions(left, right) {
                let path = owner.path(start, end, mask, &filter);
                path.log(backend, mode, order, dir);
                match path.status {
                    1 => path.require_completed(start, end, y_slop),
                    2 => {
                        assert_eq!(path.message, missing);
                        assert!(path.endpoints.is_none() && path.corridor == 0);
                    }
                    _ => panic!("unexpected native path status: {path:?}"),
                }
                topology_matches &= if mode == "enabled" {
                    path.status == 1 && path.corridor == 3 && path.markers == [(4, 1)]
                } else {
                    path.status == 2
                };
            }
        }
    }
    // Evaluate the full order/direction matrix even if the first unexpected route was complete.
    assert!(
        topology_matches,
        "native island routes must require real adjacent ground or an enabled jump"
    );
}

enum NativeOwner {
    Detour(DetourQuery),
    Cache(NonNull<c_void>),
}

impl NativeOwner {
    fn new(asset: &NavMeshAsset, cache: bool) -> Self {
        if !cache {
            return Self::Detour(DetourQuery::from_asset(asset).expect("ordinary native owner"));
        }
        let vertices = flat_vertices(asset);
        let polygons = detour_polygons(asset);
        let costs = detour_area_costs(asset);
        let links = detour_off_mesh_links(asset);
        let mut result = ffi::ZrNavDetourTileCacheCreateResult::default();
        // SAFETY: The shared marshal vectors and matching counts live through this synchronous
        // creation; null obstacles pair with count zero. This test owns the resulting handle.
        unsafe {
            ffi::zr_nav_tile_cache_create_query(
                vertices.as_ptr(),
                asset.vertices.len() as u32,
                asset.indices.as_ptr(),
                asset.indices.len() as u32,
                polygons.as_ptr(),
                polygons.len() as u32,
                costs.as_ptr(),
                costs.len() as u32,
                links.as_ptr(),
                links.len() as u32,
                std::ptr::null(),
                0,
                &mut result,
            );
        }
        assert_eq!(result.status, 1, "native TileCache creation failed");
        Self::Cache(NonNull::new(result.query).expect("native TileCache owner"))
    }

    fn path(
        &self,
        start: [f32; 3],
        end: [f32; 3],
        mask: u64,
        filter: &ZrNavDetourQueryFilter,
    ) -> NativePath {
        type FindPath = unsafe extern "C" fn(
            *const c_void,
            *const f32,
            *const f32,
            u64,
            *const ZrNavDetourQueryFilter,
            *mut ZrNavDetourPathResult,
        );
        let (query, find): (*const c_void, FindPath) = match self {
            Self::Detour(owner) => (owner.as_raw(), ffi::zr_nav_detour_find_path),
            Self::Cache(owner) => (owner.as_ptr(), ffi::zr_nav_tile_cache_find_path),
        };
        let mut result = ZrNavDetourPathResult::default();
        // SAFETY: The bound live owner and its matching function, arrays, filter, and writable
        // result are valid for this synchronous call. Copy the shared ABI buffers before freeing.
        unsafe {
            find(
                query,
                start.as_ptr(),
                end.as_ptr(),
                mask,
                filter,
                &mut result,
            );
            let points = if result.points.is_null() || result.point_count == 0 {
                &[]
            } else {
                std::slice::from_raw_parts(result.points, result.point_count as usize)
            };
            let copied = NativePath {
                status: result.status,
                message: CStr::from_ptr(result.message.as_ptr())
                    .to_string_lossy()
                    .into_owned(),
                corridor: result.visited_nodes,
                length: result.length,
                endpoints: points
                    .first()
                    .zip(points.last())
                    .map(|(a, b)| (a.position, b.position)),
                markers: points
                    .iter()
                    .filter_map(|p| {
                        let flag = p.flags & 0x04;
                        (flag != 0 || p.off_mesh_user_id != 0).then_some((flag, p.off_mesh_user_id))
                    })
                    .collect(),
            };
            ffi::zr_nav_detour_free_path_result(&mut result);
            copied
        }
    }
}

impl Drop for NativeOwner {
    fn drop(&mut self) {
        if let Self::Cache(owner) = self {
            // SAFETY: Only this owner holds the native cache and it is no longer borrowed.
            unsafe { ffi::zr_nav_tile_cache_free_query(owner.as_ptr()) };
        }
    }
}

#[derive(Debug)]
struct NativePath {
    status: u32,
    message: String,
    corridor: u32,
    length: f32,
    endpoints: Option<([f32; 3], [f32; 3])>,
    markers: Vec<(u8, u32)>,
}

impl NativePath {
    fn require_completed(&self, start: [f32; 3], end: [f32; 3], y_slop: f32) {
        assert_eq!(self.status, 1, "{self:?}");
        assert!(self.corridor > 0 && self.length.is_finite() && self.length > 0.1);
        let (first, last) = self.endpoints.expect("native start and end points");
        for (actual, expected) in [(first, start), (last, end)] {
            assert!((actual[0] - expected[0]).hypot(actual[2] - expected[2]) < 1e-4);
            assert!((actual[1] - expected[1]).abs() < y_slop);
        }
    }

    fn log(&self, backend: &str, mode: &str, order: &str, dir: &str) {
        println!("NATIVE_GROUND_NEIGHBOR backend={backend} mode={mode} order={order} dir={dir} status={} corridor={} length={} markers={:?} message={:?}",
            self.status, self.corridor, self.length, self.markers, self.message);
    }
}
