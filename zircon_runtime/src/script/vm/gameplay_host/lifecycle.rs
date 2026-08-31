use crate::core::framework::script::{
    ScriptHostCallFrame, ScriptHostError, ScriptHostHotPathMetrics, ScriptHostValue,
};
use crate::core::math::Transform;
use crate::core::resource::{MaterialMarker, ModelMarker};
use crate::scene::components::{MeshRenderer, Name, NodeKind};
use crate::script::runtime_context_for_frame;
use crate::script::vm::scene_system::with_script_binding_number_and_world_mut;

use super::error::GameplayHostResult;
use super::script_bindings::SCRIPT_BINDINGS_COMPONENT;
use super::values::{
    expect_entity, expect_float, expect_vec3_json, json_error, resource_handle_from_script_ref,
    with_string,
};

pub(super) fn spawn_empty(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let position = expect_vec3_json(context, 1)?;
    let runtime = runtime_context_for_frame(context)?;
    with_string(context, 0, |name: &str| {
        let entity = runtime
            .level
            .with_world_mut(|world| -> GameplayHostResult<u64> {
                let entity = world.spawn_node(NodeKind::Empty)?;
                let mut transform = Transform::default();
                transform.translation = position;
                world.update_transform(entity, transform)?;
                ScriptHostHotPathMetrics::record_guest_string_copy(name.len());
                world.insert(entity, crate::scene::components::Name(name.to_owned()))?;
                Ok(entity)
            });
        entity
            .map(|entity| ScriptHostValue::Int(entity as i64))
            .map_err(ScriptHostError::from)
    })
}

pub(super) fn spawn_model(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let position = expect_vec3_json(context, 1)?;
    let runtime = runtime_context_for_frame(context)?;
    with_string(context, 0, |name: &str| {
        with_string(context, 2, |model_ref: &str| {
            with_string(context, 3, |material_ref: &str| {
                with_string(context, 4, |script_bindings_json: &str| {
                    let script_bindings =
                        serde_json::from_str::<serde_json::Value>(script_bindings_json)
                            .map_err(json_error)?;
                    let entity = runtime
                        .level
                        .with_world_mut(|world| -> GameplayHostResult<u64> {
                            let entity = world.spawn_node(NodeKind::Mesh)?;
                            let mut transform = Transform::default();
                            transform.translation = position;
                            world.update_transform(entity, transform)?;
                            ScriptHostHotPathMetrics::record_guest_string_copy(name.len());
                            world.insert(entity, Name(name.to_owned()))?;
                            world.insert(
                                entity,
                                MeshRenderer::from_handles(
                                    resource_handle_from_script_ref::<ModelMarker>(model_ref),
                                    resource_handle_from_script_ref::<MaterialMarker>(material_ref),
                                ),
                            )?;
                            world.set_dynamic_component(
                                entity,
                                SCRIPT_BINDINGS_COMPONENT,
                                script_bindings,
                            )?;
                            Ok(entity)
                        });
                    entity
                        .map(|entity| ScriptHostValue::Int(entity as i64))
                        .map_err(ScriptHostError::from)
                })
            })
        })
    })
}

pub(super) fn set_hud_text(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let runtime = runtime_context_for_frame(context)?;
    with_string(context, 1, |text: &str| {
        let result = runtime
            .level
            .with_world_mut(|world| -> GameplayHostResult<bool> {
                ScriptHostHotPathMetrics::record_guest_string_copy(text.len());
                world.set_dynamic_component(
                    entity,
                    "gameplay.hud_text",
                    serde_json::Value::String(text.to_owned()),
                )?;
                Ok(true)
            });
        result
            .map(ScriptHostValue::Bool)
            .map_err(ScriptHostError::from)
    })
}

pub(super) fn set_particle_sprites(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let runtime = runtime_context_for_frame(context)?;
    with_string(context, 1, |sprites_json: &str| {
        let value = serde_json::from_str::<serde_json::Value>(sprites_json).map_err(json_error)?;
        let result = runtime
            .level
            .with_world_mut(|world| -> GameplayHostResult<bool> {
                world.set_dynamic_component(entity, "render.particle_sprites", value)?;
                Ok(true)
            });
        result
            .map(ScriptHostValue::Bool)
            .map_err(ScriptHostError::from)
    })
}

pub(super) fn set_world_hud_bar(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let max_hp = expect_float(context, 1)?.max(1.0);
    let width = expect_float(context, 2)?.max(0.05);
    let height = expect_float(context, 3)?.max(0.02);
    let y_offset = expect_float(context, 4)?;
    let intensity = expect_float(context, 5)?.max(0.0);
    let runtime = runtime_context_for_frame(context)?;
    let result = with_script_binding_number_and_world_mut(
        &runtime.level,
        entity,
        "hp",
        |hp, world| -> GameplayHostResult<bool> {
            let hp = hp.unwrap_or(f64::from(max_hp));
            let Some(position) = world
                .world_transform(entity)
                .map(|transform| transform.translation)
            else {
                return Ok(false);
            };
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
            world.set_dynamic_component(entity, "render.world_hud_bars", value)?;
            Ok(true)
        },
    )
    .map_err(|error| ScriptHostError::new(error.to_string()))?;
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::from)
}

pub(super) fn despawn(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let runtime = runtime_context_for_frame(context)?;
    let removed = runtime
        .level
        .with_world_mut(|world| world.remove_entity(entity));
    removed
        .map(|()| ScriptHostValue::Bool(true))
        .map_err(|error| ScriptHostError::new(error.to_string()))
}
