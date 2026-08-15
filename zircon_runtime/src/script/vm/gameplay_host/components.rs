use crate::core::framework::script::{
    ScriptHostCallFrame, ScriptHostError, ScriptHostHotPathMetrics, ScriptHostValue,
};
use crate::script::runtime_context_for_frame;
use crate::script::vm::scene_system::{
    script_binding_number_for_entity, with_script_binding_property_matches,
};

use super::values::{expect_entity, expect_float, json_error, to_json_string, with_string};

pub(super) fn component_json(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let runtime = runtime_context_for_frame(context)?;
    with_string(context, 1, |component_id: &str| {
        let component = runtime.level.with_world(|world| {
            world
                .dynamic_component(entity, component_id)
                .cloned()
                .unwrap_or(serde_json::Value::Null)
        });
        Ok(ScriptHostValue::String(to_json_string(&component)?))
    })
}

pub(super) fn component_string(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let runtime = runtime_context_for_frame(context)?;
    with_string(context, 1, |component_id: &str| {
        with_string(context, 2, |fallback: &str| {
            let value = runtime.level.with_world(|world| {
                world
                    .dynamic_component(entity, component_id)
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(fallback)
                    .to_owned()
            });
            Ok(ScriptHostValue::String(value))
        })
    })
}

pub(super) fn set_component_json(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let runtime = runtime_context_for_frame(context)?;
    with_string(context, 1, |component_id: &str| {
        with_string(context, 2, |component_json: &str| {
            let value = serde_json::from_str(component_json).map_err(json_error)?;
            ScriptHostHotPathMetrics::record_guest_string_copy(component_id.len());
            let result = runtime
                .level
                .with_world_mut(|world| world.set_dynamic_component(entity, component_id, value));
            result
                .map(|_| ScriptHostValue::Bool(true))
                .map_err(|error| ScriptHostError::new(error.to_string()))
        })
    })
}

pub(super) fn find_by_component(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = runtime_context_for_frame(context)?;
    with_string(context, 0, |component_id: &str| {
        let entities = runtime.level.with_world(|world| {
            let mut rows = Vec::new();
            world.dynamic_component_rows(component_id, &mut rows);
            rows.into_iter()
                .map(|(entity, _)| entity)
                .collect::<Vec<_>>()
        });
        Ok(ScriptHostValue::String(to_json_string(&entities)?))
    })
}

pub(super) fn entity_exists(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let runtime = runtime_context_for_frame(context)?;
    let exists = entity != 0
        && runtime
            .level
            .with_world(|world| world.contains_entity(entity));
    Ok(ScriptHostValue::Bool(exists))
}

pub(super) fn nearest_by_script_property(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let source_entity = expect_entity(context, 0)?;
    let max_distance = expect_float(context, 3)?.max(0.0);
    let max_distance_squared = max_distance * max_distance;
    let runtime = runtime_context_for_frame(context)?;

    with_string(context, 1, |property: &str| {
        with_string(context, 2, |expected_value: &str| {
            let nearest = with_script_binding_property_matches(
                &runtime.level,
                property,
                expected_value,
                |entities, world| {
                    let source_position = world
                        .world_transform(source_entity)
                        .map(|transform| transform.translation)?;
                    let mut nearest: Option<(u64, f32)> = None;

                    for entity in entities {
                        if *entity == source_entity {
                            continue;
                        }
                        let Some(candidate_position) = world
                            .world_transform(*entity)
                            .map(|transform| transform.translation)
                        else {
                            continue;
                        };
                        let distance_squared =
                            (candidate_position - source_position).length_squared();
                        if distance_squared <= max_distance_squared
                            && nearest
                                .map(|(_, best_distance)| distance_squared < best_distance)
                                .unwrap_or(true)
                        {
                            nearest = Some((*entity, distance_squared));
                        }
                    }

                    nearest.map(|(entity, _)| entity)
                },
            )
            .map_err(|error| ScriptHostError::new(error.to_string()))?;

            Ok(ScriptHostValue::Int(nearest.unwrap_or(0) as i64))
        })
    })
}

pub(super) fn count_by_script_property(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = runtime_context_for_frame(context)?;
    with_string(context, 0, |property: &str| {
        with_string(context, 1, |expected_value: &str| {
            let count = with_script_binding_property_matches(
                &runtime.level,
                property,
                expected_value,
                |entities, _| entities.len(),
            )
            .map_err(|error| ScriptHostError::new(error.to_string()))?;
            Ok(ScriptHostValue::Int(count as i64))
        })
    })
}

pub(super) fn script_property_matches(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let runtime = runtime_context_for_frame(context)?;
    with_string(context, 1, |property: &str| {
        with_string(context, 2, |expected_value: &str| {
            let matches = with_script_binding_property_matches(
                &runtime.level,
                property,
                expected_value,
                |entities, _| entities.binary_search(&entity).is_ok(),
            )
            .map_err(|error| ScriptHostError::new(error.to_string()))?;
            Ok(ScriptHostValue::Bool(matches))
        })
    })
}

pub(super) fn script_number(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let fallback = expect_float(context, 2)?;
    let runtime = runtime_context_for_frame(context)?;
    with_string(context, 1, |property: &str| {
        let value = script_binding_number_for_entity(&runtime.level, entity, property)
            .map_err(|error| ScriptHostError::new(error.to_string()))?
            .unwrap_or(f64::from(fallback));
        Ok(ScriptHostValue::Float(value))
    })
}

pub(super) fn script_number_at_most(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let threshold = expect_float(context, 2)?;
    let fallback = expect_float(context, 3)?;
    let runtime = runtime_context_for_frame(context)?;
    with_string(context, 1, |property: &str| {
        let value = script_binding_number_for_entity(&runtime.level, entity, property)
            .map_err(|error| ScriptHostError::new(error.to_string()))?
            .unwrap_or(f64::from(fallback));
        let matches = value <= f64::from(threshold);
        Ok(ScriptHostValue::Bool(matches))
    })
}

#[cfg(test)]
mod performance_contract_tests {
    #[test]
    fn entity_exists_uses_the_world_entity_index() {
        let source = include_str!("components.rs")
            .split_once("#[cfg(test)]")
            .unwrap()
            .0;
        let function = source.split("pub(super) fn entity_exists").nth(1).unwrap();
        let function = function.split("pub(super) fn").next().unwrap();

        assert!(function.contains("world.contains_entity(entity)"));
        assert!(!function.contains("node_records()"));
    }
}
