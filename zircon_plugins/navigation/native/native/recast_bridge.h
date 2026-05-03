#pragma once

#include <cstdint>

struct ZrNavRecastBakeSettings {
    float cell_size;
    float cell_height;
    float walkable_slope_degrees;
    float walkable_height;
    float walkable_climb;
    float walkable_radius;
    float min_region_area;
    float merge_region_area;
    float max_edge_length;
    float max_simplification_error;
    std::uint32_t max_vertices_per_polygon;
};

struct ZrNavRecastBakePolygon {
    std::uint32_t first_index;
    std::uint32_t index_count;
    std::uint8_t area;
    std::uint32_t tile;
};

struct ZrNavRecastBakeTile {
    std::uint32_t id;
    float bounds_min[3];
    float bounds_max[3];
    std::uint32_t polygon_count;
};

struct ZrNavRecastBakeResult {
    std::uint32_t status;
    char message[256];
    float* vertices;
    std::uint32_t vertex_count;
    std::uint32_t* indices;
    std::uint32_t index_count;
    ZrNavRecastBakePolygon* polygons;
    std::uint32_t polygon_count;
    ZrNavRecastBakeTile* tiles;
    std::uint32_t tile_count;
};

struct ZrNavDetourAreaCost {
    std::uint8_t area;
    float cost;
    std::uint8_t walkable;
};

struct ZrNavDetourOffMeshLink {
    float start[3];
    float end[3];
    float radius;
    std::uint8_t bidirectional;
    std::uint8_t area;
};

struct ZrNavDetourQuery;

struct ZrNavDetourQueryCreateResult {
    std::uint32_t status;
    char message[256];
    ZrNavDetourQuery* query;
    std::uint32_t polygon_count;
};

struct ZrNavDetourPathPoint {
    float position[3];
    std::uint8_t area;
    std::uint8_t flags;
};

struct ZrNavDetourPathResult {
    std::uint32_t status;
    char message[256];
    ZrNavDetourPathPoint* points;
    std::uint32_t point_count;
    std::uint32_t visited_nodes;
    float length;
};

struct ZrNavDetourSampleResult {
    std::uint32_t status;
    char message[256];
    std::uint8_t hit;
    float position[3];
    float distance;
    std::uint8_t area;
};

struct ZrNavDetourRaycastResult {
    std::uint32_t status;
    char message[256];
    std::uint8_t hit;
    float position[3];
    float normal[3];
    float distance;
    std::uint32_t visited_nodes;
};

struct ZrNavDetourTileCacheObstacle {
    float center[3];
    float half_extents[3];
    float radius;
    float height;
    std::uint8_t shape;
};

struct ZrNavDetourTileCacheQuery;

struct ZrNavDetourTileCacheCreateResult {
    std::uint32_t status;
    char message[256];
    ZrNavDetourTileCacheQuery* query;
    std::uint32_t polygon_count;
    std::uint32_t obstacle_count;
};

extern "C" void zr_nav_recast_bake_triangle_mesh(
    const float* vertices,
    std::uint32_t vertex_count,
    const std::uint32_t* indices,
    std::uint32_t index_count,
    const std::uint8_t* triangle_areas,
    std::uint32_t triangle_area_count,
    const ZrNavRecastBakeSettings* settings,
    ZrNavRecastBakeResult* out_result);

extern "C" void zr_nav_recast_free_bake_result(ZrNavRecastBakeResult* result);

extern "C" void zr_nav_detour_create_query(
    const float* vertices,
    std::uint32_t vertex_count,
    const std::uint32_t* indices,
    std::uint32_t index_count,
    const ZrNavRecastBakePolygon* polygons,
    std::uint32_t polygon_count,
    const ZrNavDetourAreaCost* area_costs,
    std::uint32_t area_cost_count,
    const ZrNavDetourOffMeshLink* off_mesh_links,
    std::uint32_t off_mesh_link_count,
    ZrNavDetourQueryCreateResult* out_result);

extern "C" void zr_nav_detour_free_query(ZrNavDetourQuery* query);

extern "C" void zr_nav_detour_find_path(
    const ZrNavDetourQuery* query,
    const float* start,
    const float* end,
    std::uint64_t area_mask,
    ZrNavDetourPathResult* out_result);

extern "C" void zr_nav_detour_free_path_result(ZrNavDetourPathResult* result);

extern "C" void zr_nav_detour_sample_position(
    const ZrNavDetourQuery* query,
    const float* position,
    const float* extents,
    std::uint64_t area_mask,
    ZrNavDetourSampleResult* out_result);

extern "C" void zr_nav_detour_raycast(
    const ZrNavDetourQuery* query,
    const float* start,
    const float* end,
    std::uint64_t area_mask,
    ZrNavDetourRaycastResult* out_result);

extern "C" void zr_nav_tile_cache_create_query(
    const float* vertices,
    std::uint32_t vertex_count,
    const std::uint32_t* indices,
    std::uint32_t index_count,
    const ZrNavRecastBakePolygon* polygons,
    std::uint32_t polygon_count,
    const ZrNavDetourAreaCost* area_costs,
    std::uint32_t area_cost_count,
    const ZrNavDetourOffMeshLink* off_mesh_links,
    std::uint32_t off_mesh_link_count,
    const ZrNavDetourTileCacheObstacle* obstacles,
    std::uint32_t obstacle_count,
    ZrNavDetourTileCacheCreateResult* out_result);

extern "C" void zr_nav_tile_cache_free_query(ZrNavDetourTileCacheQuery* query);

extern "C" void zr_nav_tile_cache_find_path(
    const ZrNavDetourTileCacheQuery* query,
    const float* start,
    const float* end,
    std::uint64_t area_mask,
    ZrNavDetourPathResult* out_result);
