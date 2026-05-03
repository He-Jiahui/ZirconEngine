#include "recast_bridge.h"

#include <algorithm>
#include <cmath>
#include <cstring>
#include <exception>
#include <iterator>
#include <limits>
#include <new>
#include <vector>

#include "DetourAlloc.h"
#include "DetourCommon.h"
#include "DetourNavMesh.h"
#include "DetourNavMeshBuilder.h"
#include "DetourNavMeshQuery.h"
#include "DetourStatus.h"

struct ZrNavDetourQuery {
    dtNavMesh* nav_mesh = nullptr;
    dtNavMeshQuery* query = nullptr;
    float bounds_min[3] = {};
    float bounds_max[3] = {};
    float query_extents[3] = { 1.0f, 1.0f, 1.0f };
    float area_costs[DT_MAX_AREAS] = {};
    bool area_walkable[DT_MAX_AREAS] = {};
    std::uint32_t polygon_count = 0;
};

namespace {

constexpr std::uint32_t ZR_NAV_DETOUR_ERROR = 0;
constexpr std::uint32_t ZR_NAV_DETOUR_OK = 1;
constexpr std::uint32_t ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH = 2;
constexpr std::uint8_t ZR_NAV_AREA_WALKABLE = 1;
constexpr unsigned short ZR_NAV_DETOUR_NULL_VERTEX = 0xffffu;
constexpr int ZR_NAV_DETOUR_MAX_VERTS_PER_POLYGON = DT_VERTS_PER_POLYGON;
constexpr int ZR_NAV_DETOUR_MAX_PATH = 512;
constexpr int ZR_NAV_DETOUR_MAX_STRAIGHT_PATH = 512;
constexpr float ZR_NAV_DETOUR_MIN_CELL = 0.001f;

class ZrNavDetourFilter final : public dtQueryFilter {
public:
    ZrNavDetourFilter(const ZrNavDetourQuery& owner, std::uint64_t area_mask)
        : m_area_mask(area_mask) {
        for (int index = 0; index < DT_MAX_AREAS; ++index) {
            m_area_costs[index] = owner.area_costs[index];
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
    float m_area_costs[DT_MAX_AREAS] = {};
    bool m_area_walkable[DT_MAX_AREAS] = {};
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

void reset_create_result(ZrNavDetourQueryCreateResult* result) {
    if (result == nullptr) {
        return;
    }
    result->status = ZR_NAV_DETOUR_ERROR;
    set_message(result->message, sizeof(result->message), "");
    result->query = nullptr;
    result->polygon_count = 0;
}

void set_create_status(ZrNavDetourQueryCreateResult* result, std::uint32_t status, const char* message) {
    if (result == nullptr) {
        return;
    }
    result->status = status;
    set_message(result->message, sizeof(result->message), message);
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

void reset_sample_result(ZrNavDetourSampleResult* result) {
    if (result == nullptr) {
        return;
    }
    result->status = ZR_NAV_DETOUR_ERROR;
    set_message(result->message, sizeof(result->message), "");
    result->hit = 0;
    result->position[0] = 0.0f;
    result->position[1] = 0.0f;
    result->position[2] = 0.0f;
    result->distance = 0.0f;
    result->area = ZR_NAV_AREA_WALKABLE;
}

void set_sample_status(ZrNavDetourSampleResult* result, std::uint32_t status, const char* message) {
    if (result == nullptr) {
        return;
    }
    result->status = status;
    set_message(result->message, sizeof(result->message), message);
}

void reset_raycast_result(ZrNavDetourRaycastResult* result) {
    if (result == nullptr) {
        return;
    }
    result->status = ZR_NAV_DETOUR_ERROR;
    set_message(result->message, sizeof(result->message), "");
    result->hit = 0;
    result->position[0] = 0.0f;
    result->position[1] = 0.0f;
    result->position[2] = 0.0f;
    result->normal[0] = 0.0f;
    result->normal[1] = 1.0f;
    result->normal[2] = 0.0f;
    result->distance = 0.0f;
    result->visited_nodes = 0;
}

void set_raycast_status(ZrNavDetourRaycastResult* result, std::uint32_t status, const char* message) {
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

float distance3(const float* left, const float* right) {
    const float dx = right[0] - left[0];
    const float dy = right[1] - left[1];
    const float dz = right[2] - left[2];
    return std::sqrt(dx * dx + dy * dy + dz * dz);
}

void lerp3(const float* from, const float* to, float t, float* output) {
    output[0] = from[0] + (to[0] - from[0]) * t;
    output[1] = from[1] + (to[1] - from[1]) * t;
    output[2] = from[2] + (to[2] - from[2]) * t;
}

void copy3(const float* source, float* target) {
    target[0] = source[0];
    target[1] = source[1];
    target[2] = source[2];
}

void initialize_area_tables(
    ZrNavDetourQuery* query,
    const ZrNavDetourAreaCost* area_costs,
    std::uint32_t area_cost_count) {
    for (int index = 0; index < DT_MAX_AREAS; ++index) {
        query->area_costs[index] = 1.0f;
        query->area_walkable[index] = index != 0;
    }
    for (std::uint32_t index = 0; index < area_cost_count; ++index) {
        const ZrNavDetourAreaCost& cost = area_costs[index];
        if (cost.area >= DT_MAX_AREAS) {
            continue;
        }
        query->area_costs[cost.area] = std::isfinite(cost.cost) ? std::max(cost.cost, 0.001f) : 1.0f;
        query->area_walkable[cost.area] = cost.walkable != 0;
    }
}

bool compute_bounds(
    const float* vertices,
    std::uint32_t vertex_count,
    const ZrNavDetourOffMeshLink* off_mesh_links,
    std::uint32_t off_mesh_link_count,
    float* bmin,
    float* bmax,
    const char** error) {
    if (vertices == nullptr || vertex_count == 0) {
        *error = "Detour query input has no vertices";
        return false;
    }
    const float* first = vertices;
    if (!finite3(first)) {
        *error = "Detour query input contains non-finite vertices";
        return false;
    }
    copy3(first, bmin);
    copy3(first, bmax);
    for (std::uint32_t index = 0; index < vertex_count; ++index) {
        const float* vertex = vertices + index * 3;
        if (!finite3(vertex)) {
            *error = "Detour query input contains non-finite vertices";
            return false;
        }
        for (int axis = 0; axis < 3; ++axis) {
            bmin[axis] = std::min(bmin[axis], vertex[axis]);
            bmax[axis] = std::max(bmax[axis], vertex[axis]);
        }
    }
    for (std::uint32_t index = 0; index < off_mesh_link_count; ++index) {
        const ZrNavDetourOffMeshLink& link = off_mesh_links[index];
        if (!finite3(link.start) || !finite3(link.end)) {
            *error = "Detour query input contains non-finite off-mesh links";
            return false;
        }
        for (int axis = 0; axis < 3; ++axis) {
            bmin[axis] = std::min({ bmin[axis], link.start[axis], link.end[axis] });
            bmax[axis] = std::max({ bmax[axis], link.start[axis], link.end[axis] });
        }
    }
    return true;
}

float quantization_cell(float span) {
    if (!std::isfinite(span) || span <= 0.0f) {
        return ZR_NAV_DETOUR_MIN_CELL;
    }
    return std::max(ZR_NAV_DETOUR_MIN_CELL, span / 60000.0f);
}

unsigned short quantize_axis(float value, float origin, float cell) {
    const float units = (value - origin) / cell;
    const float rounded = std::round(units);
    const float clamped = std::clamp(rounded, 0.0f, 65535.0f);
    return static_cast<unsigned short>(clamped);
}

std::vector<unsigned short> quantize_vertices(
    const float* vertices,
    std::uint32_t vertex_count,
    const float* bmin,
    float cell_size,
    float cell_height) {
    std::vector<unsigned short> output;
    output.reserve(static_cast<std::size_t>(vertex_count) * 3);
    for (std::uint32_t index = 0; index < vertex_count; ++index) {
        const float* vertex = vertices + index * 3;
        output.push_back(quantize_axis(vertex[0], bmin[0], cell_size));
        output.push_back(quantize_axis(vertex[1], bmin[1], cell_height));
        output.push_back(quantize_axis(vertex[2], bmin[2], cell_size));
    }
    return output;
}

bool contains_index(const std::vector<std::uint32_t>& values, std::uint32_t value) {
    return std::find(values.begin(), values.end(), value) != values.end();
}

bool collect_polygon_vertices(
    const float* vertices,
    const std::uint32_t* indices,
    std::uint32_t index_count,
    const ZrNavRecastBakePolygon* polygon,
    std::uint32_t vertex_count,
    std::vector<std::uint32_t>* output,
    const char** error) {
    const std::uint32_t start = polygon->first_index;
    const std::uint32_t end = start + polygon->index_count;
    if (start > index_count || end > index_count || end < start) {
        *error = "Detour query polygon references missing indices";
        return false;
    }
    for (std::uint32_t offset = start; offset < end; ++offset) {
        const std::uint32_t vertex = indices[offset];
        if (vertex >= vertex_count || vertex > std::numeric_limits<unsigned short>::max()) {
            *error = "Detour query polygon references a missing vertex";
            return false;
        }
        if (!contains_index(*output, vertex)) {
            output->push_back(vertex);
        }
    }
    if (output->size() < 3 || output->size() > ZR_NAV_DETOUR_MAX_VERTS_PER_POLYGON) {
        *error = "Detour query supports polygons with 3-6 unique vertices";
        return false;
    }
    const float* a = vertices + (*output)[0] * 3;
    const float* b = vertices + (*output)[1] * 3;
    const float* c = vertices + (*output)[2] * 3;
    const float normal_y = (b[2] - a[2]) * (c[0] - a[0]) - (b[0] - a[0]) * (c[2] - a[2]);
    if (normal_y < 0.0f) {
        std::reverse(output->begin() + 1, output->end());
    }
    return true;
}

void connect_polygon_neighbours(
    const std::vector<std::vector<std::uint32_t>>& polygon_vertices,
    std::vector<unsigned short>* detour_polys,
    int nvp) {
    for (std::size_t left = 0; left < polygon_vertices.size(); ++left) {
        const std::vector<std::uint32_t>& left_vertices = polygon_vertices[left];
        for (std::size_t left_edge = 0; left_edge < left_vertices.size(); ++left_edge) {
            const std::uint32_t left_a = left_vertices[left_edge];
            const std::uint32_t left_b = left_vertices[(left_edge + 1) % left_vertices.size()];
            for (std::size_t right = 0; right < polygon_vertices.size(); ++right) {
                if (left == right) {
                    continue;
                }
                const std::vector<std::uint32_t>& right_vertices = polygon_vertices[right];
                for (std::size_t right_edge = 0; right_edge < right_vertices.size(); ++right_edge) {
                    const std::uint32_t right_a = right_vertices[right_edge];
                    const std::uint32_t right_b = right_vertices[(right_edge + 1) % right_vertices.size()];
                    if (left_a == right_b && left_b == right_a && right <= std::numeric_limits<unsigned short>::max()) {
                        const std::size_t offset = left * static_cast<std::size_t>(nvp * 2) + nvp + left_edge;
                        (*detour_polys)[offset] = static_cast<unsigned short>(right);
                    }
                }
            }
        }
    }
}

bool build_detour_polys(
    const float* vertices,
    const std::uint32_t* indices,
    std::uint32_t index_count,
    std::uint32_t vertex_count,
    const ZrNavRecastBakePolygon* polygons,
    std::uint32_t polygon_count,
    std::vector<unsigned short>* detour_polys,
    std::vector<unsigned short>* poly_flags,
    std::vector<unsigned char>* poly_areas,
    const char** error) {
    if (indices == nullptr || polygons == nullptr || polygon_count == 0) {
        *error = "Detour query input has no polygons";
        return false;
    }
    if (vertex_count > std::numeric_limits<unsigned short>::max()) {
        *error = "Detour query input has too many vertices for single-tile Detour data";
        return false;
    }

    const int nvp = ZR_NAV_DETOUR_MAX_VERTS_PER_POLYGON;
    detour_polys->assign(static_cast<std::size_t>(polygon_count) * nvp * 2, 0);
    poly_flags->assign(polygon_count, 1);
    poly_areas->assign(polygon_count, ZR_NAV_AREA_WALKABLE);
    std::vector<std::vector<std::uint32_t>> polygon_vertices;
    polygon_vertices.reserve(polygon_count);

    for (std::uint32_t polygon_index = 0; polygon_index < polygon_count; ++polygon_index) {
        std::vector<std::uint32_t> unique_vertices;
        if (!collect_polygon_vertices(vertices, indices, index_count, &polygons[polygon_index], vertex_count, &unique_vertices, error)) {
            return false;
        }
        const std::size_t polygon_offset = static_cast<std::size_t>(polygon_index) * nvp * 2;
        for (int vertex_index = 0; vertex_index < nvp; ++vertex_index) {
            (*detour_polys)[polygon_offset + vertex_index] = ZR_NAV_DETOUR_NULL_VERTEX;
        }
        for (std::size_t vertex_index = 0; vertex_index < unique_vertices.size(); ++vertex_index) {
            (*detour_polys)[polygon_offset + vertex_index] = static_cast<unsigned short>(unique_vertices[vertex_index]);
        }
        (*poly_areas)[polygon_index] = polygons[polygon_index].area < DT_MAX_AREAS
            ? polygons[polygon_index].area
            : ZR_NAV_AREA_WALKABLE;
        polygon_vertices.push_back(std::move(unique_vertices));
    }

    connect_polygon_neighbours(polygon_vertices, detour_polys, nvp);
    return true;
}

std::vector<float> copy_off_mesh_vertices(
    const ZrNavDetourOffMeshLink* links,
    std::uint32_t link_count) {
    std::vector<float> output;
    output.reserve(static_cast<std::size_t>(link_count) * 6);
    for (std::uint32_t index = 0; index < link_count; ++index) {
        output.insert(output.end(), std::begin(links[index].start), std::end(links[index].start));
        output.insert(output.end(), std::begin(links[index].end), std::end(links[index].end));
    }
    return output;
}

void fill_off_mesh_arrays(
    const ZrNavDetourOffMeshLink* links,
    std::uint32_t link_count,
    std::vector<float>* radii,
    std::vector<unsigned short>* flags,
    std::vector<unsigned char>* areas,
    std::vector<unsigned char>* directions,
    std::vector<unsigned int>* user_ids) {
    radii->reserve(link_count);
    flags->reserve(link_count);
    areas->reserve(link_count);
    directions->reserve(link_count);
    user_ids->reserve(link_count);
    for (std::uint32_t index = 0; index < link_count; ++index) {
        const ZrNavDetourOffMeshLink& link = links[index];
        radii->push_back(std::isfinite(link.radius) ? std::max(link.radius, 0.05f) : 0.05f);
        flags->push_back(1);
        areas->push_back(link.area < DT_MAX_AREAS ? link.area : ZR_NAV_AREA_WALKABLE);
        directions->push_back(link.bidirectional != 0 ? DT_OFFMESH_CON_BIDIR : 0);
        user_ids->push_back(index + 1);
    }
}

unsigned char area_for_ref(const ZrNavDetourQuery* query, dtPolyRef ref, unsigned char fallback) {
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
    const ZrNavDetourQuery* query,
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
        if (index > 0) {
            *out_length += distance3(points[index - 1].position, points[index].position);
        }
    }
    return points;
}

bool find_nearest_poly(
    const ZrNavDetourQuery* query,
    const float* position,
    const float* extents,
    const ZrNavDetourFilter& filter,
    dtPolyRef* out_ref,
    float* out_position,
    bool* out_over_poly) {
    if (query == nullptr || query->query == nullptr || !finite3(position) || !finite3(extents)) {
        return false;
    }
    dtPolyRef poly_ref = 0;
    bool over_poly = false;
    float nearest[3] = {};
    const dtStatus status = query->query->findNearestPoly(position, extents, &filter, &poly_ref, nearest, &over_poly);
    if (dtStatusFailed(status) || poly_ref == 0) {
        return false;
    }
    *out_ref = poly_ref;
    copy3(nearest, out_position);
    if (out_over_poly != nullptr) {
        *out_over_poly = over_poly;
    }
    return true;
}

void sanitized_extents(const float* input, float* output) {
    for (int axis = 0; axis < 3; ++axis) {
        const float value = input == nullptr ? 0.0f : input[axis];
        output[axis] = std::isfinite(value) ? std::max(std::abs(value), ZR_NAV_DETOUR_MIN_CELL) : ZR_NAV_DETOUR_MIN_CELL;
    }
}

} // namespace

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
    ZrNavDetourQueryCreateResult* out_result) {
    try {
        reset_create_result(out_result);
        if (out_result == nullptr) {
            return;
        }
        if (vertices == nullptr || vertex_count == 0 || indices == nullptr || index_count == 0 || polygons == nullptr || polygon_count == 0) {
            set_create_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, "Detour query input has no navmesh polygons");
            return;
        }

        const char* error = nullptr;
        float bounds_min[3] = {};
        float bounds_max[3] = {};
        if (!compute_bounds(vertices, vertex_count, off_mesh_links, off_mesh_link_count, bounds_min, bounds_max, &error)) {
            set_create_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, error);
            return;
        }

        const float cell_size = quantization_cell(std::max(bounds_max[0] - bounds_min[0], bounds_max[2] - bounds_min[2]));
        const float cell_height = quantization_cell(bounds_max[1] - bounds_min[1]);
        bounds_min[0] -= cell_size;
        bounds_min[1] -= cell_height;
        bounds_min[2] -= cell_size;
        bounds_max[0] += cell_size;
        bounds_max[1] += cell_height;
        bounds_max[2] += cell_size;

        std::vector<unsigned short> detour_vertices = quantize_vertices(vertices, vertex_count, bounds_min, cell_size, cell_height);
        std::vector<unsigned short> detour_polys;
        std::vector<unsigned short> poly_flags;
        std::vector<unsigned char> poly_areas;
        if (!build_detour_polys(vertices, indices, index_count, vertex_count, polygons, polygon_count, &detour_polys, &poly_flags, &poly_areas, &error)) {
            set_create_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, error);
            return;
        }

        std::vector<float> off_mesh_vertices = copy_off_mesh_vertices(off_mesh_links, off_mesh_link_count);
        std::vector<float> off_mesh_radii;
        std::vector<unsigned short> off_mesh_flags;
        std::vector<unsigned char> off_mesh_areas;
        std::vector<unsigned char> off_mesh_directions;
        std::vector<unsigned int> off_mesh_user_ids;
        fill_off_mesh_arrays(
            off_mesh_links,
            off_mesh_link_count,
            &off_mesh_radii,
            &off_mesh_flags,
            &off_mesh_areas,
            &off_mesh_directions,
            &off_mesh_user_ids);

        dtNavMeshCreateParams params = {};
        params.verts = detour_vertices.data();
        params.vertCount = static_cast<int>(vertex_count);
        params.polys = detour_polys.data();
        params.polyFlags = poly_flags.data();
        params.polyAreas = poly_areas.data();
        params.polyCount = static_cast<int>(polygon_count);
        params.nvp = ZR_NAV_DETOUR_MAX_VERTS_PER_POLYGON;
        params.offMeshConVerts = off_mesh_vertices.empty() ? nullptr : off_mesh_vertices.data();
        params.offMeshConRad = off_mesh_radii.empty() ? nullptr : off_mesh_radii.data();
        params.offMeshConFlags = off_mesh_flags.empty() ? nullptr : off_mesh_flags.data();
        params.offMeshConAreas = off_mesh_areas.empty() ? nullptr : off_mesh_areas.data();
        params.offMeshConDir = off_mesh_directions.empty() ? nullptr : off_mesh_directions.data();
        params.offMeshConUserID = off_mesh_user_ids.empty() ? nullptr : off_mesh_user_ids.data();
        params.offMeshConCount = static_cast<int>(off_mesh_link_count);
        params.userId = 1;
        params.walkableHeight = 2.0f;
        params.walkableRadius = 0.5f;
        params.walkableClimb = 0.4f;
        params.cs = cell_size;
        params.ch = cell_height;
        params.buildBvTree = true;
        copy3(bounds_min, params.bmin);
        copy3(bounds_max, params.bmax);

        unsigned char* nav_data = nullptr;
        int nav_data_size = 0;
        if (!dtCreateNavMeshData(&params, &nav_data, &nav_data_size) || nav_data == nullptr || nav_data_size <= 0) {
            if (nav_data != nullptr) {
                dtFree(nav_data);
            }
            set_create_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, "Detour could not create navmesh tile data");
            return;
        }

        dtNavMesh* nav_mesh = dtAllocNavMesh();
        if (nav_mesh == nullptr) {
            dtFree(nav_data);
            set_create_status(out_result, ZR_NAV_DETOUR_ERROR, "Detour navmesh allocation failed");
            return;
        }
        dtStatus status = nav_mesh->init(nav_data, nav_data_size, DT_TILE_FREE_DATA);
        if (dtStatusFailed(status)) {
            dtFree(nav_data);
            dtFreeNavMesh(nav_mesh);
            set_create_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, "Detour navmesh initialization failed");
            return;
        }

        dtNavMeshQuery* nav_query = dtAllocNavMeshQuery();
        if (nav_query == nullptr) {
            dtFreeNavMesh(nav_mesh);
            set_create_status(out_result, ZR_NAV_DETOUR_ERROR, "Detour navmesh query allocation failed");
            return;
        }
        const int max_nodes = std::clamp(static_cast<int>(polygon_count) * 8 + 64, 64, 65535);
        status = nav_query->init(nav_mesh, max_nodes);
        if (dtStatusFailed(status)) {
            dtFreeNavMeshQuery(nav_query);
            dtFreeNavMesh(nav_mesh);
            set_create_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, "Detour navmesh query initialization failed");
            return;
        }

        ZrNavDetourQuery* query = new (std::nothrow) ZrNavDetourQuery();
        if (query == nullptr) {
            dtFreeNavMeshQuery(nav_query);
            dtFreeNavMesh(nav_mesh);
            set_create_status(out_result, ZR_NAV_DETOUR_ERROR, "Detour query owner allocation failed");
            return;
        }
        query->nav_mesh = nav_mesh;
        query->query = nav_query;
        query->polygon_count = polygon_count;
        copy3(bounds_min, query->bounds_min);
        copy3(bounds_max, query->bounds_max);
        query->query_extents[0] = std::max((bounds_max[0] - bounds_min[0]) * 0.5f + 1.0f, 1.0f);
        query->query_extents[1] = std::max((bounds_max[1] - bounds_min[1]) * 0.5f + 4.0f, 4.0f);
        query->query_extents[2] = std::max((bounds_max[2] - bounds_min[2]) * 0.5f + 1.0f, 1.0f);
        initialize_area_tables(query, area_costs, area_cost_count);

        out_result->query = query;
        out_result->polygon_count = polygon_count;
        set_create_status(out_result, ZR_NAV_DETOUR_OK, "Detour query created");
    } catch (const std::bad_alloc&) {
        set_create_status(out_result, ZR_NAV_DETOUR_ERROR, "Detour query allocation failed");
    } catch (const std::exception& error) {
        set_create_status(out_result, ZR_NAV_DETOUR_ERROR, error.what());
    } catch (...) {
        set_create_status(out_result, ZR_NAV_DETOUR_ERROR, "Detour query failed with an unknown native exception");
    }
}

extern "C" void zr_nav_detour_free_query(ZrNavDetourQuery* query) {
    if (query == nullptr) {
        return;
    }
    if (query->query != nullptr) {
        dtFreeNavMeshQuery(query->query);
    }
    if (query->nav_mesh != nullptr) {
        dtFreeNavMesh(query->nav_mesh);
    }
    delete query;
}

extern "C" void zr_nav_detour_find_path(
    const ZrNavDetourQuery* query,
    const float* start,
    const float* end,
    std::uint64_t area_mask,
    ZrNavDetourPathResult* out_result) {
    try {
        reset_path_result(out_result);
        if (out_result == nullptr) {
            return;
        }
        if (query == nullptr || query->query == nullptr || !finite3(start) || !finite3(end)) {
            set_path_status(out_result, ZR_NAV_DETOUR_ERROR, "Detour path query input is invalid");
            return;
        }
        ZrNavDetourFilter filter(*query, area_mask);
        dtPolyRef start_ref = 0;
        dtPolyRef end_ref = 0;
        float start_nearest[3] = {};
        float end_nearest[3] = {};
        if (!find_nearest_poly(query, start, query->query_extents, filter, &start_ref, start_nearest, nullptr)
            || !find_nearest_poly(query, end, query->query_extents, filter, &end_ref, end_nearest, nullptr)) {
            set_path_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, "Detour path query found no start or end polygon");
            return;
        }

        dtPolyRef corridor[ZR_NAV_DETOUR_MAX_PATH] = {};
        int corridor_count = 0;
        dtStatus status = query->query->findPath(
            start_ref,
            end_ref,
            start_nearest,
            end_nearest,
            &filter,
            corridor,
            &corridor_count,
            ZR_NAV_DETOUR_MAX_PATH);
        if (dtStatusFailed(status) || corridor_count == 0 || corridor[corridor_count - 1] != end_ref) {
            set_path_status(out_result, ZR_NAV_DETOUR_UNSUPPORTED_OR_NO_PATH, "Detour path query found no complete path");
            return;
        }

        float straight_path[ZR_NAV_DETOUR_MAX_STRAIGHT_PATH * 3] = {};
        unsigned char straight_flags[ZR_NAV_DETOUR_MAX_STRAIGHT_PATH] = {};
        dtPolyRef straight_refs[ZR_NAV_DETOUR_MAX_STRAIGHT_PATH] = {};
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
            ZR_NAV_DETOUR_MAX_STRAIGHT_PATH,
            DT_STRAIGHTPATH_AREA_CROSSINGS);
        if (dtStatusFailed(status) || straight_count == 0) {
            set_path_status(out_result, ZR_NAV_DETOUR_ERROR, "Detour straight path query failed");
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
            set_path_status(out_result, ZR_NAV_DETOUR_ERROR, "Detour path result allocation failed");
            return;
        }
        out_result->points = points;
        out_result->point_count = static_cast<std::uint32_t>(straight_count);
        out_result->visited_nodes = static_cast<std::uint32_t>(corridor_count);
        out_result->length = length;
        set_path_status(out_result, ZR_NAV_DETOUR_OK, "Detour path query completed");
    } catch (const std::bad_alloc&) {
        set_path_status(out_result, ZR_NAV_DETOUR_ERROR, "Detour path query allocation failed");
    } catch (const std::exception& error) {
        set_path_status(out_result, ZR_NAV_DETOUR_ERROR, error.what());
    } catch (...) {
        set_path_status(out_result, ZR_NAV_DETOUR_ERROR, "Detour path query failed with an unknown native exception");
    }
}

extern "C" void zr_nav_detour_free_path_result(ZrNavDetourPathResult* result) {
    if (result == nullptr) {
        return;
    }
    delete[] result->points;
    reset_path_result(result);
}

extern "C" void zr_nav_detour_sample_position(
    const ZrNavDetourQuery* query,
    const float* position,
    const float* extents,
    std::uint64_t area_mask,
    ZrNavDetourSampleResult* out_result) {
    try {
        reset_sample_result(out_result);
        if (out_result == nullptr) {
            return;
        }
        if (query == nullptr || query->query == nullptr || !finite3(position)) {
            set_sample_status(out_result, ZR_NAV_DETOUR_ERROR, "Detour sample query input is invalid");
            return;
        }
        float sanitized[3] = {};
        sanitized_extents(extents, sanitized);
        ZrNavDetourFilter filter(*query, area_mask);
        dtPolyRef poly_ref = 0;
        float nearest[3] = {};
        if (!find_nearest_poly(query, position, sanitized, filter, &poly_ref, nearest, nullptr)) {
            set_sample_status(out_result, ZR_NAV_DETOUR_OK, "Detour sample query found no polygon");
            return;
        }
        out_result->hit = 1;
        copy3(nearest, out_result->position);
        out_result->distance = distance3(position, nearest);
        out_result->area = area_for_ref(query, poly_ref, ZR_NAV_AREA_WALKABLE);
        set_sample_status(out_result, ZR_NAV_DETOUR_OK, "Detour sample query completed");
    } catch (const std::exception& error) {
        set_sample_status(out_result, ZR_NAV_DETOUR_ERROR, error.what());
    } catch (...) {
        set_sample_status(out_result, ZR_NAV_DETOUR_ERROR, "Detour sample query failed with an unknown native exception");
    }
}

extern "C" void zr_nav_detour_raycast(
    const ZrNavDetourQuery* query,
    const float* start,
    const float* end,
    std::uint64_t area_mask,
    ZrNavDetourRaycastResult* out_result) {
    try {
        reset_raycast_result(out_result);
        if (out_result == nullptr) {
            return;
        }
        if (query == nullptr || query->query == nullptr || !finite3(start) || !finite3(end)) {
            set_raycast_status(out_result, ZR_NAV_DETOUR_ERROR, "Detour raycast query input is invalid");
            return;
        }
        ZrNavDetourFilter filter(*query, area_mask);
        dtPolyRef start_ref = 0;
        float start_nearest[3] = {};
        if (!find_nearest_poly(query, start, query->query_extents, filter, &start_ref, start_nearest, nullptr)) {
            out_result->hit = 1;
            copy3(start, out_result->position);
            out_result->normal[0] = 0.0f;
            out_result->normal[1] = 1.0f;
            out_result->normal[2] = 0.0f;
            out_result->distance = 0.0f;
            set_raycast_status(out_result, ZR_NAV_DETOUR_OK, "Detour raycast starts outside navmesh");
            return;
        }

        dtPolyRef visited[ZR_NAV_DETOUR_MAX_PATH] = {};
        int visited_count = 0;
        float t = std::numeric_limits<float>::max();
        float normal[3] = { 0.0f, 1.0f, 0.0f };
        const dtStatus status = query->query->raycast(
            start_ref,
            start,
            end,
            &filter,
            &t,
            normal,
            visited,
            &visited_count,
            ZR_NAV_DETOUR_MAX_PATH);
        if (dtStatusFailed(status)) {
            set_raycast_status(out_result, ZR_NAV_DETOUR_ERROR, "Detour raycast query failed");
            return;
        }
        out_result->visited_nodes = static_cast<std::uint32_t>(std::max(visited_count, 0));
        if (t > 1.0f) {
            out_result->hit = 0;
            copy3(end, out_result->position);
            out_result->distance = distance3(start, end);
            set_raycast_status(out_result, ZR_NAV_DETOUR_OK, "Detour raycast completed without hit");
            return;
        }
        out_result->hit = 1;
        const float clamped_t = std::clamp(t, 0.0f, 1.0f);
        lerp3(start, end, clamped_t, out_result->position);
        copy3(normal, out_result->normal);
        out_result->distance = distance3(start, out_result->position);
        set_raycast_status(out_result, ZR_NAV_DETOUR_OK, "Detour raycast hit navmesh boundary");
    } catch (const std::exception& error) {
        set_raycast_status(out_result, ZR_NAV_DETOUR_ERROR, error.what());
    } catch (...) {
        set_raycast_status(out_result, ZR_NAV_DETOUR_ERROR, "Detour raycast failed with an unknown native exception");
    }
}
