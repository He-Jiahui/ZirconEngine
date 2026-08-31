use crate::core::framework::animation::{AnimationParameterMap, AnimationParameterValue};
use crate::core::framework::script::{
    ScriptHostCallFrame, ScriptHostError, ScriptHostHotPathMetrics, ScriptHostValue,
};
use crate::script::runtime_context_for_frame;
use crate::script::vm::scene_system::script_binding_number_for_entity;

use super::error::GameplayHostResult;
use super::script_bindings::{
    apply_damage_to_script_health, apply_heal_to_script_health, SCRIPT_BINDINGS_COMPONENT,
};
use super::values::{
    expect_bool, expect_entity, expect_float, to_json_string, vec3_to_array, with_string,
};

fn set_animation_bool_parameter(
    parameters: &mut AnimationParameterMap,
    parameter: &str,
    value: bool,
) -> bool {
    let next = AnimationParameterValue::Bool(value);
    if let Some(current) = parameters.get_mut(parameter) {
        *current = next;
        return false;
    }
    parameters.insert(parameter.to_owned(), next);
    true
}

pub(super) fn set_animation_bool(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let value = expect_bool(context, 2)?;
    let runtime = runtime_context_for_frame(context)?;
    with_string(context, 1, |parameter: &str| {
        let result = runtime
            .level
            .with_world_mut(|world| -> GameplayHostResult<bool> {
                let Some(mut player) = world.animation_state_machine_player(entity).cloned() else {
                    return Ok(false);
                };
                if set_animation_bool_parameter(&mut player.parameters, parameter, value) {
                    ScriptHostHotPathMetrics::record_guest_string_copy(parameter.len());
                }
                world.set_animation_state_machine_player(entity, Some(player))?;
                Ok(true)
            });
        result
            .map(ScriptHostValue::Bool)
            .map_err(ScriptHostError::from)
    })
}

pub(super) fn damage_entity(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let damage = expect_float(context, 1)?.max(0.0) as f64;
    let runtime = runtime_context_for_frame(context)?;
    let result = runtime
        .level
        .with_world_mut(|world| -> GameplayHostResult<bool> {
            let Some(mut bindings) = world
                .dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT)
                .cloned()
            else {
                return Ok(false);
            };
            let Some(remaining_health) = apply_damage_to_script_health(&mut bindings, damage)
            else {
                return Ok(false);
            };
            if remaining_health <= f64::EPSILON {
                world.remove_entity(entity)?;
                Ok(true)
            } else {
                world.set_dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT, bindings)?;
                Ok(true)
            }
        });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::from)
}

pub(super) fn heal_entity(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let amount = expect_float(context, 1)?.max(0.0) as f64;
    let max_health = expect_float(context, 2)?.max(0.0) as f64;
    let runtime = runtime_context_for_frame(context)?;
    let result = runtime
        .level
        .with_world_mut(|world| -> GameplayHostResult<bool> {
            let Some(mut bindings) = world
                .dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT)
                .cloned()
            else {
                return Ok(false);
            };
            if apply_heal_to_script_health(&mut bindings, amount, max_health).is_none() {
                return Ok(false);
            }
            world.set_dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT, bindings)?;
            Ok(true)
        });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::from)
}

pub(super) fn current_hp(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let fallback = expect_float(context, 1)?;
    let runtime = runtime_context_for_frame(context)?;
    let hp = script_binding_number_for_entity(&runtime.level, entity, "hp")
        .map_err(|error| ScriptHostError::new(error.to_string()))?
        .unwrap_or(f64::from(fallback));
    Ok(ScriptHostValue::Float(hp))
}

pub(super) fn damage_entity_report(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let damage = expect_float(context, 1)?.max(0.0) as f64;
    let runtime = runtime_context_for_frame(context)?;
    let report = runtime
        .level
        .with_world_mut(|world| -> GameplayHostResult<DamageReport> {
            let position = world
                .world_transform(entity)
                .map(|transform| vec3_to_array(transform.translation));
            let Some(mut bindings) = world
                .dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT)
                .cloned()
            else {
                return Ok(DamageReport::miss(position));
            };
            let Some(remaining_health) = apply_damage_to_script_health(&mut bindings, damage)
            else {
                return Ok(DamageReport::miss(position));
            };
            let killed = remaining_health <= f64::EPSILON;
            if killed {
                world.remove_entity(entity)?;
            } else {
                world.set_dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT, bindings)?;
            }
            Ok(DamageReport {
                hit: true,
                killed,
                remaining_hp: remaining_health,
                position,
            })
        });
    Ok(ScriptHostValue::String(to_json_string(
        &report.map_err(ScriptHostError::from)?,
    )?))
}

#[derive(serde::Serialize)]
struct DamageReport {
    hit: bool,
    killed: bool,
    remaining_hp: f64,
    position: Option<[f32; 3]>,
}

impl DamageReport {
    fn miss(position: Option<[f32; 3]>) -> Self {
        Self {
            hit: false,
            killed: false,
            remaining_hp: 0.0,
            position,
        }
    }
}

#[cfg(test)]
mod performance_contract_tests {
    use crate::core::framework::animation::{AnimationParameterMap, AnimationParameterValue};

    use super::set_animation_bool_parameter;

    #[test]
    fn animation_bool_parameter_only_copies_a_missing_key() {
        let mut parameters = AnimationParameterMap::from([(
            "moving".to_owned(),
            AnimationParameterValue::Bool(false),
        )]);

        assert!(!set_animation_bool_parameter(
            &mut parameters,
            "moving",
            true
        ));
        assert!(set_animation_bool_parameter(
            &mut parameters,
            "grounded",
            true
        ));
        assert_eq!(
            parameters.get("moving"),
            Some(&AnimationParameterValue::Bool(true))
        );
        assert_eq!(
            parameters.get("grounded"),
            Some(&AnimationParameterValue::Bool(true))
        );
    }
}
