#include <cstdint>

#include "DetourCommon.h"
#include "DetourCrowd.h"
#include "DetourNavMesh.h"
#include "DetourTileCache.h"
#include "Recast.h"

extern "C" std::uint32_t zr_nav_recast_bridge_version() {
    return 1;
}

extern "C" std::uint32_t zr_nav_recast_runtime_modules_smoke() {
    dtNavMesh* nav_mesh = dtAllocNavMesh();
    dtCrowd* crowd = dtAllocCrowd();
    dtTileCache* tile_cache = dtAllocTileCache();

    const bool detour_ok = nav_mesh != nullptr;
    const bool crowd_ok = crowd != nullptr;
    const bool tile_cache_ok = tile_cache != nullptr;

    if (tile_cache != nullptr) {
        dtFreeTileCache(tile_cache);
    }
    if (crowd != nullptr) {
        dtFreeCrowd(crowd);
    }
    if (nav_mesh != nullptr) {
        dtFreeNavMesh(nav_mesh);
    }

    float vertices[9] = {
        -1.0f, 0.0f, -1.0f,
         1.0f, 0.0f, -1.0f,
         0.0f, 0.0f,  1.0f,
    };
    float bounds_min[3] = {};
    float bounds_max[3] = {};
    rcCalcBounds(vertices, 3, bounds_min, bounds_max);
    const bool recast_ok = bounds_min[0] <= -1.0f && bounds_max[0] >= 1.0f;

    return (detour_ok && crowd_ok && tile_cache_ok && recast_ok) ? 1u : 0u;
}

extern "C" float zr_nav_recast_polyline_length(
    const float* xyz,
    std::uint64_t point_count
) {
    if (xyz == nullptr || point_count < 2) {
        return 0.0f;
    }

    float length = 0.0f;
    for (std::uint64_t index = 1; index < point_count; ++index) {
        const float* previous = xyz + ((index - 1) * 3);
        const float* current = xyz + (index * 3);
        length += dtVdist(previous, current);
    }
    return length;
}
