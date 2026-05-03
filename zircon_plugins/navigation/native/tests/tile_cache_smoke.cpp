#include "zircon_plugins/navigation/native/native/recast_bridge.h"

#include <cstdint>
#include <cstdio>

int main() {
    float vertices[] = {
        0.0f, 0.0f, 0.0f,
        1.0f, 0.0f, 0.0f,
        2.0f, 0.0f, 0.0f,
        3.0f, 0.0f, 0.0f,
        0.0f, 0.0f, 1.0f,
        1.0f, 0.0f, 1.0f,
        2.0f, 0.0f, 1.0f,
        3.0f, 0.0f, 1.0f,
    };
    std::uint32_t indices[] = {0, 1, 5, 0, 5, 4, 1, 2, 6, 1, 6, 5, 2, 3, 7, 2, 7, 6};
    ZrNavRecastBakePolygon polygons[] = {
        {0, 6, 1, 0},
        {6, 6, 1, 0},
        {12, 6, 1, 0},
    };
    ZrNavDetourAreaCost costs[] = {{1, 1.0f, 1}};
    ZrNavDetourTileCacheObstacle obstacles[] = {
        {{1.5f, 0.0f, 0.5f}, {0.55f, 1.0f, 0.6f}, 0.6f, 2.0f, 1},
    };

    ZrNavDetourTileCacheCreateResult create{};
    zr_nav_tile_cache_create_query(
        vertices,
        8,
        indices,
        18,
        polygons,
        3,
        costs,
        1,
        nullptr,
        0,
        obstacles,
        1,
        &create);
    std::printf(
        "create status=%u polygons=%u obstacles=%u message=%s\n",
        create.status,
        create.polygon_count,
        create.obstacle_count,
        create.message);
    if (create.status != 1 || create.query == nullptr) {
        return 2;
    }

    float start[] = {0.2f, 0.0f, 0.5f};
    float end[] = {2.8f, 0.0f, 0.5f};
    ZrNavDetourPathResult path{};
    zr_nav_tile_cache_find_path(create.query, start, end, ~std::uint64_t(0), &path);
    std::printf(
        "path status=%u points=%u visited=%u length=%f message=%s\n",
        path.status,
        path.point_count,
        path.visited_nodes,
        path.length,
        path.message);
    const std::uint32_t status = path.status;
    zr_nav_detour_free_path_result(&path);
    zr_nav_tile_cache_free_query(create.query);
    return status == 2 ? 0 : 3;
}
