use std::os::raw::{c_char, c_float, c_uchar, c_uint, c_ulonglong, c_ushort, c_void};

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
#[derive(Clone, Copy, Debug)]
pub(crate) struct ZrNavRecastBakeTileRequest {
    pub id: c_uint,
    pub bounds_min: [c_float; 3],
    pub bounds_max: [c_float; 3],
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

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct ZrNavDetourAreaCost {
    pub area: c_uchar,
    pub cost: c_float,
    pub walkable: c_uchar,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct ZrNavDetourQueryFilter {
    pub area_costs: [c_float; 64],
    pub include_flags: c_ushort,
    pub exclude_flags: c_ushort,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct ZrNavDetourOffMeshLink {
    pub start: [c_float; 3],
    pub end: [c_float; 3],
    pub radius: c_float,
    pub bidirectional: c_uchar,
    pub area: c_uchar,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct ZrNavDetourQueryCreateResult {
    pub status: c_uint,
    pub message: [c_char; 256],
    pub query: *mut c_void,
    pub polygon_count: c_uint,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct ZrNavCrowdCreateResult {
    pub status: c_uint,
    pub message: [c_char; 256],
    pub crowd: *mut c_void,
    pub capacity: c_uint,
}

impl Default for ZrNavCrowdCreateResult {
    fn default() -> Self {
        Self {
            status: 0,
            message: [0; 256],
            crowd: std::ptr::null_mut(),
            capacity: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct ZrNavCrowdAgentParams {
    pub radius: c_float,
    pub height: c_float,
    pub max_acceleration: c_float,
    pub max_speed: c_float,
    pub collision_query_range: c_float,
    pub path_optimization_range: c_float,
    pub separation_weight: c_float,
    pub avoidance_quality: c_uchar,
    pub avoidance_priority: c_uchar,
    pub area_mask: c_ulonglong,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct ZrNavCrowdCommandResult {
    pub status: c_uint,
    pub message: [c_char; 256],
    pub agent_id: c_uint,
    pub state_count: c_uint,
}

impl Default for ZrNavCrowdCommandResult {
    fn default() -> Self {
        Self {
            status: 0,
            message: [0; 256],
            agent_id: 0,
            state_count: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ZrNavCrowdAgentState {
    pub agent_id: c_uint,
    pub active: c_uchar,
    pub traversal_state: c_uchar,
    pub target_state: c_uchar,
    pub partial_path: c_uchar,
    pub position: [c_float; 3],
    pub desired_velocity: [c_float; 3],
    pub avoidance_velocity: [c_float; 3],
    pub velocity: [c_float; 3],
}

impl Default for ZrNavDetourQueryCreateResult {
    fn default() -> Self {
        Self {
            status: 0,
            message: [0; 256],
            query: std::ptr::null_mut(),
            polygon_count: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct ZrNavDetourPathPoint {
    pub position: [c_float; 3],
    pub area: c_uchar,
    pub flags: c_uchar,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct ZrNavDetourPathResult {
    pub status: c_uint,
    pub message: [c_char; 256],
    pub points: *mut ZrNavDetourPathPoint,
    pub point_count: c_uint,
    pub visited_nodes: c_uint,
    pub length: c_float,
}

impl Default for ZrNavDetourPathResult {
    fn default() -> Self {
        Self {
            status: 0,
            message: [0; 256],
            points: std::ptr::null_mut(),
            point_count: 0,
            visited_nodes: 0,
            length: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct ZrNavDetourSampleResult {
    pub status: c_uint,
    pub message: [c_char; 256],
    pub hit: c_uchar,
    pub position: [c_float; 3],
    pub distance: c_float,
    pub area: c_uchar,
}

impl Default for ZrNavDetourSampleResult {
    fn default() -> Self {
        Self {
            status: 0,
            message: [0; 256],
            hit: 0,
            position: [0.0; 3],
            distance: 0.0,
            area: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct ZrNavDetourRaycastResult {
    pub status: c_uint,
    pub message: [c_char; 256],
    pub hit: c_uchar,
    pub position: [c_float; 3],
    pub normal: [c_float; 3],
    pub distance: c_float,
    pub visited_nodes: c_uint,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct ZrNavDetourTileCacheObstacle {
    pub center: [c_float; 3],
    pub half_extents: [c_float; 3],
    pub radius: c_float,
    pub height: c_float,
    pub shape: c_uchar,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct ZrNavDetourTileCacheCreateResult {
    pub status: c_uint,
    pub message: [c_char; 256],
    pub query: *mut c_void,
    pub polygon_count: c_uint,
    pub obstacle_count: c_uint,
}

impl Default for ZrNavDetourTileCacheCreateResult {
    fn default() -> Self {
        Self {
            status: 0,
            message: [0; 256],
            query: std::ptr::null_mut(),
            polygon_count: 0,
            obstacle_count: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct ZrNavDetourTileCacheCommandResult {
    pub status: c_uint,
    pub message: [c_char; 256],
    pub obstacle_ref: c_ulonglong,
}

impl Default for ZrNavDetourTileCacheCommandResult {
    fn default() -> Self {
        Self {
            status: 0,
            message: [0; 256],
            obstacle_ref: 0,
        }
    }
}

impl Default for ZrNavDetourRaycastResult {
    fn default() -> Self {
        Self {
            status: 0,
            message: [0; 256],
            hit: 0,
            position: [0.0; 3],
            normal: [0.0, 1.0, 0.0],
            distance: 0.0,
            visited_nodes: 0,
        }
    }
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
    pub(crate) fn zr_nav_recast_bake_tile(
        vertices: *const c_float,
        vertex_count: c_uint,
        indices: *const c_uint,
        index_count: c_uint,
        triangle_areas: *const c_uchar,
        triangle_area_count: c_uint,
        settings: *const ZrNavRecastBakeSettings,
        tile: *const ZrNavRecastBakeTileRequest,
        out_result: *mut ZrNavRecastBakeResult,
    );
    pub(crate) fn zr_nav_recast_free_bake_result(result: *mut ZrNavRecastBakeResult);
    pub(crate) fn zr_nav_detour_create_query(
        vertices: *const c_float,
        vertex_count: c_uint,
        indices: *const c_uint,
        index_count: c_uint,
        polygons: *const ZrNavRecastBakePolygon,
        polygon_count: c_uint,
        area_costs: *const ZrNavDetourAreaCost,
        area_cost_count: c_uint,
        off_mesh_links: *const ZrNavDetourOffMeshLink,
        off_mesh_link_count: c_uint,
        out_result: *mut ZrNavDetourQueryCreateResult,
    );
    pub(crate) fn zr_nav_detour_free_query(query: *mut c_void);
    pub(crate) fn zr_nav_crowd_create(
        query_owner: *mut c_void,
        max_agents: c_uint,
        max_agent_radius: c_float,
        area_costs: *const ZrNavDetourAreaCost,
        area_cost_count: c_uint,
        out_result: *mut ZrNavCrowdCreateResult,
    );
    pub(crate) fn zr_nav_crowd_free(crowd: *mut c_void);
    pub(crate) fn zr_nav_crowd_add_agent(
        crowd: *mut c_void,
        position: *const c_float,
        params: *const ZrNavCrowdAgentParams,
        out_result: *mut ZrNavCrowdCommandResult,
    );
    pub(crate) fn zr_nav_crowd_remove_agent(
        crowd: *mut c_void,
        agent_id: c_uint,
        out_result: *mut ZrNavCrowdCommandResult,
    );
    pub(crate) fn zr_nav_crowd_set_target(
        crowd: *mut c_void,
        agent_id: c_uint,
        target: *const c_float,
        out_result: *mut ZrNavCrowdCommandResult,
    );
    pub(crate) fn zr_nav_crowd_sync_agent_position(
        crowd: *mut c_void,
        agent_id: c_uint,
        position: *const c_float,
        out_result: *mut ZrNavCrowdCommandResult,
    );
    pub(crate) fn zr_nav_crowd_update(
        crowd: *mut c_void,
        dt_seconds: c_float,
        out_result: *mut ZrNavCrowdCommandResult,
    );
    pub(crate) fn zr_nav_crowd_read_states(
        crowd: *const c_void,
        states: *mut ZrNavCrowdAgentState,
        state_capacity: c_uint,
        out_result: *mut ZrNavCrowdCommandResult,
    );
    pub(crate) fn zr_nav_detour_find_path(
        query: *const c_void,
        start: *const c_float,
        end: *const c_float,
        area_mask: c_ulonglong,
        filter: *const ZrNavDetourQueryFilter,
        out_result: *mut ZrNavDetourPathResult,
    );
    pub(crate) fn zr_nav_detour_free_path_result(result: *mut ZrNavDetourPathResult);
    pub(crate) fn zr_nav_detour_sample_position(
        query: *const c_void,
        position: *const c_float,
        extents: *const c_float,
        area_mask: c_ulonglong,
        out_result: *mut ZrNavDetourSampleResult,
    );
    pub(crate) fn zr_nav_detour_raycast(
        query: *const c_void,
        start: *const c_float,
        end: *const c_float,
        area_mask: c_ulonglong,
        out_result: *mut ZrNavDetourRaycastResult,
    );
    pub(crate) fn zr_nav_tile_cache_create_query(
        vertices: *const c_float,
        vertex_count: c_uint,
        indices: *const c_uint,
        index_count: c_uint,
        polygons: *const ZrNavRecastBakePolygon,
        polygon_count: c_uint,
        area_costs: *const ZrNavDetourAreaCost,
        area_cost_count: c_uint,
        off_mesh_links: *const ZrNavDetourOffMeshLink,
        off_mesh_link_count: c_uint,
        obstacles: *const ZrNavDetourTileCacheObstacle,
        obstacle_count: c_uint,
        out_result: *mut ZrNavDetourTileCacheCreateResult,
    );
    pub(crate) fn zr_nav_tile_cache_free_query(query: *mut c_void);
    pub(crate) fn zr_nav_tile_cache_add_obstacle(
        query: *mut c_void,
        obstacle: *const ZrNavDetourTileCacheObstacle,
        out_result: *mut ZrNavDetourTileCacheCommandResult,
    );
    pub(crate) fn zr_nav_tile_cache_remove_obstacle(
        query: *mut c_void,
        obstacle_ref: c_ulonglong,
        out_result: *mut ZrNavDetourTileCacheCommandResult,
    );
    pub(crate) fn zr_nav_tile_cache_update(
        query: *mut c_void,
        out_result: *mut ZrNavDetourTileCacheCommandResult,
    );
    pub(crate) fn zr_nav_tile_cache_find_path(
        query: *const c_void,
        start: *const c_float,
        end: *const c_float,
        area_mask: c_ulonglong,
        filter: *const ZrNavDetourQueryFilter,
        out_result: *mut ZrNavDetourPathResult,
    );
}
