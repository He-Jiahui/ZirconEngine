#include "recast_bridge.h"

#include <algorithm>
#include <cmath>
#include <cstring>
#include <exception>
#include <iterator>
#include <limits>
#include <memory>
#include <new>
#include <vector>

#include "Recast.h"

namespace {

constexpr std::uint32_t ZR_NAV_RECAST_BAKE_ERROR = 0;
constexpr std::uint32_t ZR_NAV_RECAST_BAKE_OK = 1;
constexpr std::uint8_t ZR_NAV_AREA_WALKABLE = 1;

template <typename T, void (*FreeFn)(T*)>
using RecastPtr = std::unique_ptr<T, decltype(FreeFn)>;

void reset_result(ZrNavRecastBakeResult* result) {
    if (result == nullptr) {
        return;
    }
    result->status = ZR_NAV_RECAST_BAKE_ERROR;
    std::memset(result->message, 0, sizeof(result->message));
    result->vertices = nullptr;
    result->vertex_count = 0;
    result->indices = nullptr;
    result->index_count = 0;
    result->polygons = nullptr;
    result->polygon_count = 0;
    result->tiles = nullptr;
    result->tile_count = 0;
}

void set_message(ZrNavRecastBakeResult* result, const char* message) {
    if (result == nullptr || message == nullptr) {
        return;
    }
    std::strncpy(result->message, message, sizeof(result->message) - 1);
    result->message[sizeof(result->message) - 1] = '\0';
}

void set_error(ZrNavRecastBakeResult* result, const char* message) {
    reset_result(result);
    set_message(result, message);
}

bool finite_positive(float value) {
    return std::isfinite(value) && value > 0.0f;
}

int voxel_ceil(float world_units, float cell_size, int minimum) {
    if (!std::isfinite(world_units) || world_units <= 0.0f) {
        return minimum;
    }
    return std::max(minimum, static_cast<int>(std::ceil(world_units / cell_size)));
}

int voxel_floor(float world_units, float cell_size, int minimum) {
    if (!std::isfinite(world_units) || world_units <= 0.0f) {
        return minimum;
    }
    return std::max(minimum, static_cast<int>(std::floor(world_units / cell_size)));
}

int area_to_cells(float area, float cell_size) {
    if (!std::isfinite(area) || area <= 0.0f || !finite_positive(cell_size)) {
        return 0;
    }
    return std::max(0, static_cast<int>(std::ceil(area / (cell_size * cell_size))));
}

bool make_config(
    const float* vertices,
    int vertex_count,
    const ZrNavRecastBakeSettings& settings,
    rcConfig* config,
    const char** error) {
    if (!finite_positive(settings.cell_size) || !finite_positive(settings.cell_height)) {
        *error = "native Recast bake settings require positive cell size and cell height";
        return false;
    }
    if (!std::isfinite(settings.walkable_slope_degrees)
        || settings.walkable_slope_degrees < 0.0f
        || settings.walkable_slope_degrees >= 90.0f) {
        *error = "native Recast bake settings require slope in [0, 90) degrees";
        return false;
    }

    std::memset(config, 0, sizeof(*config));
    config->cs = settings.cell_size;
    config->ch = settings.cell_height;
    config->walkableSlopeAngle = settings.walkable_slope_degrees;
    config->walkableHeight = voxel_ceil(settings.walkable_height, settings.cell_height, 3);
    config->walkableClimb = voxel_floor(settings.walkable_climb, settings.cell_height, 0);
    config->walkableRadius = voxel_ceil(settings.walkable_radius, settings.cell_size, 0);
    config->maxEdgeLen = voxel_floor(settings.max_edge_length, settings.cell_size, 0);
    config->maxSimplificationError = std::isfinite(settings.max_simplification_error)
        ? std::max(0.0f, settings.max_simplification_error)
        : 1.3f;
    config->minRegionArea = area_to_cells(settings.min_region_area, settings.cell_size);
    config->mergeRegionArea = area_to_cells(settings.merge_region_area, settings.cell_size);
    config->maxVertsPerPoly = static_cast<int>(std::clamp(
        settings.max_vertices_per_polygon,
        static_cast<std::uint32_t>(3),
        static_cast<std::uint32_t>(6)));

    rcCalcBounds(vertices, vertex_count, config->bmin, config->bmax);
    const float vertical_padding = static_cast<float>(config->walkableHeight) * config->ch + config->ch * 2.0f;
    config->bmin[1] -= config->ch * 2.0f;
    config->bmax[1] += vertical_padding;
    rcCalcGridSize(config->bmin, config->bmax, config->cs, &config->width, &config->height);
    if (config->width <= 0 || config->height <= 0) {
        *error = "native Recast bake source bounds produce an empty voxel grid";
        return false;
    }
    return true;
}

bool copy_indices(
    const float* vertices,
    const std::uint32_t* indices,
    std::uint32_t index_count,
    std::uint32_t vertex_count,
    std::vector<int>* out_indices,
    const char** error) {
    if (index_count % 3 != 0) {
        *error = "native Recast bake index count is not divisible by three";
        return false;
    }
    out_indices->reserve(index_count);
    for (std::uint32_t index = 0; index < index_count; index += 3) {
        const std::uint32_t a = indices[index];
        const std::uint32_t b = indices[index + 1];
        const std::uint32_t c = indices[index + 2];
        if (a >= vertex_count || b >= vertex_count || c >= vertex_count
            || a > static_cast<std::uint32_t>(std::numeric_limits<int>::max())
            || b > static_cast<std::uint32_t>(std::numeric_limits<int>::max())
            || c > static_cast<std::uint32_t>(std::numeric_limits<int>::max())) {
            *error = "native Recast bake source mesh references a missing vertex";
            return false;
        }
        const float* va = vertices + a * 3;
        const float* vb = vertices + b * 3;
        const float* vc = vertices + c * 3;
        const float ux = vb[0] - va[0];
        const float uz = vb[2] - va[2];
        const float vx = vc[0] - va[0];
        const float vz = vc[2] - va[2];
        const float normal_y = uz * vx - ux * vz;
        out_indices->push_back(static_cast<int>(a));
        if (normal_y < 0.0f) {
            out_indices->push_back(static_cast<int>(c));
            out_indices->push_back(static_cast<int>(b));
        } else {
            out_indices->push_back(static_cast<int>(b));
            out_indices->push_back(static_cast<int>(c));
        }
    }
    return true;
}

std::vector<unsigned char> copy_areas(
    const std::uint8_t* triangle_areas,
    std::uint32_t triangle_area_count,
    std::uint32_t triangle_count) {
    std::vector<unsigned char> areas(triangle_count, ZR_NAV_AREA_WALKABLE);
    for (std::uint32_t index = 0; index < triangle_count && index < triangle_area_count; ++index) {
        areas[index] = triangle_areas == nullptr ? ZR_NAV_AREA_WALKABLE : triangle_areas[index];
    }
    return areas;
}

template <typename T>
T* copy_to_heap(const std::vector<T>& source) {
    if (source.empty()) {
        return nullptr;
    }
    T* output = new (std::nothrow) T[source.size()];
    if (output == nullptr) {
        return nullptr;
    }
    std::copy(source.begin(), source.end(), output);
    return output;
}

bool copy_poly_mesh_to_result(const rcPolyMesh& mesh, ZrNavRecastBakeResult* result) {
    std::vector<float> vertices;
    vertices.reserve(static_cast<std::size_t>(mesh.nverts) * 3);
    for (int index = 0; index < mesh.nverts; ++index) {
        const unsigned short* vertex = &mesh.verts[index * 3];
        vertices.push_back(mesh.bmin[0] + static_cast<float>(vertex[0]) * mesh.cs);
        vertices.push_back(mesh.bmin[1] + static_cast<float>(vertex[1]) * mesh.ch);
        vertices.push_back(mesh.bmin[2] + static_cast<float>(vertex[2]) * mesh.cs);
    }

    std::vector<std::uint32_t> indices;
    std::vector<ZrNavRecastBakePolygon> polygons;
    for (int polygon_index = 0; polygon_index < mesh.npolys; ++polygon_index) {
        const unsigned short* polygon = &mesh.polys[polygon_index * 2 * mesh.nvp];
        std::vector<std::uint32_t> polygon_vertices;
        for (int vertex_index = 0; vertex_index < mesh.nvp; ++vertex_index) {
            if (polygon[vertex_index] == RC_MESH_NULL_IDX) {
                break;
            }
            polygon_vertices.push_back(polygon[vertex_index]);
        }
        if (polygon_vertices.size() < 3) {
            continue;
        }

        const std::uint32_t first_index = static_cast<std::uint32_t>(indices.size());
        for (std::size_t vertex_index = 1; vertex_index + 1 < polygon_vertices.size(); ++vertex_index) {
            indices.push_back(polygon_vertices[0]);
            indices.push_back(polygon_vertices[vertex_index]);
            indices.push_back(polygon_vertices[vertex_index + 1]);
        }
        polygons.push_back(ZrNavRecastBakePolygon {
            first_index,
            static_cast<std::uint32_t>(indices.size()) - first_index,
            mesh.areas == nullptr ? ZR_NAV_AREA_WALKABLE : mesh.areas[polygon_index],
            0,
        });
    }

    if (vertices.empty() || indices.empty() || polygons.empty()) {
        set_error(result, "native Recast bake produced no walkable polygons");
        return false;
    }

    std::vector<ZrNavRecastBakeTile> tiles;
    ZrNavRecastBakeTile tile = {};
    tile.id = 0;
    std::copy(std::begin(mesh.bmin), std::end(mesh.bmin), std::begin(tile.bounds_min));
    std::copy(std::begin(mesh.bmax), std::end(mesh.bmax), std::begin(tile.bounds_max));
    tile.polygon_count = static_cast<std::uint32_t>(polygons.size());
    tiles.push_back(tile);

    result->vertices = copy_to_heap(vertices);
    result->indices = copy_to_heap(indices);
    result->polygons = copy_to_heap(polygons);
    result->tiles = copy_to_heap(tiles);
    if (result->vertices == nullptr || result->indices == nullptr || result->polygons == nullptr || result->tiles == nullptr) {
        zr_nav_recast_free_bake_result(result);
        set_error(result, "native Recast bake could not allocate output buffers");
        return false;
    }

    result->vertex_count = static_cast<std::uint32_t>(vertices.size() / 3);
    result->index_count = static_cast<std::uint32_t>(indices.size());
    result->polygon_count = static_cast<std::uint32_t>(polygons.size());
    result->tile_count = static_cast<std::uint32_t>(tiles.size());
    result->status = ZR_NAV_RECAST_BAKE_OK;
    set_message(result, "native Recast bake completed");
    return true;
}

} // namespace

extern "C" void zr_nav_recast_bake_triangle_mesh(
    const float* vertices,
    std::uint32_t vertex_count,
    const std::uint32_t* indices,
    std::uint32_t index_count,
    const std::uint8_t* triangle_areas,
    std::uint32_t triangle_area_count,
    const ZrNavRecastBakeSettings* settings,
    ZrNavRecastBakeResult* out_result) {
    try {
        reset_result(out_result);
        if (out_result == nullptr) {
            return;
        }
        if (vertices == nullptr || vertex_count == 0 || indices == nullptr || index_count < 3) {
            set_error(out_result, "native Recast bake source mesh has no triangles");
            return;
        }
        if (settings == nullptr) {
            set_error(out_result, "native Recast bake settings are missing");
            return;
        }

        const char* error = nullptr;
        std::vector<int> recast_indices;
        if (!copy_indices(vertices, indices, index_count, vertex_count, &recast_indices, &error)) {
            set_error(out_result, error);
            return;
        }
        const std::uint32_t triangle_count = index_count / 3;
        std::vector<unsigned char> areas = copy_areas(triangle_areas, triangle_area_count, triangle_count);

        rcConfig config = {};
        if (!make_config(vertices, static_cast<int>(vertex_count), *settings, &config, &error)) {
            set_error(out_result, error);
            return;
        }

        rcContext context;
        RecastPtr<rcHeightfield, rcFreeHeightField> solid(rcAllocHeightfield(), rcFreeHeightField);
        if (!solid || !rcCreateHeightfield(&context, *solid, config.width, config.height, config.bmin, config.bmax, config.cs, config.ch)) {
            set_error(out_result, "native Recast bake could not create a heightfield");
            return;
        }

        rcClearUnwalkableTriangles(
            &context,
            config.walkableSlopeAngle,
            vertices,
            static_cast<int>(vertex_count),
            recast_indices.data(),
            static_cast<int>(triangle_count),
            areas.data());
        if (!rcRasterizeTriangles(
                &context,
                vertices,
                static_cast<int>(vertex_count),
                recast_indices.data(),
                areas.data(),
                static_cast<int>(triangle_count),
                *solid,
                config.walkableClimb)) {
            set_error(out_result, "native Recast bake could not rasterize triangles");
            return;
        }

        rcFilterLowHangingWalkableObstacles(&context, config.walkableClimb, *solid);
        rcFilterLedgeSpans(&context, config.walkableHeight, config.walkableClimb, *solid);
        rcFilterWalkableLowHeightSpans(&context, config.walkableHeight, *solid);

        RecastPtr<rcCompactHeightfield, rcFreeCompactHeightfield> compact(rcAllocCompactHeightfield(), rcFreeCompactHeightfield);
        if (!compact || !rcBuildCompactHeightfield(&context, config.walkableHeight, config.walkableClimb, *solid, *compact)) {
            set_error(out_result, "native Recast bake could not build a compact heightfield");
            return;
        }
        if (config.walkableRadius > 0 && !rcErodeWalkableArea(&context, config.walkableRadius, *compact)) {
            set_error(out_result, "native Recast bake could not erode walkable area");
            return;
        }
        if (!rcBuildDistanceField(&context, *compact)) {
            set_error(out_result, "native Recast bake could not build a distance field");
            return;
        }
        if (!rcBuildRegions(&context, *compact, 0, config.minRegionArea, config.mergeRegionArea)) {
            set_error(out_result, "native Recast bake could not build regions");
            return;
        }

        RecastPtr<rcContourSet, rcFreeContourSet> contours(rcAllocContourSet(), rcFreeContourSet);
        if (!contours || !rcBuildContours(&context, *compact, config.maxSimplificationError, config.maxEdgeLen, *contours)) {
            set_error(out_result, "native Recast bake could not build contours");
            return;
        }

        RecastPtr<rcPolyMesh, rcFreePolyMesh> poly_mesh(rcAllocPolyMesh(), rcFreePolyMesh);
        if (!poly_mesh || !rcBuildPolyMesh(&context, *contours, config.maxVertsPerPoly, *poly_mesh)) {
            set_error(out_result, "native Recast bake could not build a polygon mesh");
            return;
        }
        copy_poly_mesh_to_result(*poly_mesh, out_result);
    } catch (const std::bad_alloc&) {
        set_error(out_result, "native Recast bake allocation failed");
    } catch (const std::exception& error) {
        set_error(out_result, error.what());
    } catch (...) {
        set_error(out_result, "native Recast bake failed with an unknown native exception");
    }
}

extern "C" void zr_nav_recast_free_bake_result(ZrNavRecastBakeResult* result) {
    if (result == nullptr) {
        return;
    }
    delete[] result->vertices;
    delete[] result->indices;
    delete[] result->polygons;
    delete[] result->tiles;
    reset_result(result);
}
