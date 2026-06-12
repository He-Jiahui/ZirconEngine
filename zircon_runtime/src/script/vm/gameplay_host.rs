use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::input::InputButton;
use crate::core::framework::navigation::{
    NavMeshAgentDescriptor, NavPathQuery, NavPathStatus, DEFAULT_AGENT_TYPE, DEFAULT_AREA_MASK,
    NAV_MESH_AGENT_COMPONENT_TYPE,
};
use crate::core::framework::script::{
    ScriptHostCallContext, ScriptHostError, ScriptHostFunctionDescriptor,
    ScriptHostModuleDescriptor, ScriptHostParameterDescriptor, ScriptHostValue,
    ScriptHostValueKind,
};
use crate::core::manager::{resolve_input_manager, resolve_navigation_manager};
use crate::core::math::{Quat, Transform, Vec3};
use crate::core::resource::{AssetUuid, MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::scene::components::{MeshRenderer, Name, NodeKind};
use crate::script::{
    current_script_runtime_call_context, script_float, HostExportFunction, HostExportRegistry,
    HostHandle, VmError,
};

mod script_bindings;

use script_bindings::{
    apply_damage_to_script_health, apply_heal_to_script_health, script_binding_number,
    script_binding_property_matches, SCRIPT_BINDINGS_COMPONENT,
};

const GAMEPLAY_MODULE: &str = "zr.zircon.gameplay";
const GAMEPLAY_MODULE_VERSION: &str = "0.1.0";

pub fn register_gameplay_host_module(
    exports: &HostExportRegistry,
) -> Result<Option<HostHandle>, VmError> {
    if exports.module(GAMEPLAY_MODULE).is_some() {
        return Ok(None);
    }

    let descriptor = ScriptHostModuleDescriptor::new(GAMEPLAY_MODULE, GAMEPLAY_MODULE_VERSION)
        .with_capability("gameplay.input")
        .with_capability("gameplay.entity")
        .with_capability("gameplay.navigation")
        .with_function(function("delta_seconds", 0, 0, ScriptHostValueKind::Float))
        .with_function(
            function("entity", 0, 0, ScriptHostValueKind::Int)
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("key_pressed", 1, 1, ScriptHostValueKind::Bool)
                .with_parameter(string_parameter("key"))
                .with_required_capability("gameplay.input"),
        )
        .with_function(
            function("position_json", 1, 1, ScriptHostValueKind::String)
                .with_parameter(int_parameter("entity"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("position_x", 1, 1, ScriptHostValueKind::Float)
                .with_parameter(int_parameter("entity"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("position_y", 1, 1, ScriptHostValueKind::Float)
                .with_parameter(int_parameter("entity"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("position_z", 1, 1, ScriptHostValueKind::Float)
                .with_parameter(int_parameter("entity"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("set_position_json", 2, 2, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(string_parameter("position_json"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("set_position", 4, 4, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(float_parameter("x"))
                .with_parameter(float_parameter("y"))
                .with_parameter(float_parameter("z"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("translate_json", 2, 2, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(string_parameter("delta_json"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("translate", 4, 4, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(float_parameter("x"))
                .with_parameter(float_parameter("y"))
                .with_parameter(float_parameter("z"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("face_direction", 3, 3, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(float_parameter("x"))
                .with_parameter(float_parameter("z"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("set_scale", 4, 4, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(float_parameter("x"))
                .with_parameter(float_parameter("y"))
                .with_parameter(float_parameter("z"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("follow_position", 5, 5, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(int_parameter("target_entity"))
                .with_parameter(float_parameter("offset_x"))
                .with_parameter(float_parameter("offset_y"))
                .with_parameter(float_parameter("offset_z"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("move_towards_entity", 4, 4, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(int_parameter("target_entity"))
                .with_parameter(float_parameter("speed"))
                .with_parameter(float_parameter("dt"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("camera_follow", 5, 5, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(int_parameter("target_entity"))
                .with_parameter(float_parameter("offset_x"))
                .with_parameter(float_parameter("offset_y"))
                .with_parameter(float_parameter("offset_z"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("component_json", 2, 2, ScriptHostValueKind::String)
                .with_parameter(int_parameter("entity"))
                .with_parameter(string_parameter("component_id"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("component_string", 3, 3, ScriptHostValueKind::String)
                .with_parameter(int_parameter("entity"))
                .with_parameter(string_parameter("component_id"))
                .with_parameter(string_parameter("fallback"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("set_component_json", 3, 3, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(string_parameter("component_id"))
                .with_parameter(string_parameter("component_json"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("find_by_component", 1, 1, ScriptHostValueKind::String)
                .with_parameter(string_parameter("component_id"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("nearest_by_script_property", 4, 4, ScriptHostValueKind::Int)
                .with_parameter(int_parameter("source_entity"))
                .with_parameter(string_parameter("property"))
                .with_parameter(string_parameter("value"))
                .with_parameter(float_parameter("max_distance"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("count_by_script_property", 2, 2, ScriptHostValueKind::Int)
                .with_parameter(string_parameter("property"))
                .with_parameter(string_parameter("value"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("script_property_matches", 3, 3, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(string_parameter("property"))
                .with_parameter(string_parameter("value"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("script_number", 3, 3, ScriptHostValueKind::Float)
                .with_parameter(int_parameter("entity"))
                .with_parameter(string_parameter("property"))
                .with_parameter(float_parameter("fallback"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("set_animation_bool", 3, 3, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(string_parameter("parameter"))
                .with_parameter(ScriptHostParameterDescriptor::new(
                    "value",
                    ScriptHostValueKind::Bool,
                ))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("damage_entity", 2, 2, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(float_parameter("damage"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("heal_entity", 3, 3, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(float_parameter("amount"))
                .with_parameter(float_parameter("max_hp"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("current_hp", 2, 2, ScriptHostValueKind::Float)
                .with_parameter(int_parameter("entity"))
                .with_parameter(float_parameter("fallback_hp"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("damage_entity_report", 2, 2, ScriptHostValueKind::String)
                .with_parameter(int_parameter("entity"))
                .with_parameter(float_parameter("damage"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("spawn_empty", 2, 2, ScriptHostValueKind::Int)
                .with_parameter(string_parameter("name"))
                .with_parameter(string_parameter("position_json"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("spawn_model", 5, 5, ScriptHostValueKind::Int)
                .with_parameter(string_parameter("name"))
                .with_parameter(string_parameter("position_json"))
                .with_parameter(string_parameter("model_ref"))
                .with_parameter(string_parameter("material_ref"))
                .with_parameter(string_parameter("script_bindings_json"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("set_hud_text", 2, 2, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(string_parameter("text"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("set_particle_sprites", 2, 2, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(string_parameter("sprites_json"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("set_world_hud_bar", 6, 6, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(float_parameter("max_hp"))
                .with_parameter(float_parameter("width"))
                .with_parameter(float_parameter("height"))
                .with_parameter(float_parameter("y_offset"))
                .with_parameter(float_parameter("intensity"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("despawn", 1, 1, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("nav_next_point_json", 2, 2, ScriptHostValueKind::String)
                .with_parameter(string_parameter("start_json"))
                .with_parameter(string_parameter("end_json"))
                .with_required_capability("gameplay.navigation"),
        )
        .with_function(
            function("nav_move_towards_entity", 4, 4, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(int_parameter("target_entity"))
                .with_parameter(float_parameter("speed"))
                .with_parameter(float_parameter("dt"))
                .with_required_capability("gameplay.navigation"),
        )
        .with_documentation("Runtime gameplay host calls scoped to the active script entity.");

    let handle = exports.register_module(
        descriptor,
        [
            HostExportFunction::new("delta_seconds", |_| {
                let context = current_script_runtime_call_context()?;
                Ok(script_float(context.delta_seconds))
            }),
            HostExportFunction::new("entity", |_| {
                let context = current_script_runtime_call_context()?;
                Ok(ScriptHostValue::Int(context.entity as i64))
            }),
            HostExportFunction::new("key_pressed", key_pressed),
            HostExportFunction::new("position_json", position_json),
            HostExportFunction::new("position_x", |context| position_axis(context, 0)),
            HostExportFunction::new("position_y", |context| position_axis(context, 1)),
            HostExportFunction::new("position_z", |context| position_axis(context, 2)),
            HostExportFunction::new("set_position_json", set_position_json),
            HostExportFunction::new("set_position", set_position),
            HostExportFunction::new("translate_json", translate_json),
            HostExportFunction::new("translate", translate),
            HostExportFunction::new("face_direction", face_direction),
            HostExportFunction::new("set_scale", set_scale),
            HostExportFunction::new("follow_position", follow_position),
            HostExportFunction::new("move_towards_entity", move_towards_entity),
            HostExportFunction::new("camera_follow", camera_follow),
            HostExportFunction::new("component_json", component_json),
            HostExportFunction::new("component_string", component_string),
            HostExportFunction::new("set_component_json", set_component_json),
            HostExportFunction::new("find_by_component", find_by_component),
            HostExportFunction::new("nearest_by_script_property", nearest_by_script_property),
            HostExportFunction::new("count_by_script_property", count_by_script_property),
            HostExportFunction::new("script_property_matches", script_property_matches),
            HostExportFunction::new("script_number", script_number),
            HostExportFunction::new("set_animation_bool", set_animation_bool),
            HostExportFunction::new("damage_entity", damage_entity),
            HostExportFunction::new("heal_entity", heal_entity),
            HostExportFunction::new("current_hp", current_hp),
            HostExportFunction::new("damage_entity_report", damage_entity_report),
            HostExportFunction::new("spawn_empty", spawn_empty),
            HostExportFunction::new("spawn_model", spawn_model),
            HostExportFunction::new("set_hud_text", set_hud_text),
            HostExportFunction::new("set_particle_sprites", set_particle_sprites),
            HostExportFunction::new("set_world_hud_bar", set_world_hud_bar),
            HostExportFunction::new("despawn", despawn),
            HostExportFunction::new("nav_next_point_json", nav_next_point_json),
            HostExportFunction::new("nav_move_towards_entity", nav_move_towards_entity),
        ],
    )?;
    Ok(Some(handle))
}

fn key_pressed(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
    let key = expect_string(context, 0)?;
    let runtime = current_script_runtime_call_context()?;
    let core = runtime.core_handle()?;
    let input = resolve_input_manager(&core).map_err(script_core_error)?;
    let snapshot = input.frame_snapshot();
    let pressed = parse_key_code(&key)
        .map(InputButton::KeyCode)
        .map(|button| snapshot.buttons.pressed(&button))
        .unwrap_or_else(|| snapshot.buttons.pressed(&InputButton::Key(key)));
    Ok(ScriptHostValue::Bool(pressed))
}

fn position_json(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let runtime = current_script_runtime_call_context()?;
    let position = runtime.level.with_world(|world| {
        world
            .world_transform(entity)
            .map(|transform| vec3_to_array(transform.translation))
    });
    Ok(ScriptHostValue::String(to_json_string(&position)?))
}

fn position_axis(
    context: &ScriptHostCallContext,
    axis: usize,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let runtime = current_script_runtime_call_context()?;
    let position = runtime.level.with_world(|world| {
        world
            .world_transform(entity)
            .map(|transform| vec3_to_array(transform.translation))
            .unwrap_or([0.0, 0.0, 0.0])
    });
    Ok(ScriptHostValue::Float(f64::from(position[axis])))
}

fn set_position_json(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let position = expect_vec3_json(context, 1)?;
    set_entity_position(entity, position)
}

fn set_position(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let position = Vec3::new(
        expect_float(context, 1)?,
        expect_float(context, 2)?,
        expect_float(context, 3)?,
    );
    set_entity_position(entity, position)
}

fn translate_json(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let delta = expect_vec3_json(context, 1)?;
    translate_entity(entity, delta)
}

fn translate(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let delta = Vec3::new(
        expect_float(context, 1)?,
        expect_float(context, 2)?,
        expect_float(context, 3)?,
    );
    translate_entity(entity, delta)
}

fn face_direction(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let direction = Vec3::new(expect_float(context, 1)?, 0.0, expect_float(context, 2)?);
    face_entity_direction(entity, direction)
}

fn set_scale(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let scale = Vec3::new(
        expect_float(context, 1)?,
        expect_float(context, 2)?,
        expect_float(context, 3)?,
    );
    set_entity_scale(entity, scale)
}

fn follow_position(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let target_entity = expect_entity(context, 1)?;
    let offset = Vec3::new(
        expect_float(context, 2)?,
        expect_float(context, 3)?,
        expect_float(context, 4)?,
    );
    follow_entity_position(entity, target_entity, offset)
}

fn move_towards_entity(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let target_entity = expect_entity(context, 1)?;
    let speed = expect_float(context, 2)?;
    let dt = expect_float(context, 3)?;
    move_entity_towards_target(entity, target_entity, speed, dt, false)
}

fn camera_follow(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let target_entity = expect_entity(context, 1)?;
    let offset = Vec3::new(
        expect_float(context, 2)?,
        expect_float(context, 3)?,
        expect_float(context, 4)?,
    );
    let runtime = current_script_runtime_call_context()?;
    let result = runtime.level.with_world_mut(|world| {
        let Some(target) = world.world_transform(target_entity) else {
            return Err(format!(
                "camera follow target entity {target_entity} is missing"
            ));
        };
        let eye = target.translation + offset;
        let focus = target.translation + Vec3::Y;
        world.update_transform(entity, Transform::looking_at(eye, focus, Vec3::Y))
    });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::new)
}

fn component_json(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
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

fn component_string(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
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

fn set_component_json(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
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

fn find_by_component(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
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

fn nearest_by_script_property(
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

fn count_by_script_property(
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

fn script_property_matches(
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

fn script_number(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
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

fn set_animation_bool(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
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
            .map(|_| true)
    });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::new)
}

fn damage_entity(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
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
                .map(|_| true)
        }
    });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::new)
}

fn heal_entity(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
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
            .map(|_| true)
    });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::new)
}

fn current_hp(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
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

fn damage_entity_report(
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
            world.set_dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT, bindings)?;
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

fn spawn_empty(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
    let name = expect_string(context, 0)?;
    let position = expect_vec3_json(context, 1)?;
    let runtime = current_script_runtime_call_context()?;
    let entity = runtime.level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::Empty);
        let mut transform = Transform::default();
        transform.translation = position;
        let _ = world.update_transform(entity, transform);
        let _ = world.insert(entity, crate::scene::components::Name(name));
        entity
    });
    Ok(ScriptHostValue::Int(entity as i64))
}

fn spawn_model(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
    let name = expect_string(context, 0)?;
    let position = expect_vec3_json(context, 1)?;
    let model_ref = expect_string(context, 2)?;
    let material_ref = expect_string(context, 3)?;
    let script_bindings_json = expect_string(context, 4)?;
    let script_bindings =
        serde_json::from_str::<serde_json::Value>(&script_bindings_json).map_err(json_error)?;
    let runtime = current_script_runtime_call_context()?;
    let entity = runtime
        .level
        .with_world_mut(|world| -> Result<u64, String> {
            let entity = world.spawn_node(NodeKind::Mesh);
            let mut transform = Transform::default();
            transform.translation = position;
            world.update_transform(entity, transform)?;
            world.insert(entity, Name(name))?;
            world.insert(
                entity,
                MeshRenderer::from_handles(
                    resource_handle_from_script_ref::<ModelMarker>(&model_ref),
                    resource_handle_from_script_ref::<MaterialMarker>(&material_ref),
                ),
            )?;
            world.set_dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT, script_bindings)?;
            Ok(entity)
        });
    entity
        .map(|entity| ScriptHostValue::Int(entity as i64))
        .map_err(ScriptHostError::new)
}

fn set_hud_text(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let text = expect_string(context, 1)?;
    let runtime = current_script_runtime_call_context()?;
    let result = runtime.level.with_world_mut(|world| {
        world.set_dynamic_component(entity, "gameplay.hud_text", serde_json::json!(text))
    });
    result
        .map(|_| ScriptHostValue::Bool(true))
        .map_err(ScriptHostError::new)
}

fn set_particle_sprites(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let sprites_json = expect_string(context, 1)?;
    let value = serde_json::from_str::<serde_json::Value>(&sprites_json).map_err(json_error)?;
    let runtime = current_script_runtime_call_context()?;
    let result = runtime.level.with_world_mut(|world| {
        world.set_dynamic_component(entity, "render.particle_sprites", value)
    });
    result
        .map(|_| ScriptHostValue::Bool(true))
        .map_err(ScriptHostError::new)
}

fn set_world_hud_bar(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let max_hp = expect_float(context, 1)?.max(1.0);
    let width = expect_float(context, 2)?.max(0.05);
    let height = expect_float(context, 3)?.max(0.02);
    let y_offset = expect_float(context, 4)?;
    let intensity = expect_float(context, 5)?.max(0.0);
    let runtime = current_script_runtime_call_context()?;
    let result = runtime.level.with_world_mut(|world| {
        let Some(position) = world
            .world_transform(entity)
            .map(|transform| transform.translation)
        else {
            return Ok(false);
        };
        let hp = world
            .dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT)
            .and_then(|bindings| script_binding_number(bindings, "hp"))
            .unwrap_or(f64::from(max_hp));
        let ratio = (hp / f64::from(max_hp)).clamp(0.0, 1.0);
        let fill_color = if ratio > 0.62 {
            [0.26, 0.95, 0.42, 0.88]
        } else if ratio > 0.28 {
            [1.0, 0.62, 0.14, 0.90]
        } else {
            [1.0, 0.08, 0.12, 0.94]
        };
        let value = serde_json::json!({
            "bars": [{
                "position": [position.x, position.y + y_offset, position.z],
                "width": width,
                "height": height,
                "ratio": ratio,
                "back_color": [0.04, 0.035, 0.04, 0.72],
                "fill_color": fill_color,
                "intensity": intensity
            }]
        });
        world
            .set_dynamic_component(entity, "render.world_hud_bars", value)
            .map(|_| true)
    });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::new)
}

fn despawn(context: &ScriptHostCallContext) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let runtime = current_script_runtime_call_context()?;
    let removed = runtime
        .level
        .with_world_mut(|world| world.remove_entity(entity));
    Ok(ScriptHostValue::Bool(removed))
}

fn nav_next_point_json(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let start = expect_vec3_json(context, 0)?;
    let end = expect_vec3_json(context, 1)?;
    let runtime = current_script_runtime_call_context()?;
    let core = runtime.core_handle()?;
    let navigation = resolve_navigation_manager(&core).map_err(script_core_error)?;
    let result = navigation
        .find_path(NavPathQuery {
            nav_mesh: None,
            start: vec3_to_array(start),
            end: vec3_to_array(end),
            agent_type: DEFAULT_AGENT_TYPE.to_string(),
            area_mask: DEFAULT_AREA_MASK,
        })
        .map_err(|error| ScriptHostError::new(error.to_string()))?;
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

fn nav_move_towards_entity(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let target_entity = expect_entity(context, 1)?;
    let speed = expect_float(context, 2)?;
    let dt = expect_float(context, 3)?;
    move_entity_with_navigation(entity, target_entity, speed, dt)
}

fn set_entity_position(entity: u64, position: Vec3) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = current_script_runtime_call_context()?;
    let result = runtime.level.with_world_mut(|world| {
        let mut transform = world
            .world_transform(entity)
            .unwrap_or_else(Transform::default);
        transform.translation = position;
        world.update_transform(entity, transform)
    });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::new)
}

fn translate_entity(entity: u64, delta: Vec3) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = current_script_runtime_call_context()?;
    let result = runtime.level.with_world_mut(|world| {
        let mut transform = world
            .world_transform(entity)
            .unwrap_or_else(Transform::default);
        transform.translation += delta;
        world.update_transform(entity, transform)
    });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::new)
}

fn face_entity_direction(entity: u64, direction: Vec3) -> Result<ScriptHostValue, ScriptHostError> {
    let planar = Vec3::new(direction.x, 0.0, direction.z);
    if planar.length_squared() <= f32::EPSILON {
        return Ok(ScriptHostValue::Bool(false));
    }
    let yaw = planar.x.atan2(-planar.z);
    let runtime = current_script_runtime_call_context()?;
    let result = runtime.level.with_world_mut(|world| {
        let mut transform = world
            .world_transform(entity)
            .unwrap_or_else(Transform::default);
        transform.rotation = Quat::from_rotation_y(yaw);
        world.update_transform(entity, transform)
    });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::new)
}

fn set_entity_scale(entity: u64, scale: Vec3) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = current_script_runtime_call_context()?;
    let result = runtime.level.with_world_mut(|world| {
        let mut transform = world
            .world_transform(entity)
            .unwrap_or_else(Transform::default);
        transform.scale = scale;
        world.update_transform(entity, transform)
    });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::new)
}

fn follow_entity_position(
    entity: u64,
    target_entity: u64,
    offset: Vec3,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = current_script_runtime_call_context()?;
    let result = runtime.level.with_world_mut(|world| {
        let Some(target) = world.world_transform(target_entity) else {
            return Err(format!("follow target entity {target_entity} is missing"));
        };
        let mut transform = world
            .world_transform(entity)
            .unwrap_or_else(Transform::default);
        transform.translation = target.translation + offset;
        world.update_transform(entity, transform)
    });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::new)
}

fn move_entity_towards_target(
    entity: u64,
    target_entity: u64,
    speed: f32,
    dt: f32,
    prefer_navigation: bool,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = current_script_runtime_call_context()?;
    let (start, target) = runtime.level.with_world(|world| {
        (
            world
                .world_transform(entity)
                .map(|transform| transform.translation),
            world
                .world_transform(target_entity)
                .map(|transform| transform.translation),
        )
    });
    let Some(start) = start else {
        return Err(ScriptHostError::new(format!("entity {entity} is missing")));
    };
    let Some(target) = target else {
        return Err(ScriptHostError::new(format!(
            "target entity {target_entity} is missing"
        )));
    };
    let target = if prefer_navigation {
        navigation_next_point(&runtime, start, target).unwrap_or(target)
    } else {
        target
    };
    let delta = target - start;
    let distance = delta.length();
    if distance <= f32::EPSILON {
        return Ok(ScriptHostValue::Bool(false));
    }
    let step = delta.normalize() * (speed * dt).min(distance);
    translate_entity(entity, step)
}

fn move_entity_with_navigation(
    entity: u64,
    target_entity: u64,
    speed: f32,
    dt: f32,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = current_script_runtime_call_context()?;
    let core = runtime.core_handle()?;
    let navigation = resolve_navigation_manager(&core).map_err(script_core_error)?;
    let target = runtime.level.with_world(|world| {
        world
            .world_transform(target_entity)
            .map(|transform| transform.translation)
    });
    let Some(target) = target else {
        return Err(ScriptHostError::new(format!(
            "target entity {target_entity} is missing"
        )));
    };
    let result = runtime.level.with_world_mut(|world| {
        if world.world_transform(entity).is_none() {
            return Err(format!("entity {entity} is missing"));
        }
        let mut agent = world
            .dynamic_component(entity, NAV_MESH_AGENT_COMPONENT_TYPE)
            .and_then(|value| serde_json::from_value::<NavMeshAgentDescriptor>(value.clone()).ok())
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
        let value = serde_json::to_value(agent).map_err(|error| error.to_string())?;
        world.set_dynamic_component(entity, NAV_MESH_AGENT_COMPONENT_TYPE, value)?;
        navigation
            .tick_world_agent(world, entity, dt)
            .map_err(|error| error.to_string())
    });
    result
        .map(|report| ScriptHostValue::Bool(report.moved_agents > 0))
        .map_err(ScriptHostError::new)
}

fn navigation_next_point(
    runtime: &crate::script::ScriptRuntimeCallContext,
    start: Vec3,
    target: Vec3,
) -> Option<Vec3> {
    let core = runtime.core_handle().ok()?;
    let navigation = resolve_navigation_manager(&core).ok()?;
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

fn function(
    name: impl Into<String>,
    min_argument_count: usize,
    max_argument_count: usize,
    return_value_kind: ScriptHostValueKind,
) -> ScriptHostFunctionDescriptor {
    ScriptHostFunctionDescriptor::new(
        name,
        min_argument_count,
        max_argument_count,
        return_value_kind,
    )
}

fn string_parameter(name: impl Into<String>) -> ScriptHostParameterDescriptor {
    ScriptHostParameterDescriptor::new(name, ScriptHostValueKind::String)
}

fn int_parameter(name: impl Into<String>) -> ScriptHostParameterDescriptor {
    ScriptHostParameterDescriptor::new(name, ScriptHostValueKind::Int)
}

fn float_parameter(name: impl Into<String>) -> ScriptHostParameterDescriptor {
    ScriptHostParameterDescriptor::new(name, ScriptHostValueKind::Float)
}

fn expect_string(context: &ScriptHostCallContext, index: usize) -> Result<String, ScriptHostError> {
    match context.arguments.get(index) {
        Some(ScriptHostValue::String(value)) => Ok(value.clone()),
        Some(value) => Err(ScriptHostError::new(format!(
            "argument {index} expected string, received {:?}",
            value.kind()
        ))),
        None => Err(ScriptHostError::new(format!(
            "argument {index} was not provided"
        ))),
    }
}

fn expect_entity(context: &ScriptHostCallContext, index: usize) -> Result<u64, ScriptHostError> {
    match context.arguments.get(index) {
        Some(ScriptHostValue::Int(value)) if *value >= 0 => Ok(*value as u64),
        Some(ScriptHostValue::HostHandle(value)) => Ok(*value),
        Some(value) => Err(ScriptHostError::new(format!(
            "argument {index} expected entity id, received {:?}",
            value.kind()
        ))),
        None => Err(ScriptHostError::new(format!(
            "argument {index} was not provided"
        ))),
    }
}

fn expect_float(context: &ScriptHostCallContext, index: usize) -> Result<f32, ScriptHostError> {
    match context.arguments.get(index) {
        Some(ScriptHostValue::Float(value)) => Ok(*value as f32),
        Some(ScriptHostValue::Int(value)) => Ok(*value as f32),
        Some(value) => Err(ScriptHostError::new(format!(
            "argument {index} expected float, received {:?}",
            value.kind()
        ))),
        None => Err(ScriptHostError::new(format!(
            "argument {index} was not provided"
        ))),
    }
}

fn expect_bool(context: &ScriptHostCallContext, index: usize) -> Result<bool, ScriptHostError> {
    match context.arguments.get(index) {
        Some(ScriptHostValue::Bool(value)) => Ok(*value),
        Some(value) => Err(ScriptHostError::new(format!(
            "argument {index} expected bool, received {:?}",
            value.kind()
        ))),
        None => Err(ScriptHostError::new(format!(
            "argument {index} was not provided"
        ))),
    }
}

fn expect_vec3_json(
    context: &ScriptHostCallContext,
    index: usize,
) -> Result<Vec3, ScriptHostError> {
    let value = expect_string(context, index)?;
    vec3_from_json(&value)
}

fn parse_key_code(key: &str) -> Option<u32> {
    key.strip_prefix("KeyCode:")
        .and_then(|value| value.parse::<u32>().ok())
        .or_else(|| key.parse::<u32>().ok())
}

fn vec3_from_json(value: &str) -> Result<Vec3, ScriptHostError> {
    let array = serde_json::from_str::<[f32; 3]>(value).map_err(json_error)?;
    Ok(Vec3::new(array[0], array[1], array[2]))
}

fn vec3_to_array(value: Vec3) -> [f32; 3] {
    [value.x, value.y, value.z]
}

fn resource_handle_from_script_ref<T>(value: &str) -> ResourceHandle<T> {
    let value = value.trim();
    let id = value
        .parse::<AssetUuid>()
        .map(ResourceId::from_asset_uuid)
        .or_else(|_| value.parse::<ResourceId>())
        .unwrap_or_else(|_| ResourceId::from_stable_label(value));
    ResourceHandle::new(id)
}

fn to_json_string<T: serde::Serialize>(value: &T) -> Result<String, ScriptHostError> {
    serde_json::to_string(value).map_err(json_error)
}

fn json_error(error: serde_json::Error) -> ScriptHostError {
    ScriptHostError::new(format!("invalid JSON payload: {error}"))
}

fn script_core_error(error: crate::core::CoreError) -> ScriptHostError {
    ScriptHostError::new(error.to_string())
}

#[cfg(test)]
mod tests;
