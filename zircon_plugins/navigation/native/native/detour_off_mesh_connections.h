#pragma once

#include "recast_bridge.h"

#include <cstddef>
#include <cstdint>
#include <vector>

#include "DetourNavMesh.h"
#include "DetourNavMeshBuilder.h"

namespace zr_nav_off_mesh {

/// Owns the arrays referenced by dtNavMeshCreateParams while one tile is built.
class ConnectionSet {
public:
    bool assign(
        const ZrNavDetourOffMeshLink* links,
        std::uint32_t link_count,
        const char** error);

    void bind_all(dtNavMeshCreateParams* params);
    void bind_for_tile(dtNavMeshCreateParams* params);

private:
    void bind_indices(dtNavMeshCreateParams* params, const std::vector<std::size_t>& indices);

    std::vector<ZrNavDetourOffMeshLink> links_;
    std::vector<float> vertices_;
    std::vector<float> radii_;
    std::vector<unsigned short> flags_;
    std::vector<unsigned char> areas_;
    std::vector<unsigned char> directions_;
    std::vector<unsigned int> user_ids_;
};

std::uint32_t user_id_for_ref(const dtNavMesh* nav_mesh, dtPolyRef ref);

} // namespace zr_nav_off_mesh
