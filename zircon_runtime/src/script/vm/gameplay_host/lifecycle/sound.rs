use crate::core::framework::script::{ScriptHostCallFrame, ScriptHostError, ScriptHostValue};

/// The export remains discoverable in script-only builds; using it requires
/// sound contracts and an explicitly installed sound service.
#[cfg(not(feature = "sound-contracts"))]
pub(crate) fn play_sound(
    _context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    Err(ScriptHostError::new("play_sound requires sound-contracts"))
}

#[cfg(feature = "sound-contracts")]
pub(crate) fn play_sound(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    use crate::core::framework::script::ScriptHostHotPathMetrics;
    use crate::core::framework::sound::{SoundPlayRequest, SoundPlayRequests};
    use crate::core::manager::{resolve_manager_service, sound_manager_handle};
    use crate::script::runtime_context_for_frame;

    use super::super::error::{GameplayHostError, GameplayHostResult};
    use super::super::values::{expect_entity, script_core_error, with_string};

    let entity = expect_entity(context, 0)?;
    let runtime = runtime_context_for_frame(context)?;
    let epoch = runtime.level.with_world_and_replacement_epoch(
        |world, epoch| -> GameplayHostResult<u64> {
            if !world.contains_entity(entity) {
                return Err(GameplayHostError::missing_entity("play sound", entity));
            }
            Ok(epoch)
        },
    )?;
    let core = runtime.core_handle()?;
    let sound = sound_manager_handle(&core)
        .and_then(|handle| resolve_manager_service(&core, handle))
        .map_err(script_core_error)?;
    with_string(context, 1, |uri| {
        // URI validation, project resolution, decoding and clip caching all
        // remain in the existing sound/asset service. A failure queues nothing.
        sound
            .load_clip(uri)
            .map_err(|error| ScriptHostError::new(error.to_string()))?;
        runtime
            .level
            .with_world_mut_if_replacement_epoch(epoch, |world| -> GameplayHostResult<()> {
                if !world.contains_entity(entity) {
                    return Err(GameplayHostError::missing_entity("play sound", entity));
                }
                if !world.contains_resource::<SoundPlayRequests>() {
                    world.insert_resource(SoundPlayRequests::default());
                }
                ScriptHostHotPathMetrics::record_guest_string_copy(uri.len());
                world
                    .get_resource_mut::<SoundPlayRequests>()
                    .expect("sound inbox was inserted")
                    .0
                    .push(SoundPlayRequest {
                        entity,
                        uri: uri.to_owned(),
                    });
                Ok(())
            })
            .ok_or_else(|| ScriptHostError::new("play_sound world was replaced while loading"))??;
        Ok(ScriptHostValue::Bool(true))
    })
}
