#include "detour_off_mesh_connections.h"

#include <algorithm>
#include <cmath>
#include <iterator>
#include <unordered_set>

namespace {

constexpr std::uint8_t ZR_NAV_AREA_WALKABLE = 1;
constexpr float ZR_NAV_MIN_OFF_MESH_RADIUS = 0.05f;

bool finite3(const float* value) {
    return value != nullptr
        && std::isfinite(value[0])
        && std::isfinite(value[1])
        && std::isfinite(value[2]);
}

unsigned short area_flag(const unsigned char area) {
    if (area == 0) {
        return 0;
    }
    return area <= 15 ? static_cast<unsigned short>(1u << (area - 1))
                      : static_cast<unsigned short>(1u << 15);
}

bool starts_in_tile(const ZrNavDetourOffMeshLink& link, const dtNavMeshCreateParams& params) {
    return link.start[0] >= params.bmin[0] && link.start[0] < params.bmax[0]
        && link.start[2] >= params.bmin[2] && link.start[2] < params.bmax[2];
}

} // namespace

namespace zr_nav_off_mesh {

bool ConnectionSet::assign(
    const ZrNavDetourOffMeshLink* links,
    const std::uint32_t link_count,
    const char** error) {
    links_.clear();
    if (link_count == 0) {
        return true;
    }
    if (links == nullptr) {
        *error = "off-mesh link count requires link data";
        return false;
    }

    std::unordered_set<std::uint32_t> user_ids;
    links_.reserve(link_count);
    for (std::uint32_t index = 0; index < link_count; ++index) {
        const ZrNavDetourOffMeshLink& link = links[index];
        if (!finite3(link.start) || !finite3(link.end)) {
            *error = "off-mesh link contains a non-finite endpoint";
            links_.clear();
            return false;
        }
        if (link.user_id == 0 || !user_ids.insert(link.user_id).second) {
            *error = "off-mesh link user ids must be non-zero and unique";
            links_.clear();
            return false;
        }
        links_.push_back(link);
    }
    return true;
}

void ConnectionSet::bind_all(dtNavMeshCreateParams* params) {
    std::vector<std::size_t> indices;
    indices.reserve(links_.size());
    for (std::size_t index = 0; index < links_.size(); ++index) {
        indices.push_back(index);
    }
    bind_indices(params, indices);
}

void ConnectionSet::bind_for_tile(dtNavMeshCreateParams* params) {
    std::vector<std::size_t> indices;
    indices.reserve(links_.size());
    for (std::size_t index = 0; index < links_.size(); ++index) {
        if (starts_in_tile(links_[index], *params)) {
            indices.push_back(index);
        }
    }
    bind_indices(params, indices);
}

void ConnectionSet::bind_indices(
    dtNavMeshCreateParams* params,
    const std::vector<std::size_t>& indices) {
    vertices_.clear();
    radii_.clear();
    flags_.clear();
    areas_.clear();
    directions_.clear();
    user_ids_.clear();
    vertices_.reserve(indices.size() * 6);
    radii_.reserve(indices.size());
    flags_.reserve(indices.size());
    areas_.reserve(indices.size());
    directions_.reserve(indices.size());
    user_ids_.reserve(indices.size());

    for (const std::size_t index : indices) {
        const ZrNavDetourOffMeshLink& link = links_[index];
        vertices_.insert(vertices_.end(), std::begin(link.start), std::end(link.start));
        vertices_.insert(vertices_.end(), std::begin(link.end), std::end(link.end));
        radii_.push_back(
            std::isfinite(link.radius) ? std::max(link.radius, ZR_NAV_MIN_OFF_MESH_RADIUS)
                                       : ZR_NAV_MIN_OFF_MESH_RADIUS);
        const unsigned char area = link.area < DT_MAX_AREAS ? link.area : ZR_NAV_AREA_WALKABLE;
        flags_.push_back(area_flag(area));
        areas_.push_back(area);
        directions_.push_back(link.bidirectional != 0 ? DT_OFFMESH_CON_BIDIR : 0);
        user_ids_.push_back(link.user_id);
    }

    params->offMeshConVerts = vertices_.empty() ? nullptr : vertices_.data();
    params->offMeshConRad = radii_.empty() ? nullptr : radii_.data();
    params->offMeshConFlags = flags_.empty() ? nullptr : flags_.data();
    params->offMeshConAreas = areas_.empty() ? nullptr : areas_.data();
    params->offMeshConDir = directions_.empty() ? nullptr : directions_.data();
    params->offMeshConUserID = user_ids_.empty() ? nullptr : user_ids_.data();
    params->offMeshConCount = static_cast<int>(indices.size());
}

std::uint32_t user_id_for_ref(const dtNavMesh* nav_mesh, const dtPolyRef ref) {
    if (nav_mesh == nullptr || ref == 0) {
        return 0;
    }
    const dtOffMeshConnection* connection = nav_mesh->getOffMeshConnectionByRef(ref);
    return connection == nullptr ? 0 : connection->userId;
}

} // namespace zr_nav_off_mesh
