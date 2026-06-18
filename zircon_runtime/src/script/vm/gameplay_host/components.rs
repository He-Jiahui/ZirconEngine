use crate::core::framework::script::{ScriptHostCallContext, ScriptHostError, ScriptHostValue};
use crate::script::current_script_runtime_call_context;

use super::script_bindings::{
    script_binding_number, script_binding_property_matches, SCRIPT_BINDINGS_COMPONENT,
};
use super::values::{expect_entity, expect_float, expect_string, json_error, to_json_string};

pub(super) fn component_json(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let component_id = expect_string(context, 1)?;
    let runtime = current_script_runtime_call_context()?;
    let component = runtime.level.with_world(|world| {
        world
            .dynamic_component(entity, &component_id)
            .cloned()
            .unwrap_or(serde_json::Value::Null)
    });
    Ok(ScriptHostValue::String(to_json_string(&component)?))
}

pub(super) fn component_string(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let component_id = expect_string(context, 1)?;
    let fallback = expect_string(context, 2)?;
    let runtime = current_script_runtime_call_context()?;
    let value = runtime.level.with_world(|world| {
        world
            .dynamic_component(entity, &component_id)
            .and_then(serde_json::Value::as_str)
            .unwrap_or(&fallback)
            .to_string()
    });
    Ok(ScriptHostValue::String(value))
}

pub(super) fn set_component_json(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let component_id = expect_string(context, 1)?;
    let component_json = expect_string(context, 2)?;
    let value = serde_json::from_str(&component_json).map_err(json_error)?;
    let runtime = current_script_runtime_call_context()?;
    let result = runtime
        .level
        .with_world_mut(|world| world.set_dynamic_component(entity, component_id, value));
    result
        .map(|_| ScriptHostValue::Bool(true))
        .map_err(ScriptHostError::new)
}

pub(super) fn find_by_component(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let component_id = expect_string(context, 0)?;
    let runtime = current_script_runtime_call_context()?;
    let entities = runtime.level.with_world(|world| {
        world
            .node_records()
            .into_iter()
            .filter(|node| world.dynamic_component(node.id, &component_id).is_some())
            .map(|node| node.id)
            .collect::<Vec<_>>()
    });
    Ok(ScriptHostValue::String(to_json_string(&entities)?))
}

pub(super) fn entity_exists(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let runtime = current_script_runtime_call_context()?;
    let exists = entity != 0
        && runtime.level.with_world(|world| {
            world
                .node_records()
                .into_iter()
                .any(|node| node.id == entity)
        });
    Ok(ScriptHostValue::Bool(exists))
}

pub(super) fn nearest_by_script_property(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let source_entity = expect_entity(context, 0)?;
    let property = expect_string(context, 1)?;
    let expected_value = expect_string(context, 2)?;
    let max_distance = expect_float(context, 3)?.max(0.0);
    let max_distance_squared = max_distance * max_distance;
    let runtime = current_script_runtime_call_context()?;

    let nearest = runtime.level.with_world(|world| {
        let source_position = world
            .world_transform(source_entity)
            .map(|transform| transform.translation)?;
        let mut nearest: Option<(u64, f32)> = None;

        for node in world.node_records() {
            if node.id == source_entity {
                continue;
            }
            let Some(bindings) = world.dynamic_component(node.id, SCRIPT_BINDINGS_COMPONENT) else {
                continue;
            };
            if !script_binding_property_matches(bindings, &property, &expected_value) {
                continue;
            }
            let Some(candidate_position) = world
                .world_transform(node.id)
                .map(|transform| transform.translation)
            else {
                continue;
            };
            let distance_squared = (candidate_position - source_position).length_squared();
            if distance_squared <= max_distance_squared
                && nearest
                    .map(|(_, best_distance)| distance_squared < best_distance)
                    .unwrap_or(true)
            {
                nearest = Some((node.id, distance_squared));
            }
        }

        nearest.map(|(entity, _)| entity)
    });

    Ok(ScriptHostValue::Int(nearest.unwrap_or(0) as i64))
}

pub(super) fn count_by_script_property(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let property = expect_string(context, 0)?;
    let expected_value = expect_string(context, 1)?;
    let runtime = current_script_runtime_call_context()?;
    let count = runtime.level.with_world(|world| {
        world
            .node_records()
            .into_iter()
            .filter(|node| {
                world
                    .dynamic_component(node.id, SCRIPT_BINDINGS_COMPONENT)
                    .is_some_and(|bindings| {
                        script_binding_property_matches(bindings, &property, &expected_value)
                    })
            })
            .count()
    });
    Ok(ScriptHostValue::Int(count as i64))
}

pub(super) fn script_property_matches(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let property = expect_string(context, 1)?;
    let expected_value = expect_string(context, 2)?;
    let runtime = current_script_runtime_call_context()?;
    let matches = runtime.level.with_world(|world| {
        world
            .dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT)
            .is_some_and(|bindings| {
                script_binding_property_matches(bindings, &property, &expected_value)
            })
    });
    Ok(ScriptHostValue::Bool(matches))
}

pub(super) fn script_number(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let property = expect_string(context, 1)?;
    let fallback = expect_float(context, 2)?;
    let runtime = current_script_runtime_call_context()?;
    let value = runtime.level.with_world(|world| {
        world
            .dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT)
            .and_then(|bindings| script_binding_number(bindings, &property))
            .unwrap_or(f64::from(fallback))
    });
    Ok(ScriptHostValue::Float(value))
}

pub(super) fn script_number_at_most(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let property = expect_string(context, 1)?;
    let threshold = expect_float(context, 2)?;
    let fallback = expect_float(context, 3)?;
    let runtime = current_script_runtime_call_context()?;
    let matches = runtime.level.with_world(|world| {
        let value = world
            .dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT)
            .and_then(|bindings| script_binding_number(bindings, &property))
            .unwrap_or(f64::from(fallback));
        value <= f64::from(threshold)
    });
    Ok(ScriptHostValue::Bool(matches))
}
