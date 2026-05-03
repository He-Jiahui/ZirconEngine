use std::os::raw::{c_char, c_float, c_uchar, c_uint, c_ulonglong};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct ZrNavRecastBakeSettings {
    pub cell_size: c_float,
    pub cell_height: c_float,
    pub walkable_slope_degrees: c_float,
    pub walkable_height: c_float,
    pub walkable_climb: c_float,
    pub walkable_radius: c_float,
    pub min_region_area: c_float,
    pub merge_region_area: c_float,
    pub max_edge_length: c_float,
    pub max_simplification_error: c_float,
    pub max_vertices_per_polygon: c_uint,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct ZrNavRecastBakePolygon {
    pub first_index: c_uint,
    pub index_count: c_uint,
    pub area: c_uchar,
    pub tile: c_uint,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct ZrNavRecastBakeTile {
    pub id: c_uint,
    pub bounds_min: [c_float; 3],
    pub bounds_max: [c_float; 3],
    pub polygon_count: c_uint,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct ZrNavRecastBakeResult {
    pub status: c_uint,
    pub message: [c_char; 256],
    pub vertices: *mut c_float,
    pub vertex_count: c_uint,
    pub indices: *mut c_uint,
    pub index_count: c_uint,
    pub polygons: *mut ZrNavRecastBakePolygon,
    pub polygon_count: c_uint,
    pub tiles: *mut ZrNavRecastBakeTile,
    pub tile_count: c_uint,
}

impl Default for ZrNavRecastBakeResult {
    fn default() -> Self {
        Self {
            status: 0,
            message: [0; 256],
            vertices: std::ptr::null_mut(),
            vertex_count: 0,
            indices: std::ptr::null_mut(),
            index_count: 0,
            polygons: std::ptr::null_mut(),
            polygon_count: 0,
            tiles: std::ptr::null_mut(),
            tile_count: 0,
        }
    }
}

extern "C" {
    pub(crate) fn zr_nav_recast_bridge_version() -> c_uint;
    pub(crate) fn zr_nav_recast_runtime_modules_smoke() -> c_uint;
    pub(crate) fn zr_nav_recast_polyline_length(
        xyz: *const c_float,
        point_count: c_ulonglong,
    ) -> c_float;
    pub(crate) fn zr_nav_recast_bake_triangle_mesh(
        vertices: *const c_float,
        vertex_count: c_uint,
        indices: *const c_uint,
        index_count: c_uint,
        triangle_areas: *const c_uchar,
        triangle_area_count: c_uint,
        settings: *const ZrNavRecastBakeSettings,
        out_result: *mut ZrNavRecastBakeResult,
    );
    pub(crate) fn zr_nav_recast_free_bake_result(result: *mut ZrNavRecastBakeResult);
}
