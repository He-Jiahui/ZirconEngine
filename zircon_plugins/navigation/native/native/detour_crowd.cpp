#include "recast_bridge.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstring>
#include <exception>
#include <memory>
#include <new>

#include "DetourCrowd.h"
#include "DetourNavMesh.h"
#include "DetourNavMeshQuery.h"
#include "DetourStatus.h"

struct ZrNavCrowd {
    ZrNavDetourQuery* query_owner = nullptr;
    dtCrowd* crowd = nullptr;
    std::uint32_t capacity = 0;
    std::array<std::uint64_t, DT_CROWD_MAX_QUERY_FILTER_TYPE> filter_masks = {};
    std::array<bool, DT_CROWD_MAX_QUERY_FILTER_TYPE> filter_in_use = {};
    std::array<float, DT_MAX_AREAS> area_costs = {};
    std::uint64_t walkable_area_mask = 0;

    ~ZrNavCrowd() {
        if (crowd != nullptr) {
            dtFreeCrowd(crowd);
        }
        if (query_owner != nullptr) {
            zr_nav_detour_free_query(query_owner);
        }
    }
};

namespace {

constexpr std::uint32_t ZR_NAV_CROWD_ERROR = 0;
constexpr std::uint32_t ZR_NAV_CROWD_OK = 1;
constexpr std::uint8_t ZR_NAV_CROWD_MAX_AVOIDANCE_QUALITY = 3;

void set_message(char* output, std::size_t output_size, const char* message) {
    if (output == nullptr || output_size == 0) {
        return;
    }
    std::memset(output, 0, output_size);
    if (message != nullptr) {
        std::strncpy(output, message, output_size - 1);
        output[output_size - 1] = '\0';
    }
}

void reset_create_result(ZrNavCrowdCreateResult* result) {
    if (result == nullptr) {
        return;
    }
    result->status = ZR_NAV_CROWD_ERROR;
    set_message(result->message, sizeof(result->message), "");
    result->crowd = nullptr;
    result->capacity = 0;
}

void reset_command_result(ZrNavCrowdCommandResult* result) {
    if (result == nullptr) {
        return;
    }
    result->status = ZR_NAV_CROWD_ERROR;
    set_message(result->message, sizeof(result->message), "");
    result->agent_id = 0;
    result->state_count = 0;
}

void succeed(ZrNavCrowdCommandResult* result, const char* message) {
    result->status = ZR_NAV_CROWD_OK;
    set_message(result->message, sizeof(result->message), message);
}

bool finite3(const float* value) {
    return value != nullptr
        && std::isfinite(value[0])
        && std::isfinite(value[1])
        && std::isfinite(value[2]);
}

bool valid_crowd(const ZrNavCrowd* crowd) {
    return crowd != nullptr && crowd->crowd != nullptr && crowd->query_owner != nullptr;
}

bool valid_agent(const ZrNavCrowd* crowd, std::uint32_t agent_id) {
    return valid_crowd(crowd) && agent_id < crowd->capacity;
}

void configure_avoidance_profiles(dtCrowd* crowd) {
    dtObstacleAvoidanceParams params = *crowd->getObstacleAvoidanceParams(0);
    params.velBias = 0.5f;
    params.adaptiveDivs = 5;
    params.adaptiveRings = 2;
    params.adaptiveDepth = 1;
    crowd->setObstacleAvoidanceParams(1, &params);

    params.adaptiveDepth = 2;
    crowd->setObstacleAvoidanceParams(2, &params);

    params.adaptiveDivs = 7;
    params.adaptiveRings = 3;
    params.adaptiveDepth = 3;
    crowd->setObstacleAvoidanceParams(3, &params);
}

int query_filter_for(ZrNavCrowd* owner, std::uint64_t requested_mask) {
    const std::uint64_t effective_mask = requested_mask & owner->walkable_area_mask;
    if (effective_mask == 0) {
        return -1;
    }
    for (int index = 0; index < DT_CROWD_MAX_QUERY_FILTER_TYPE; ++index) {
        if (owner->filter_in_use[index] && owner->filter_masks[index] == effective_mask) {
            return index;
        }
    }
    for (int index = 0; index < DT_CROWD_MAX_QUERY_FILTER_TYPE; ++index) {
        bool active_agent_uses_filter = false;
        if (owner->filter_in_use[index]) {
            for (std::uint32_t agent_id = 0; agent_id < owner->capacity; ++agent_id) {
                const dtCrowdAgent* agent = owner->crowd->getAgent(static_cast<int>(agent_id));
                if (agent != nullptr && agent->active && agent->params.queryFilterType == index) {
                    active_agent_uses_filter = true;
                    break;
                }
            }
        }
        if (active_agent_uses_filter) {
            continue;
        }
        dtQueryFilter* filter = owner->crowd->getEditableFilter(index);
        if (filter == nullptr) {
            return -1;
        }
        filter->setAreaMask(effective_mask);
        for (int area = 0; area < DT_MAX_AREAS; ++area) {
            filter->setAreaCost(area, owner->area_costs[area]);
        }
        owner->filter_in_use[index] = true;
        owner->filter_masks[index] = effective_mask;
        return index;
    }
    return -1;
}

dtCrowdAgentParams detour_agent_params(const ZrNavCrowdAgentParams& source, int query_filter_type) {
    dtCrowdAgentParams params = {};
    params.radius = std::max(source.radius, 0.01f);
    params.height = std::max(source.height, 0.01f);
    params.maxAcceleration = std::max(source.max_acceleration, 0.0f);
    params.maxSpeed = std::max(source.max_speed, 0.0f);
    params.collisionQueryRange = std::max(source.collision_query_range, params.radius * 2.0f);
    params.pathOptimizationRange = std::max(source.path_optimization_range, params.radius * 2.0f);
    params.separationWeight = std::max(source.separation_weight, 0.0f)
        * (1.0f + (255.0f - static_cast<float>(source.avoidance_priority)) / 255.0f);
    params.updateFlags = DT_CROWD_ANTICIPATE_TURNS | DT_CROWD_OPTIMIZE_VIS | DT_CROWD_OPTIMIZE_TOPO;
    if (source.avoidance_quality > 0) {
        params.updateFlags |= DT_CROWD_OBSTACLE_AVOIDANCE | DT_CROWD_SEPARATION;
    }
    params.obstacleAvoidanceType = std::min(source.avoidance_quality, ZR_NAV_CROWD_MAX_AVOIDANCE_QUALITY);
    params.queryFilterType = static_cast<unsigned char>(query_filter_type);
    params.userData = nullptr;
    return params;
}

} // namespace

extern "C" void zr_nav_crowd_create(
    ZrNavDetourQuery* query_owner,
    std::uint32_t max_agents,
    float max_agent_radius,
    const ZrNavDetourAreaCost* area_costs,
    std::uint32_t area_cost_count,
    ZrNavCrowdCreateResult* out_result) {
    try {
        reset_create_result(out_result);
        if (out_result == nullptr) {
            return;
        }
        auto* nav_mesh = static_cast<dtNavMesh*>(zr_nav_detour_query_nav_mesh(query_owner));
        if (query_owner == nullptr || nav_mesh == nullptr || max_agents == 0
            || !std::isfinite(max_agent_radius) || max_agent_radius <= 0.0f
            || (area_cost_count > 0 && area_costs == nullptr)) {
            set_message(out_result->message, sizeof(out_result->message), "Crowd create input is invalid");
            return;
        }

        std::unique_ptr<ZrNavCrowd> owner(new (std::nothrow) ZrNavCrowd());
        if (owner == nullptr) {
            set_message(out_result->message, sizeof(out_result->message), "Crowd owner allocation failed");
            return;
        }
        owner->crowd = dtAllocCrowd();
        if (owner->crowd == nullptr
            || !owner->crowd->init(static_cast<int>(max_agents), max_agent_radius, nav_mesh)) {
            set_message(out_result->message, sizeof(out_result->message), "DetourCrowd initialization failed");
            return;
        }
        owner->capacity = max_agents;
        owner->area_costs.fill(1.0f);
        for (std::uint32_t index = 0; index < area_cost_count; ++index) {
            const auto& area = area_costs[index];
            if (area.area >= DT_MAX_AREAS || !std::isfinite(area.cost) || area.cost <= 0.0f) {
                continue;
            }
            owner->area_costs[area.area] = area.cost;
            if (area.walkable != 0) {
                owner->walkable_area_mask |= (1ULL << area.area);
            }
        }
        if (area_cost_count == 0) {
            owner->walkable_area_mask = ~0ULL;
        }
        configure_avoidance_profiles(owner->crowd);
        owner->query_owner = query_owner;
        out_result->status = ZR_NAV_CROWD_OK;
        out_result->crowd = owner.release();
        out_result->capacity = max_agents;
        set_message(out_result->message, sizeof(out_result->message), "DetourCrowd created");
    } catch (const std::exception& error) {
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, error.what());
    } catch (...) {
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, "Crowd create failed with an unknown native exception");
    }
}

extern "C" void zr_nav_crowd_free(ZrNavCrowd* crowd) {
    delete crowd;
}

extern "C" void zr_nav_crowd_add_agent(
    ZrNavCrowd* crowd,
    const float* position,
    const ZrNavCrowdAgentParams* params,
    ZrNavCrowdCommandResult* out_result) {
    try {
    reset_command_result(out_result);
    if (out_result == nullptr || !valid_crowd(crowd) || !finite3(position) || params == nullptr) {
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, "Crowd add-agent input is invalid");
        return;
    }
    const int filter_type = query_filter_for(crowd, params->area_mask);
    if (filter_type < 0) {
        set_message(out_result->message, sizeof(out_result->message), "Crowd area mask is empty or query-filter capacity is exhausted");
        return;
    }
    const dtCrowdAgentParams native_params = detour_agent_params(*params, filter_type);
    const int agent_id = crowd->crowd->addAgent(position, &native_params);
    if (agent_id < 0) {
        set_message(out_result->message, sizeof(out_result->message), "DetourCrowd agent pool is full or position is off mesh");
        return;
    }
    out_result->agent_id = static_cast<std::uint32_t>(agent_id);
    succeed(out_result, "Crowd agent added");
    } catch (const std::exception& error) {
        reset_command_result(out_result);
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, error.what());
    } catch (...) {
        reset_command_result(out_result);
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, "Crowd add-agent failed with an unknown native exception");
    }
}

extern "C" void zr_nav_crowd_remove_agent(
    ZrNavCrowd* crowd,
    std::uint32_t agent_id,
    ZrNavCrowdCommandResult* out_result) {
    try {
    reset_command_result(out_result);
    if (out_result == nullptr || !valid_agent(crowd, agent_id)) {
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, "Crowd remove-agent input is invalid");
        return;
    }
    crowd->crowd->removeAgent(static_cast<int>(agent_id));
    out_result->agent_id = agent_id;
    succeed(out_result, "Crowd agent removed");
    } catch (const std::exception& error) {
        reset_command_result(out_result);
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, error.what());
    } catch (...) {
        reset_command_result(out_result);
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, "Crowd remove-agent failed with an unknown native exception");
    }
}

extern "C" void zr_nav_crowd_set_target(
    ZrNavCrowd* crowd,
    std::uint32_t agent_id,
    const float* target,
    ZrNavCrowdCommandResult* out_result) {
    try {
    reset_command_result(out_result);
    if (out_result == nullptr || !valid_agent(crowd, agent_id)) {
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, "Crowd target input is invalid");
        return;
    }
    const dtCrowdAgent* agent = crowd->crowd->getAgent(static_cast<int>(agent_id));
    if (agent == nullptr || !agent->active) {
        set_message(out_result->message, sizeof(out_result->message), "Crowd agent is inactive");
        return;
    }
    if (target == nullptr) {
        if (!crowd->crowd->resetMoveTarget(static_cast<int>(agent_id))) {
            set_message(out_result->message, sizeof(out_result->message), "Crowd target could not be reset");
            return;
        }
        out_result->agent_id = agent_id;
        succeed(out_result, "Crowd target reset");
        return;
    }
    if (!finite3(target)) {
        set_message(out_result->message, sizeof(out_result->message), "Crowd target is invalid");
        return;
    }
    dtPolyRef target_ref = 0;
    float nearest[3] = {};
    const dtStatus status = crowd->crowd->getNavMeshQuery()->findNearestPoly(
        target,
        crowd->crowd->getQueryHalfExtents(),
        crowd->crowd->getFilter(agent->params.queryFilterType),
        &target_ref,
        nearest);
    if (dtStatusFailed(status) || target_ref == 0
        || !crowd->crowd->requestMoveTarget(static_cast<int>(agent_id), target_ref, nearest)) {
        set_message(out_result->message, sizeof(out_result->message), "Crowd target could not be projected onto navmesh");
        return;
    }
    out_result->agent_id = agent_id;
    succeed(out_result, "Crowd target accepted");
    } catch (const std::exception& error) {
        reset_command_result(out_result);
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, error.what());
    } catch (...) {
        reset_command_result(out_result);
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, "Crowd target failed with an unknown native exception");
    }
}

extern "C" void zr_nav_crowd_sync_agent_position(
    ZrNavCrowd* crowd,
    std::uint32_t agent_id,
    const float* position,
    ZrNavCrowdCommandResult* out_result) {
    try {
    reset_command_result(out_result);
    if (out_result == nullptr || !valid_agent(crowd, agent_id) || !finite3(position)) {
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, "Crowd position-sync input is invalid");
        return;
    }
    dtCrowdAgent* agent = crowd->crowd->getEditableAgent(static_cast<int>(agent_id));
    if (agent == nullptr || !agent->active) {
        set_message(out_result->message, sizeof(out_result->message), "Crowd agent is inactive");
        return;
    }
    dtNavMeshQuery* query = const_cast<dtNavMeshQuery*>(crowd->crowd->getNavMeshQuery());
    const dtQueryFilter* filter = crowd->crowd->getFilter(agent->params.queryFilterType);
    if (query == nullptr || filter == nullptr || !agent->corridor.movePosition(position, query, filter)) {
        set_message(out_result->message, sizeof(out_result->message), "Crowd position could not be synchronized on the current corridor");
        return;
    }
    std::memcpy(agent->npos, agent->corridor.getPos(), sizeof(agent->npos));
    agent->boundary.reset();
    out_result->agent_id = agent_id;
    succeed(out_result, "Crowd position synchronized");
    } catch (const std::exception& error) {
        reset_command_result(out_result);
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, error.what());
    } catch (...) {
        reset_command_result(out_result);
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, "Crowd position sync failed with an unknown native exception");
    }
}

extern "C" void zr_nav_crowd_update(
    ZrNavCrowd* crowd,
    float dt_seconds,
    ZrNavCrowdCommandResult* out_result) {
    try {
    reset_command_result(out_result);
    if (out_result == nullptr || !valid_crowd(crowd)
        || !std::isfinite(dt_seconds) || dt_seconds <= 0.0f) {
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, "Crowd update input is invalid");
        return;
    }
    crowd->crowd->update(dt_seconds, nullptr);
    succeed(out_result, "Crowd updated");
    } catch (const std::exception& error) {
        reset_command_result(out_result);
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, error.what());
    } catch (...) {
        reset_command_result(out_result);
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, "Crowd update failed with an unknown native exception");
    }
}

extern "C" void zr_nav_crowd_read_states(
    const ZrNavCrowd* crowd,
    ZrNavCrowdAgentState* states,
    std::uint32_t state_capacity,
    ZrNavCrowdCommandResult* out_result) {
    try {
    reset_command_result(out_result);
    if (out_result == nullptr || !valid_crowd(crowd)
        || (state_capacity > 0 && states == nullptr)) {
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, "Crowd read-states input is invalid");
        return;
    }
    std::uint32_t count = 0;
    for (std::uint32_t agent_id = 0; agent_id < crowd->capacity && count < state_capacity; ++agent_id) {
        const dtCrowdAgent* agent = crowd->crowd->getAgent(static_cast<int>(agent_id));
        if (agent == nullptr || !agent->active) {
            continue;
        }
        ZrNavCrowdAgentState& state = states[count++];
        state.agent_id = agent_id;
        state.active = 1;
        state.traversal_state = agent->state;
        state.target_state = agent->targetState;
        state.partial_path = agent->partial ? 1 : 0;
        std::memcpy(state.position, agent->npos, sizeof(state.position));
        std::memcpy(state.desired_velocity, agent->dvel, sizeof(state.desired_velocity));
        std::memcpy(state.avoidance_velocity, agent->nvel, sizeof(state.avoidance_velocity));
        std::memcpy(state.velocity, agent->vel, sizeof(state.velocity));
    }
    out_result->state_count = count;
    succeed(out_result, "Crowd states read");
    } catch (const std::exception& error) {
        reset_command_result(out_result);
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, error.what());
    } catch (...) {
        reset_command_result(out_result);
        set_message(out_result == nullptr ? nullptr : out_result->message, 256, "Crowd read-states failed with an unknown native exception");
    }
}
