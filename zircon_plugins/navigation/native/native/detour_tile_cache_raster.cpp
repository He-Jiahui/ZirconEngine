#include "detour_tile_cache_raster.h"

#include <algorithm>
#include <cmath>
#include <limits>

#include "DetourTileCacheBuilder.h"

namespace {

constexpr std::uint8_t ZR_NAV_AREA_WALKABLE = 1;
constexpr int ZR_NAV_TILE_CACHE_MAX_CELLS = 160;
constexpr float ZR_NAV_TILE_CACHE_MIN_CELL = 0.05f;
constexpr unsigned char ZR_NAV_TILE_CACHE_WEST = 1 << 0;
constexpr unsigned char ZR_NAV_TILE_CACHE_NORTH = 1 << 1;
constexpr unsigned char ZR_NAV_TILE_CACHE_EAST = 1 << 2;
constexpr unsigned char ZR_NAV_TILE_CACHE_SOUTH = 1 << 3;

bool finite3(const float* value) {
    return value != nullptr
        && std::isfinite(value[0])
        && std::isfinite(value[1])
        && std::isfinite(value[2]);
}

void copy3(const float* source, float* target) {
    target[0] = source[0];
    target[1] = source[1];
    target[2] = source[2];
}

bool barycentric_xz(const float* a, const float* b, const float* c, const float x, const float z) {
    const float denominator = (b[2] - c[2]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[2] - c[2]);
    if (std::abs(denominator) <= std::numeric_limits<float>::epsilon()) {
        return false;
    }
    const float u = ((b[2] - c[2]) * (x - c[0]) + (c[0] - b[0]) * (z - c[2])) / denominator;
    const float v = ((c[2] - a[2]) * (x - c[0]) + (a[0] - c[0]) * (z - c[2])) / denominator;
    const float w = 1.0f - u - v;
    return u >= -0.0001f && v >= -0.0001f && w >= -0.0001f;
}

unsigned char area_at_cell(
    const float* vertices,
    std::uint32_t vertex_count,
    const std::uint32_t* indices,
    std::uint32_t index_count,
    const ZrNavRecastBakePolygon* polygons,
    std::uint32_t polygon_count,
    float x,
    float z) {
    for (std::uint32_t polygon_index = 0; polygon_index < polygon_count; ++polygon_index) {
        const ZrNavRecastBakePolygon& polygon = polygons[polygon_index];
        const std::uint32_t start = polygon.first_index;
        const std::uint32_t end = start + polygon.index_count;
        if (start > index_count || end > index_count || end < start) {
            continue;
        }
        for (std::uint32_t offset = start; offset + 2 < end; offset += 3) {
            const std::uint32_t ia = indices[offset];
            const std::uint32_t ib = indices[offset + 1];
            const std::uint32_t ic = indices[offset + 2];
            if (ia >= vertex_count || ib >= vertex_count || ic >= vertex_count) {
                continue;
            }
            if (barycentric_xz(vertices + ia * 3, vertices + ib * 3, vertices + ic * 3, x, z)) {
                return polygon.area == DT_TILECACHE_NULL_AREA ? ZR_NAV_AREA_WALKABLE : polygon.area;
            }
        }
    }
    return DT_TILECACHE_NULL_AREA;
}

void build_connectivity(
    const std::vector<unsigned char>& areas,
    std::vector<unsigned char>* connections,
    int width,
    int height) {
    connections->assign(static_cast<std::size_t>(width * height), 0);
    for (int y = 0; y < height; ++y) {
        for (int x = 0; x < width; ++x) {
            const int index = x + y * width;
            if (areas[index] == DT_TILECACHE_NULL_AREA) {
                continue;
            }
            unsigned char connection = 0;
            if (x > 0 && areas[index - 1] != DT_TILECACHE_NULL_AREA) {
                connection |= ZR_NAV_TILE_CACHE_WEST;
            }
            if (y + 1 < height && areas[index + width] != DT_TILECACHE_NULL_AREA) {
                connection |= ZR_NAV_TILE_CACHE_NORTH;
            }
            if (x + 1 < width && areas[index + 1] != DT_TILECACHE_NULL_AREA) {
                connection |= ZR_NAV_TILE_CACHE_EAST;
            }
            if (y > 0 && areas[index - width] != DT_TILECACHE_NULL_AREA) {
                connection |= ZR_NAV_TILE_CACHE_SOUTH;
            }
            (*connections)[index] = connection;
        }
    }
}

} // namespace

namespace zr_nav_tile_cache_raster {

bool compute_bounds(
    const float* vertices,
    std::uint32_t vertex_count,
    float* bounds_min,
    float* bounds_max,
    const char** error) {
    if (vertices == nullptr || vertex_count == 0) {
        *error = "TileCache input has no vertices";
        return false;
    }
    if (!finite3(vertices)) {
        *error = "TileCache input contains non-finite vertices";
        return false;
    }
    copy3(vertices, bounds_min);
    copy3(vertices, bounds_max);
    for (std::uint32_t index = 0; index < vertex_count; ++index) {
        const float* vertex = vertices + index * 3;
        if (!finite3(vertex)) {
            *error = "TileCache input contains non-finite vertices";
            return false;
        }
        for (int axis = 0; axis < 3; ++axis) {
            bounds_min[axis] = std::min(bounds_min[axis], vertex[axis]);
            bounds_max[axis] = std::max(bounds_max[axis], vertex[axis]);
        }
    }
    return true;
}

float choose_cell_size(float span) {
    if (!std::isfinite(span) || span <= 0.0f) {
        return ZR_NAV_TILE_CACHE_MIN_CELL;
    }
    return std::max(ZR_NAV_TILE_CACHE_MIN_CELL, span / static_cast<float>(ZR_NAV_TILE_CACHE_MAX_CELLS));
}

int choose_cell_count(float span, float cell_size) {
    const int cells = static_cast<int>(std::ceil(std::max(span, cell_size) / cell_size)) + 2;
    return std::clamp(cells, 1, 255);
}

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
    std::vector<unsigned char>* connections) {
    const std::size_t cell_count = static_cast<std::size_t>(width * height);
    heights->assign(cell_count, 0);
    areas->assign(cell_count, DT_TILECACHE_NULL_AREA);
    for (int y = 0; y < height; ++y) {
        for (int x = 0; x < width; ++x) {
            const float sample_x = bounds_min[0] + (static_cast<float>(x) + 0.5f) * cell_size;
            const float sample_z = bounds_min[2] + (static_cast<float>(y) + 0.5f) * cell_size;
            (*areas)[static_cast<std::size_t>(x + y * width)] = area_at_cell(
                vertices,
                vertex_count,
                indices,
                index_count,
                polygons,
                polygon_count,
                sample_x,
                sample_z);
        }
    }
    build_connectivity(*areas, connections, width, height);
    return std::any_of(areas->begin(), areas->end(), [](unsigned char area) {
        return area != DT_TILECACHE_NULL_AREA;
    });
}

} // namespace zr_nav_tile_cache_raster
