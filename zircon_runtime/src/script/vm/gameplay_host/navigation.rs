use serde::Deserialize;

use crate::core::framework::navigation::{
    NavMeshAgentDescriptor, NavPathQuery, NavPathStatus, DEFAULT_AGENT_TYPE, DEFAULT_AREA_MASK,
    NAV_MESH_AGENT_COMPONENT_TYPE,
};
use crate::core::framework::script::{ScriptHostCallFrame, ScriptHostError, ScriptHostValue};
use crate::core::manager::{navigation_manager_handle, resolve_manager_service};
use crate::core::math::Vec3;
use crate::scene::{
    SceneNavigationRuntime, SceneNavigationRuntimeHandle, SCENE_NAVIGATION_RUNTIME_DRIVER_NAME,
};
use crate::script::runtime_context_for_frame;

use super::error::{GameplayHostError, GameplayHostResult};
use super::values::{
    expect_entity, expect_float, expect_vec3_json, script_core_error, to_json_string, vec3_to_array,
};

pub(super) fn nav_next_point_json(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let start = expect_vec3_json(context, 0)?;
    let end = expect_vec3_json(context, 1)?;
    let runtime = runtime_context_for_frame(context)?;
    let core = runtime.core_handle()?;
    let navigation = navigation_manager_handle(&core)
        .and_then(|handle| resolve_manager_service(&core, handle))
        .map_err(script_core_error)?;
    let result = navigation
        .find_path(NavPathQuery {
            nav_mesh: None,
            start: vec3_to_array(start),
            end: vec3_to_array(end),
            agent_type: DEFAULT_AGENT_TYPE.to_string(),
            area_mask: DEFAULT_AREA_MASK,
        })
        .map_err(GameplayHostError::from)?;
    if matches!(result.status, NavPathStatus::NoPath) || result.points.is_empty() {
        return Ok(ScriptHostValue::String("null".to_string()));
    }
    let point = result
        .points
        .get(1)
        .or_else(|| result.points.first())
        .map(|point| point.position);
    Ok(ScriptHostValue::String(to_json_string(&point)?))
}

pub(super) fn nav_move_towards_entity(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let target_entity = expect_entity(context, 1)?;
    let speed = expect_float(context, 2)?;
    let dt = expect_float(context, 3)?;
    move_entity_with_navigation(context, entity, target_entity, speed, dt)
}

pub(super) fn move_entity_with_navigation(
    context: &ScriptHostCallFrame<'_>,
    entity: u64,
    target_entity: u64,
    speed: f32,
    dt: f32,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = runtime_context_for_frame(context)?;
    let core = runtime.core_handle()?;
    let navigation = core
        .resolve_driver::<SceneNavigationRuntimeHandle>(SCENE_NAVIGATION_RUNTIME_DRIVER_NAME)
        .map_err(script_core_error)?;
    let result = runtime
        .level
        .with_world_mut(|world| -> GameplayHostResult<_> {
            let target = world
                .world_transform(target_entity)
                .map(|transform| transform.translation)
                .ok_or_else(|| {
                    GameplayHostError::missing_entity("navigation target", target_entity)
                })?;
            if world.world_transform(entity).is_none() {
                return Err(GameplayHostError::missing_entity(
                    "navigation source",
                    entity,
                ));
            }
            let mut agent = world
                .dynamic_component(entity, NAV_MESH_AGENT_COMPONENT_TYPE)
                .and_then(|value| NavMeshAgentDescriptor::deserialize(value).ok())
                .unwrap_or_else(NavMeshAgentDescriptor::default);
            agent.agent_type = DEFAULT_AGENT_TYPE.to_string();
            agent.speed = speed.max(0.0);
            agent.acceleration = agent.acceleration.max(agent.speed * 3.0).max(8.0);
            agent.radius = agent.radius.max(0.38);
            agent.height = agent.height.max(1.4);
            agent.stopping_distance = agent.stopping_distance.max(0.92);
            agent.avoidance_quality = crate::core::framework::navigation::NavAvoidanceQuality::High;
            agent.area_mask = DEFAULT_AREA_MASK;
            agent.update_position = true;
            agent.update_rotation = true;
            agent.destination = Some(target.to_array());
            let value = serde_json::to_value(agent)?;
            world.set_dynamic_component(entity, NAV_MESH_AGENT_COMPONENT_TYPE, value)?;
            Ok(navigation.tick_world_agent(world, entity, dt)?)
        });
    result
        .map(|report| ScriptHostValue::Bool(report.moved_agents > 0))
        .map_err(ScriptHostError::from)
}

pub(super) fn navigation_next_point(
    runtime: &crate::script::ScriptRuntimeCallContext,
    start: Vec3,
    target: Vec3,
) -> Option<Vec3> {
    let core = runtime.core_handle().ok()?;
    let navigation = navigation_manager_handle(&core)
        .and_then(|handle| resolve_manager_service(&core, handle))
        .ok()?;
    let result = navigation
        .find_path(NavPathQuery {
            nav_mesh: None,
            start: vec3_to_array(start),
            end: vec3_to_array(target),
            agent_type: DEFAULT_AGENT_TYPE.to_string(),
            area_mask: DEFAULT_AREA_MASK,
        })
        .ok()?;
    if matches!(result.status, NavPathStatus::NoPath) {
        return None;
    }
    result
        .points
        .get(1)
        .or_else(|| result.points.first())
        .map(|point| Vec3::new(point.position[0], point.position[1], point.position[2]))
}

#[cfg(test)]
mod performance_contract_tests {
    use serde::Deserialize;

    use super::NavMeshAgentDescriptor;

    #[test]
    fn borrowed_navigation_agent_deserialization_preserves_descriptor() {
        let expected = NavMeshAgentDescriptor::default();
        let value = serde_json::to_value(&expected).unwrap();

        let actual = NavMeshAgentDescriptor::deserialize(&value).unwrap();

        assert_eq!(serde_json::to_value(actual).unwrap(), value);
    }
}
