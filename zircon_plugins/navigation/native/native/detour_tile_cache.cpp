#include "recast_bridge.h"
#include "detour_off_mesh_connections.h"
#include "detour_tile_cache_raster.h"

#include <algorithm>
#include <cmath>
#include <cstring>
#include <exception>
#include <new>
#include <vector>

#include "DetourAlloc.h"
#include "DetourCommon.h"
#include "DetourNavMesh.h"
#include "DetourNavMeshBuilder.h"
#include "DetourNavMeshQuery.h"
#include "DetourStatus.h"
#include "DetourTileCache.h"
#include "DetourTileCacheBuilder.h"

namespace {

constexpr std::uint32_t ZR_NAV_DETOUR_ERROR = 0;
constexpr std::uint32_t ZR_NAV_DETOUR_OK = 1;
constexpr std::uint32_t ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH = 2;
constexpr std::uint8_t ZR_NAV_AREA_WALKABLE = 1;
constexpr std::uint8_t ZR_NAV_TILE_CACHE_SHAPE_CYLINDER = 0;
constexpr std::uint8_t ZR_NAV_TILE_CACHE_SHAPE_BOX = 1;
constexpr int ZR_NAV_TILE_CACHE_MAX_PATH = 512;
constexpr int ZR_NAV_TILE_CACHE_MAX_STRAIGHT_PATH = 512;
constexpr int ZR_NAV_TILE_CACHE_MAX_OBSTACLES = 128;
constexpr std::uint32_t ZR_NAV_TILE_CACHE_MAX_PENDING_REQUESTS = 64;
constexpr float ZR_NAV_TILE_CACHE_CELL_HEIGHT = 0.05f;

using zr_nav_tile_cache_raster::build_layer_data;
using zr_nav_tile_cache_raster::choose_cell_count;
using zr_nav_tile_cache_raster::choose_cell_size;
using zr_nav_tile_cache_raster::compute_bounds;

unsigned short area_flag(const unsigned char area) {
    if (area == 0) {
        return 0;
    }
    return area <= 15 ? static_cast<unsigned short>(1u << (area - 1)) : static_cast<unsigned short>(1u << 15);
}

class ZrNavTileCacheCompressor final : public dtTileCacheCompressor {
public:
    int maxCompressedSize(const int buffer_size) override {
        return buffer_size;
    }

    dtStatus compress(
        const unsigned char* buffer,
        const int buffer_size,
        unsigned char* compressed,
        const int max_compressed_size,
        int* compressed_size) override {
        if (buffer == nullptr || compressed == nullptr || compressed_size == nullptr || buffer_size > max_compressed_size) {
            return DT_FAILURE | DT_INVALID_PARAM;
        }
        std::memcpy(compressed, buffer, static_cast<std::size_t>(buffer_size));
        *compressed_size = buffer_size;
        return DT_SUCCESS;
    }

    dtStatus decompress(
        const unsigned char* compressed,
        const int compressed_size,
        unsigned char* buffer,
        const int max_buffer_size,
        int* buffer_size) override {
        if (compressed == nullptr || buffer == nullptr || buffer_size == nullptr || compressed_size > max_buffer_size) {
            return DT_FAILURE | DT_INVALID_PARAM;
        }
        std::memcpy(buffer, compressed, static_cast<std::size_t>(compressed_size));
        *buffer_size = compressed_size;
        return DT_SUCCESS;
    }
};

class ZrNavTileCacheMeshProcess final : public dtTileCacheMeshProcess {
public:
    bool assign_off_mesh_connections(
        const ZrNavDetourOffMeshLink* links,
        std::uint32_t link_count,
        const char** error) {
        return off_mesh_connections_.assign(links, link_count, error);
    }

    void process(dtNavMeshCreateParams* params, unsigned char* poly_areas, unsigned short* poly_flags) override {
        if (params == nullptr || poly_areas == nullptr || poly_flags == nullptr) {
            return;
        }
        for (int index = 0; index < params->polyCount; ++index) {
            poly_flags[index] = poly_areas[index] == DT_TILECACHE_NULL_AREA ? 0 : area_flag(poly_areas[index]);
        }
        off_mesh_connections_.bind_for_tile(params);
    }

private:
    zr_nav_off_mesh::ConnectionSet off_mesh_connections_;
};

void set_message(char* output, std::size_t output_size, const char* message) {
    if (output == nullptr || output_size == 0) {
        return;
    }
    std::memset(output, 0, output_size);
    if (message == nullptr) {
        return;
    }
    std::strncpy(output, message, output_size - 1);
    output[output_size - 1] = '\0';
}

void reset_create_result(ZrNavDetourTileCacheCreateResult* result) {
    if (result == nullptr) {
        return;
    }
    result->status = ZR_NAV_DETOUR_ERROR;
    set_message(result->message, sizeof(result->message), "");
    result->query = nullptr;
    result->polygon_count = 0;
    result->obstacle_count = 0;
}

void set_create_status(ZrNavDetourTileCacheCreateResult* result, std::uint32_t status, const char* message) {
    if (result == nullptr) {
        return;
    }
    result->status = status;
    set_message(result->message, sizeof(result->message), message);
}

void reset_command_result(ZrNavDetourTileCacheCommandResult* result) {
    if (result == nullptr) {
        return;
    }
    result->status = ZR_NAV_DETOUR_ERROR;
    set_message(result->message, sizeof(result->message), "");
    result->obstacle_ref = 0;
}

void set_command_status(
    ZrNavDetourTileCacheCommandResult* result,
    std::uint32_t status,
    const char* message,
    dtObstacleRef obstacle_ref = 0) {
    if (result == nullptr) {
        return;
    }
    result->status = status;
    set_message(result->message, sizeof(result->message), message);
    result->obstacle_ref = static_cast<std::uint64_t>(obstacle_ref);
}

void reset_path_result(ZrNavDetourPathResult* result) {
    if (result == nullptr) {
        return;
    }
    result->status = ZR_NAV_DETOUR_ERROR;
    set_message(result->message, sizeof(result->message), "");
    result->points = nullptr;
    result->point_count = 0;
    result->visited_nodes = 0;
    result->length = 0.0f;
}

void set_path_status(ZrNavDetourPathResult* result, std::uint32_t status, const char* message) {
    if (result == nullptr) {
        return;
    }
    result->status = status;
    set_message(result->message, sizeof(result->message), message);
}

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

float distance3(const float* left, const float* right) {
    const float dx = right[0] - left[0];
    const float dy = right[1] - left[1];
    const float dz = right[2] - left[2];
    return std::sqrt(dx * dx + dy * dy + dz * dz);
}

void initialize_area_tables(
    float* area_costs,
    bool* area_walkable,
    const ZrNavDetourAreaCost* costs,
    std::uint32_t cost_count) {
    for (int index = 0; index < DT_MAX_AREAS; ++index) {
        area_costs[index] = 1.0f;
        area_walkable[index] = index != 0;
    }
    for (std::uint32_t index = 0; index < cost_count; ++index) {
        const ZrNavDetourAreaCost& cost = costs[index];
        if (cost.area >= DT_MAX_AREAS) {
            continue;
        }
        area_costs[cost.area] = std::isfinite(cost.cost) ? std::max(cost.cost, 0.001f) : 1.0f;
        area_walkable[cost.area] = cost.walkable != 0;
    }
}

} // namespace

struct ZrNavDetourTileCacheQuery {
    dtTileCache* tile_cache = nullptr;
    dtNavMesh* nav_mesh = nullptr;
    dtNavMeshQuery* query = nullptr;
    dtTileCacheAlloc alloc;
    ZrNavTileCacheCompressor compressor;
    ZrNavTileCacheMeshProcess mesh_process;
    float query_extents[3] = { 1.0f, 4.0f, 1.0f };
    float area_costs[DT_MAX_AREAS] = {};
    bool area_walkable[DT_MAX_AREAS] = {};
    std::uint32_t polygon_count = 0;
    std::uint32_t obstacle_count = 0;
};

namespace {

class ZrNavTileCacheFilter final : public dtQueryFilter {
public:
    ZrNavTileCacheFilter(
        const ZrNavDetourTileCacheQuery& owner,
        std::uint64_t area_mask,
        const ZrNavDetourQueryFilter* filter = nullptr)
        : m_area_mask(area_mask),
          m_include_flags(filter == nullptr ? 0xffffu : filter->include_flags),
          m_exclude_flags(filter == nullptr ? 0u : filter->exclude_flags) {
        for (int index = 0; index < DT_MAX_AREAS; ++index) {
            const float requested = filter == nullptr ? owner.area_costs[index] : filter->area_costs[index];
            m_area_costs[index] = std::isfinite(requested) && requested > 0.0f ? requested : 1.0f;
            m_area_walkable[index] = owner.area_walkable[index];
        }
    }

    bool passFilter(const dtPolyRef, const dtMeshTile*, const dtPoly* poly) const override {
        if (poly == nullptr) {
            return false;
        }
        const int area = poly->getArea();
        return area >= 0
            && area < DT_MAX_AREAS
            && m_area_walkable[area]
            && (poly->flags & m_include_flags) != 0
            && (poly->flags & m_exclude_flags) == 0
            && ((m_area_mask & (std::uint64_t(1) << area)) != 0);
    }

    float getCost(
        const float* pa,
        const float* pb,
        const dtPolyRef,
        const dtMeshTile*,
        const dtPoly*,
        const dtPolyRef,
        const dtMeshTile*,
        const dtPoly* cur_poly,
        const dtPolyRef,
        const dtMeshTile*,
        const dtPoly*) const override {
        const int area = cur_poly == nullptr ? ZR_NAV_AREA_WALKABLE : cur_poly->getArea();
        const float cost = area >= 0 && area < DT_MAX_AREAS ? m_area_costs[area] : 1.0f;
        return dtVdist(pa, pb) * std::max(cost, 0.001f);
    }

private:
    std::uint64_t m_area_mask;
    unsigned short m_include_flags;
    unsigned short m_exclude_flags;
    float m_area_costs[DT_MAX_AREAS] = {};
    bool m_area_walkable[DT_MAX_AREAS] = {};
};

dtStatus add_obstacle(
    dtTileCache* tile_cache,
    const ZrNavDetourTileCacheObstacle& obstacle,
    dtObstacleRef* obstacle_ref) {
    if (tile_cache == nullptr || obstacle_ref == nullptr || !finite3(obstacle.center)) {
        return DT_FAILURE | DT_INVALID_PARAM;
    }
    if (obstacle.shape == ZR_NAV_TILE_CACHE_SHAPE_BOX) {
        if (!finite3(obstacle.half_extents)) {
            return DT_FAILURE | DT_INVALID_PARAM;
        }
        float bmin[3] = {
            obstacle.center[0] - std::abs(obstacle.half_extents[0]),
            obstacle.center[1] - std::abs(obstacle.half_extents[1]),
            obstacle.center[2] - std::abs(obstacle.half_extents[2]),
        };
        float bmax[3] = {
            obstacle.center[0] + std::abs(obstacle.half_extents[0]),
            obstacle.center[1] + std::abs(obstacle.half_extents[1]),
            obstacle.center[2] + std::abs(obstacle.half_extents[2]),
        };
        return tile_cache->addBoxObstacle(bmin, bmax, obstacle_ref);
    }
    const float radius = std::isfinite(obstacle.radius) ? std::max(obstacle.radius, 0.05f) : 0.05f;
    const float height = std::isfinite(obstacle.height) ? std::max(obstacle.height, 0.05f) : 0.05f;
    return tile_cache->addObstacle(obstacle.center, radius, height, obstacle_ref);
}

dtStatus update_until_current(ZrNavDetourTileCacheQuery* query);

dtStatus add_obstacles(
    ZrNavDetourTileCacheQuery* query,
    const ZrNavDetourTileCacheObstacle* obstacles,
    std::uint32_t obstacle_count,
    std::uint32_t* added_count) {
    *added_count = 0;
    for (std::uint32_t index = 0; index < obstacle_count; ++index) {
        if (*added_count > 0
            && (*added_count % ZR_NAV_TILE_CACHE_MAX_PENDING_REQUESTS) == 0) {
            const dtStatus update_status = update_until_current(query);
            if (dtStatusFailed(update_status)) {
                return update_status;
            }
        }
        dtObstacleRef obstacle_ref = 0;
        const dtStatus status = add_obstacle(query->tile_cache, obstacles[index], &obstacle_ref);
        if (dtStatusFailed(status)) {
            return status;
        }
        ++(*added_count);
    }
    return DT_SUCCESS;
}

dtStatus update_until_current(ZrNavDetourTileCacheQuery* query) {
    if (query == nullptr || query->tile_cache == nullptr || query->nav_mesh == nullptr) {
        return DT_FAILURE | DT_INVALID_PARAM;
    }
    bool up_to_date = false;
    dtStatus status = DT_SUCCESS;
    for (int iteration = 0; iteration < 64 && !up_to_date; ++iteration) {
        status = query->tile_cache->update(0.0f, query->nav_mesh, &up_to_date);
        if (dtStatusFailed(status)) {
            return status;
        }
    }
    return up_to_date ? DT_SUCCESS : DT_FAILURE | DT_BUFFER_TOO_SMALL;
}

bool find_nearest_poly(
    const ZrNavDetourTileCacheQuery* query,
    const float* position,
    const ZrNavTileCacheFilter& filter,
    dtPolyRef* out_ref,
    float* out_position) {
    if (query == nullptr || query->query == nullptr || !finite3(position)) {
        return false;
    }
    dtPolyRef poly_ref = 0;
    bool over_poly = false;
    float nearest[3] = {};
    const dtStatus status = query->query->findNearestPoly(position, query->query_extents, &filter, &poly_ref, nearest, &over_poly);
    if (dtStatusFailed(status) || poly_ref == 0) {
        return false;
    }
    *out_ref = poly_ref;
    copy3(nearest, out_position);
    return true;
}

unsigned char area_for_ref(const ZrNavDetourTileCacheQuery* query, dtPolyRef ref, unsigned char fallback) {
    if (query == nullptr || query->nav_mesh == nullptr || ref == 0) {
        return fallback;
    }
    unsigned char area = fallback;
    if (dtStatusSucceed(query->nav_mesh->getPolyArea(ref, &area))) {
        return area;
    }
    return fallback;
}

ZrNavDetourPathPoint* copy_straight_path(
    const ZrNavDetourTileCacheQuery* query,
    const float* straight_path,
    const unsigned char* straight_flags,
    const dtPolyRef* straight_refs,
    int straight_count,
    const dtPolyRef* corridor,
    int corridor_count,
    float* out_length) {
    if (straight_count <= 0) {
        return nullptr;
    }
    ZrNavDetourPathPoint* points = new (std::nothrow) ZrNavDetourPathPoint[straight_count];
    if (points == nullptr) {
        return nullptr;
    }
    *out_length = 0.0f;
    unsigned char previous_area = ZR_NAV_AREA_WALKABLE;
    for (int index = 0; index < straight_count; ++index) {
        const float* position = straight_path + index * 3;
        copy3(position, points[index].position);
        const dtPolyRef ref = straight_refs[index] != 0
            ? straight_refs[index]
            : (corridor_count > 0 ? corridor[corridor_count - 1] : 0);
        previous_area = area_for_ref(query, ref, previous_area);
        points[index].area = previous_area;
        points[index].flags = straight_flags[index];
        points[index].off_mesh_user_id =
            (straight_flags[index] & DT_STRAIGHTPATH_OFFMESH_CONNECTION) != 0
                ? zr_nav_off_mesh::user_id_for_ref(query->nav_mesh, ref)
                : 0;
        if (index > 0) {
            *out_length += distance3(points[index - 1].position, points[index].position);
        }
    }
    return points;
}

} // namespace

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
    ZrNavDetourTileCacheCreateResult* out_result) {
    try {
        reset_create_result(out_result);
        if (out_result == nullptr) {
            return;
        }
        if (vertices == nullptr || vertex_count == 0 || indices == nullptr || index_count == 0 || polygons == nullptr || polygon_count == 0) {
            set_create_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, "TileCache input has no navmesh polygons");
            return;
        }

        const char* error = nullptr;
        float bounds_min[3] = {};
        float bounds_max[3] = {};
        if (!compute_bounds(vertices, vertex_count, bounds_min, bounds_max, &error)) {
            set_create_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, error);
            return;
        }
        const float span_x = std::max(bounds_max[0] - bounds_min[0], 0.1f);
        const float span_z = std::max(bounds_max[2] - bounds_min[2], 0.1f);
        const float cell_size = choose_cell_size(std::max(span_x, span_z));
        const float cell_height = ZR_NAV_TILE_CACHE_CELL_HEIGHT;
        const int width = choose_cell_count(span_x, cell_size);
        const int height = choose_cell_count(span_z, cell_size);
        bounds_min[0] -= cell_size;
        bounds_min[1] -= cell_height;
        bounds_min[2] -= cell_size;
        bounds_max[0] = bounds_min[0] + static_cast<float>(width) * cell_size;
        bounds_max[1] += std::max(cell_height, 4.0f);
        bounds_max[2] = bounds_min[2] + static_cast<float>(height) * cell_size;

        std::vector<unsigned char> heights;
        std::vector<unsigned char> areas;
        std::vector<unsigned char> cons;
        if (!build_layer_data(vertices, vertex_count, indices, index_count, polygons, polygon_count, bounds_min, width, height, cell_size, &heights, &areas, &cons)) {
            set_create_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, "TileCache input has no walkable cells");
            return;
        }

        dtTileCacheLayerHeader header = {};
        header.magic = DT_TILECACHE_MAGIC;
        header.version = DT_TILECACHE_VERSION;
        header.tx = 0;
        header.ty = 0;
        header.tlayer = 0;
        copy3(bounds_min, header.bmin);
        copy3(bounds_max, header.bmax);
        header.hmin = 0;
        header.hmax = 1;
        header.width = static_cast<unsigned char>(width);
        header.height = static_cast<unsigned char>(height);
        header.minx = 0;
        header.maxx = static_cast<unsigned char>(width - 1);
        header.miny = 0;
        header.maxy = static_cast<unsigned char>(height - 1);

        ZrNavDetourTileCacheQuery* owner = new (std::nothrow) ZrNavDetourTileCacheQuery();
        if (owner == nullptr) {
            set_create_status(out_result, ZR_NAV_DETOUR_ERROR, "TileCache owner allocation failed");
            return;
        }
        if (!owner->mesh_process.assign_off_mesh_connections(
                off_mesh_links,
                off_mesh_link_count,
                &error)) {
            delete owner;
            set_create_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, error);
            return;
        }
        unsigned char* compressed_tile = nullptr;
        int compressed_tile_size = 0;
        dtStatus status = dtBuildTileCacheLayer(&owner->compressor, &header, heights.data(), areas.data(), cons.data(), &compressed_tile, &compressed_tile_size);
        if (dtStatusFailed(status) || compressed_tile == nullptr || compressed_tile_size <= 0) {
            delete owner;
            set_create_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, "TileCache layer build failed");
            return;
        }

        owner->tile_cache = dtAllocTileCache();
        owner->nav_mesh = dtAllocNavMesh();
        owner->query = dtAllocNavMeshQuery();
        if (owner->tile_cache == nullptr || owner->nav_mesh == nullptr || owner->query == nullptr) {
            dtFree(compressed_tile);
            zr_nav_tile_cache_free_query(owner);
            set_create_status(out_result, ZR_NAV_DETOUR_ERROR, "TileCache native allocation failed");
            return;
        }

        dtTileCacheParams tile_cache_params = {};
        copy3(bounds_min, tile_cache_params.orig);
        tile_cache_params.cs = cell_size;
        tile_cache_params.ch = cell_height;
        tile_cache_params.width = width;
        tile_cache_params.height = height;
        tile_cache_params.walkableHeight = 2.0f;
        tile_cache_params.walkableRadius = 0.0f;
        tile_cache_params.walkableClimb = 0.4f;
        tile_cache_params.maxSimplificationError = 1.3f;
        tile_cache_params.maxTiles = 4;
        tile_cache_params.maxObstacles = std::max(
            ZR_NAV_TILE_CACHE_MAX_OBSTACLES,
            static_cast<int>(obstacle_count) + 4);
        status = owner->tile_cache->init(&tile_cache_params, &owner->alloc, &owner->compressor, &owner->mesh_process);
        if (dtStatusFailed(status)) {
            dtFree(compressed_tile);
            zr_nav_tile_cache_free_query(owner);
            set_create_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, "TileCache initialization failed");
            return;
        }

        dtNavMeshParams nav_mesh_params = {};
        copy3(bounds_min, nav_mesh_params.orig);
        nav_mesh_params.tileWidth = static_cast<float>(width) * cell_size;
        nav_mesh_params.tileHeight = static_cast<float>(height) * cell_size;
        nav_mesh_params.maxTiles = 4;
        nav_mesh_params.maxPolys = std::max(width * height, 64);
        status = owner->nav_mesh->init(&nav_mesh_params);
        if (dtStatusFailed(status)) {
            dtFree(compressed_tile);
            zr_nav_tile_cache_free_query(owner);
            set_create_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, "TileCache navmesh initialization failed");
            return;
        }

        dtCompressedTileRef tile_ref = 0;
        status = owner->tile_cache->addTile(compressed_tile, compressed_tile_size, DT_COMPRESSEDTILE_FREE_DATA, &tile_ref);
        if (dtStatusFailed(status)) {
            dtFree(compressed_tile);
            zr_nav_tile_cache_free_query(owner);
            set_create_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, "TileCache could not add compressed tile");
            return;
        }
        status = owner->tile_cache->buildNavMeshTile(tile_ref, owner->nav_mesh);
        if (dtStatusFailed(status)) {
            zr_nav_tile_cache_free_query(owner);
            set_create_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, "TileCache could not build base navmesh tile");
            return;
        }

        std::uint32_t added_obstacles = 0;
        status = add_obstacles(owner, obstacles, obstacle_count, &added_obstacles);
        if (dtStatusFailed(status)) {
            zr_nav_tile_cache_free_query(owner);
            set_create_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, "TileCache could not add obstacle");
            return;
        }
        status = update_until_current(owner);
        if (dtStatusFailed(status)) {
            zr_nav_tile_cache_free_query(owner);
            set_create_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, "TileCache obstacle update failed");
            return;
        }

        const int max_nodes = std::clamp(width * height + 64, 64, 65535);
        status = owner->query->init(owner->nav_mesh, max_nodes);
        if (dtStatusFailed(status)) {
            zr_nav_tile_cache_free_query(owner);
            set_create_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, "TileCache query initialization failed");
            return;
        }
        owner->query_extents[0] = std::max(span_x * 0.5f + 1.0f, 1.0f);
        owner->query_extents[1] = std::max(bounds_max[1] - bounds_min[1] + 1.0f, 4.0f);
        owner->query_extents[2] = std::max(span_z * 0.5f + 1.0f, 1.0f);
        owner->polygon_count = polygon_count;
        owner->obstacle_count = added_obstacles;
        initialize_area_tables(owner->area_costs, owner->area_walkable, area_costs, area_cost_count);

        out_result->query = owner;
        out_result->polygon_count = polygon_count;
        out_result->obstacle_count = added_obstacles;
        set_create_status(out_result, ZR_NAV_DETOUR_OK, "TileCache query created");
    } catch (const std::bad_alloc&) {
        set_create_status(out_result, ZR_NAV_DETOUR_ERROR, "TileCache allocation failed");
    } catch (const std::exception& error) {
        set_create_status(out_result, ZR_NAV_DETOUR_ERROR, error.what());
    } catch (...) {
        set_create_status(out_result, ZR_NAV_DETOUR_ERROR, "TileCache failed with an unknown native exception");
    }
}

extern "C" void zr_nav_tile_cache_free_query(ZrNavDetourTileCacheQuery* query) {
    if (query == nullptr) {
        return;
    }
    if (query->query != nullptr) {
        dtFreeNavMeshQuery(query->query);
    }
    if (query->nav_mesh != nullptr) {
        dtFreeNavMesh(query->nav_mesh);
    }
    if (query->tile_cache != nullptr) {
        dtFreeTileCache(query->tile_cache);
    }
    delete query;
}

extern "C" void zr_nav_tile_cache_add_obstacle(
    ZrNavDetourTileCacheQuery* query,
    const ZrNavDetourTileCacheObstacle* obstacle,
    ZrNavDetourTileCacheCommandResult* out_result) {
    try {
        reset_command_result(out_result);
        if (out_result == nullptr || query == nullptr || query->tile_cache == nullptr || obstacle == nullptr) {
            set_command_status(out_result, ZR_NAV_DETOUR_ERROR, "TileCache add obstacle input is invalid");
            return;
        }
        dtObstacleRef obstacle_ref = 0;
        const dtStatus status = add_obstacle(query->tile_cache, *obstacle, &obstacle_ref);
        if (dtStatusFailed(status) || obstacle_ref == 0) {
            set_command_status(out_result, ZR_NAV_DETOUR_ERROR, "TileCache could not add obstacle");
            return;
        }
        ++query->obstacle_count;
        set_command_status(out_result, ZR_NAV_DETOUR_OK, "TileCache obstacle queued", obstacle_ref);
    } catch (const std::exception& error) {
        set_command_status(out_result, ZR_NAV_DETOUR_ERROR, error.what());
    } catch (...) {
        set_command_status(out_result, ZR_NAV_DETOUR_ERROR, "TileCache add obstacle failed");
    }
}

extern "C" void zr_nav_tile_cache_remove_obstacle(
    ZrNavDetourTileCacheQuery* query,
    std::uint64_t obstacle_ref,
    ZrNavDetourTileCacheCommandResult* out_result) {
    try {
        reset_command_result(out_result);
        if (out_result == nullptr || query == nullptr || query->tile_cache == nullptr || obstacle_ref == 0) {
            set_command_status(out_result, ZR_NAV_DETOUR_ERROR, "TileCache remove obstacle input is invalid");
            return;
        }
        const dtObstacleRef native_ref = static_cast<dtObstacleRef>(obstacle_ref);
        const dtStatus status = query->tile_cache->removeObstacle(native_ref);
        if (dtStatusFailed(status)) {
            set_command_status(out_result, ZR_NAV_DETOUR_ERROR, "TileCache could not remove obstacle");
            return;
        }
        query->obstacle_count = query->obstacle_count > 0 ? query->obstacle_count - 1 : 0;
        set_command_status(out_result, ZR_NAV_DETOUR_OK, "TileCache obstacle removal queued", native_ref);
    } catch (const std::exception& error) {
        set_command_status(out_result, ZR_NAV_DETOUR_ERROR, error.what());
    } catch (...) {
        set_command_status(out_result, ZR_NAV_DETOUR_ERROR, "TileCache remove obstacle failed");
    }
}

extern "C" void zr_nav_tile_cache_update(
    ZrNavDetourTileCacheQuery* query,
    ZrNavDetourTileCacheCommandResult* out_result) {
    try {
        reset_command_result(out_result);
        const dtStatus status = update_until_current(query);
        if (dtStatusFailed(status)) {
            set_command_status(out_result, ZR_NAV_DETOUR_ERROR, "TileCache update failed");
            return;
        }
        set_command_status(out_result, ZR_NAV_DETOUR_OK, "TileCache is current");
    } catch (const std::exception& error) {
        set_command_status(out_result, ZR_NAV_DETOUR_ERROR, error.what());
    } catch (...) {
        set_command_status(out_result, ZR_NAV_DETOUR_ERROR, "TileCache update failed");
    }
}

extern "C" void zr_nav_tile_cache_find_path(
    const ZrNavDetourTileCacheQuery* query,
    const float* start,
    const float* end,
    std::uint64_t area_mask,
    const ZrNavDetourQueryFilter* query_filter,
    ZrNavDetourPathResult* out_result) {
    try {
        reset_path_result(out_result);
        if (out_result == nullptr) {
            return;
        }
        if (query == nullptr || query->query == nullptr || !finite3(start) || !finite3(end)) {
            set_path_status(out_result, ZR_NAV_DETOUR_ERROR, "TileCache path query input is invalid");
            return;
        }
        ZrNavTileCacheFilter filter(*query, area_mask, query_filter);
        dtPolyRef start_ref = 0;
        dtPolyRef end_ref = 0;
        float start_nearest[3] = {};
        float end_nearest[3] = {};
        if (!find_nearest_poly(query, start, filter, &start_ref, start_nearest)
            || !find_nearest_poly(query, end, filter, &end_ref, end_nearest)) {
            set_path_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, "TileCache path query found no start or end polygon");
            return;
        }

        dtPolyRef corridor[ZR_NAV_TILE_CACHE_MAX_PATH] = {};
        int corridor_count = 0;
        dtStatus status = query->query->findPath(
            start_ref,
            end_ref,
            start_nearest,
            end_nearest,
            &filter,
            corridor,
            &corridor_count,
            ZR_NAV_TILE_CACHE_MAX_PATH);
        if (dtStatusFailed(status) || corridor_count == 0 || corridor[corridor_count - 1] != end_ref) {
            set_path_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, "TileCache path query found no complete path");
            return;
        }

        float straight_path[ZR_NAV_TILE_CACHE_MAX_STRAIGHT_PATH * 3] = {};
        unsigned char straight_flags[ZR_NAV_TILE_CACHE_MAX_STRAIGHT_PATH] = {};
        dtPolyRef straight_refs[ZR_NAV_TILE_CACHE_MAX_STRAIGHT_PATH] = {};
        int straight_count = 0;
        status = query->query->findStraightPath(
            start_nearest,
            end_nearest,
            corridor,
            corridor_count,
            straight_path,
            straight_flags,
            straight_refs,
            &straight_count,
            ZR_NAV_TILE_CACHE_MAX_STRAIGHT_PATH,
            DT_STRAIGHTPATH_AREA_CROSSINGS);
        if (dtStatusFailed(status) || straight_count == 0) {
            set_path_status(out_result, ZR_NAV_DETOUR_ERROR, "TileCache straight path query failed");
            return;
        }

        float length = 0.0f;
        ZrNavDetourPathPoint* points = copy_straight_path(
            query,
            straight_path,
            straight_flags,
            straight_refs,
            straight_count,
            corridor,
            corridor_count,
            &length);
        if (points == nullptr) {
            set_path_status(out_result, ZR_NAV_DETOUR_ERROR, "TileCache path result allocation failed");
            return;
        }
        out_result->points = points;
        out_result->point_count = static_cast<std::uint32_t>(straight_count);
        out_result->visited_nodes = static_cast<std::uint32_t>(corridor_count);
        out_result->length = length;
        set_path_status(out_result, ZR_NAV_DETOUR_OK, "TileCache path query completed");
    } catch (const std::bad_alloc&) {
        set_path_status(out_result, ZR_NAV_DETOUR_ERROR, "TileCache path query allocation failed");
    } catch (const std::exception& error) {
        set_path_status(out_result, ZR_NAV_DETOUR_ERROR, error.what());
    } catch (...) {
        set_path_status(out_result, ZR_NAV_DETOUR_ERROR, "TileCache path query failed with an unknown native exception");
    }
}
