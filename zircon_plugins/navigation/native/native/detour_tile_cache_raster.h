#pragma once

#include <cstdint>
#include <vector>

#include "recast_bridge.h"

namespace zr_nav_tile_cache_raster {

bool compute_bounds(
    const float* vertices,
    std::uint32_t vertex_count,
    float* bounds_min,
    float* bounds_max,
    const char** error);

float choose_cell_size(float span);

int choose_cell_count(float span, float cell_size);

bool build_layer_data(
    const float* vertices,
    std::uint32_t vertex_count,
    const std::uint32_t* indices,
    std::uint32_t index_count,
    const ZrNavRecastBakePolygon* polygons,
    std::uint32_t polygon_count,
    const float* bounds_min,
    int width,
    int height,
    float cell_size,
    std::vector<unsigned char>* heights,
    std::vector<unsigned char>* areas,
    std::vector<unsigned char>* connections);

} // namespace zr_nav_tile_cache_raster
