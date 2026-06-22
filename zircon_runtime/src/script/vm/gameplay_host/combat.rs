use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::script::{ScriptHostCallContext, ScriptHostError, ScriptHostValue};
use crate::script::current_script_runtime_call_context;

use super::script_bindings::{
    apply_damage_to_script_health, apply_heal_to_script_health, script_binding_number,
    SCRIPT_BINDINGS_COMPONENT,
};
use super::values::{
    expect_bool, expect_entity, expect_float, expect_string, to_json_string, vec3_to_array,
};

pub(super) fn set_animation_bool(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let parameter = expect_string(context, 1)?;
    let value = expect_bool(context, 2)?;
    let runtime = current_script_runtime_call_context()?;
    let result = runtime.level.with_world_mut(|world| {
        let Some(mut player) = world.animation_state_machine_player(entity).cloned() else {
            return Ok(false);
        };
        player
            .parameters
            .insert(parameter, AnimationParameterValue::Bool(value));
        world
            .set_animation_state_machine_player(entity, Some(player))
            .map_err(|error| error.to_string())
            .map(|_| true)
    });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::new)
}

pub(super) fn damage_entity(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let damage = expect_float(context, 1)?.max(0.0) as f64;
    let runtime = current_script_runtime_call_context()?;
    let result = runtime.level.with_world_mut(|world| {
        let Some(mut bindings) = world
            .dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT)
            .cloned()
        else {
            return Ok(false);
        };
        let Some(remaining_health) = apply_damage_to_script_health(&mut bindings, damage) else {
            return Ok(false);
        };
        if remaining_health <= f64::EPSILON {
            Ok(world.remove_entity(entity))
        } else {
            world
                .set_dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT, bindings)
                .map_err(|error| error.to_string())
                .map(|_| true)
        }
    });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::new)
}

pub(super) fn heal_entity(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let amount = expect_float(context, 1)?.max(0.0) as f64;
    let max_health = expect_float(context, 2)?.max(0.0) as f64;
    let runtime = current_script_runtime_call_context()?;
    let result = runtime.level.with_world_mut(|world| {
        let Some(mut bindings) = world
            .dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT)
            .cloned()
        else {
            return Ok(false);
        };
        if apply_heal_to_script_health(&mut bindings, amount, max_health).is_none() {
            return Ok(false);
        }
        world
            .set_dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT, bindings)
            .map_err(|error| error.to_string())
            .map(|_| true)
    });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::new)
}

pub(super) fn current_hp(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let fallback = expect_float(context, 1)?;
    let runtime = current_script_runtime_call_context()?;
    let hp = runtime.level.with_world(|world| {
        world
            .dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT)
            .and_then(|bindings| script_binding_number(bindings, "hp"))
            .unwrap_or(f64::from(fallback))
    });
    Ok(ScriptHostValue::Float(hp))
}

pub(super) fn damage_entity_report(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let damage = expect_float(context, 1)?.max(0.0) as f64;
    let runtime = current_script_runtime_call_context()?;
    let report = runtime.level.with_world_mut(|world| {
        let position = world
            .world_transform(entity)
            .map(|transform| vec3_to_array(transform.translation));
        let Some(mut bindings) = world
            .dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT)
            .cloned()
        else {
            return Ok(DamageReport::miss(position));
        };
        let Some(remaining_health) = apply_damage_to_script_health(&mut bindings, damage) else {
            return Ok(DamageReport::miss(position));
        };
        let killed = remaining_health <= f64::EPSILON;
        if killed {
            world.remove_entity(entity);
        } else {
            world
                .set_dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT, bindings)
                .map_err(|error| error.to_string())?;
        }
        Ok(DamageReport {
            hit: true,
            killed,
            remaining_hp: remaining_health,
            position,
        })
    });
    report
        .and_then(|report| to_json_string(&report).map_err(|error| error.message))
        .map(ScriptHostValue::String)
        .map_err(ScriptHostError::new)
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
