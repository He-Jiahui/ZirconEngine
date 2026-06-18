use crate::core::framework::script::{
    ScriptHostFunctionDescriptor, ScriptHostModuleDescriptor, ScriptHostParameterDescriptor,
    ScriptHostValue, ScriptHostValueKind,
};
use crate::script::{
    current_script_runtime_call_context, script_float, HostExportFunction, HostExportRegistry,
    HostHandle, VmError,
};

mod combat;
mod components;
mod input;
mod lifecycle;
mod navigation;
mod script_bindings;
mod transform;
mod values;

use combat::{current_hp, damage_entity, damage_entity_report, heal_entity, set_animation_bool};
use components::{
    component_json, component_string, count_by_script_property, entity_exists, find_by_component,
    nearest_by_script_property, script_number, script_number_at_most, script_property_matches,
    set_component_json,
};
use input::key_pressed;
use lifecycle::{
    despawn, set_hud_text, set_particle_sprites, set_world_hud_bar, spawn_empty, spawn_model,
};
use navigation::{nav_move_towards_entity, nav_next_point_json};
use transform::{
    camera_follow, face_direction, follow_position, move_towards_entity, position_axis,
    position_json, set_position, set_position_json, set_scale, translate, translate_json,
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
            function("entity_exists", 1, 1, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
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
            function("script_number_at_most", 4, 4, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(string_parameter("property"))
                .with_parameter(float_parameter("threshold"))
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
            HostExportFunction::new("entity_exists", entity_exists),
            HostExportFunction::new("nearest_by_script_property", nearest_by_script_property),
            HostExportFunction::new("count_by_script_property", count_by_script_property),
            HostExportFunction::new("script_property_matches", script_property_matches),
            HostExportFunction::new("script_number", script_number),
            HostExportFunction::new("script_number_at_most", script_number_at_most),
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

#[cfg(test)]
mod tests;
