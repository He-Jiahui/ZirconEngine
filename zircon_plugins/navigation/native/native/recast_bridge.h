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
