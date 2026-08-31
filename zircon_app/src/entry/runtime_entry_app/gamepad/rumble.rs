#[cfg(feature = "gamepad-gilrs")]
use std::collections::BTreeMap;
#[cfg(feature = "gamepad-gilrs")]
use std::time::{Duration, Instant};

#[cfg(feature = "gamepad-gilrs")]
use gilrs::ff::{
    BaseEffect, BaseEffectType, Effect, EffectBuilder, Error as ForceFeedbackError, Repeat, Replay,
    Ticks,
};
use zircon_runtime::diagnostic_log::write_warn;
#[cfg(feature = "gamepad-gilrs")]
use zircon_runtime_interface::ZrRuntimeGamepadRumbleRequestKindV1;
use zircon_runtime_interface::ZrRuntimeGamepadRumbleRequestV1;

use super::super::RuntimeEntryApp;
use super::events::gamepad_id;

#[cfg(feature = "gamepad-gilrs")]
pub(in crate::entry::runtime_entry_app) struct RunningRumbleEffect {
    deadline: Instant,
    effect: Effect,
}

#[cfg(feature = "gamepad-gilrs")]
pub(in crate::entry::runtime_entry_app) type RunningRumbleEffects =
    BTreeMap<u64, Vec<RunningRumbleEffect>>;

#[cfg(feature = "gamepad-gilrs")]
const RUMBLE_MS_MAX: u32 = 10_000;
#[cfg(feature = "gamepad-gilrs")]
const RUMBLE_EFFECTS_MAX_PER_GAMEPAD: usize = 32;

impl RuntimeEntryApp {
    #[cfg(feature = "gamepad-gilrs")]
    pub(in crate::entry::runtime_entry_app) fn apply_runtime_gamepad_rumble_request(
        &mut self,
        request: ZrRuntimeGamepadRumbleRequestV1,
    ) -> Result<(), &'static str> {
        let Some(gamepads) = self.gamepads.as_mut() else {
            return Err("runtime_gamepad_rumble_gilrs_unavailable");
        };
        let gamepad_id = resolve_gilrs_gamepad_id(gamepads, request.gamepad_id)
            .ok_or("runtime_gamepad_rumble_gamepad_not_found")?;
        match request.kind {
            ZrRuntimeGamepadRumbleRequestKindV1::Add => {
                if request.duration_millis == 0 {
                    return Ok(());
                }
                let duration_millis = request.duration_millis.min(RUMBLE_MS_MAX);
                let duration = Ticks::from_ms(duration_millis);
                let mut effect_builder = EffectBuilder::new();
                let has_effect = append_base_effects(
                    &mut effect_builder,
                    request.strong_motor,
                    request.weak_motor,
                    duration,
                );
                if !has_effect {
                    return Ok(());
                }
                let active_effect_count = self
                    .gamepad_rumble_effects
                    .as_ref()
                    .and_then(|effects| effects.get(&request.gamepad_id))
                    .map_or(0, Vec::len);
                admit_rumble_effect(active_effect_count)?;
                effect_builder.gamepads(&[gamepad_id]);
                effect_builder.repeat(Repeat::For(duration));
                let effect = effect_builder
                    .finish(gamepads)
                    .map_err(rumble_force_feedback_error)?;
                effect.play().map_err(rumble_force_feedback_error)?;
                self.gamepad_rumble_effects
                    .get_or_insert_with(BTreeMap::new)
                    .entry(request.gamepad_id)
                    .or_default()
                    .push(RunningRumbleEffect {
                        deadline: Instant::now() + Duration::from_millis(duration_millis as u64),
                        effect,
                    });
                Ok(())
            }
            ZrRuntimeGamepadRumbleRequestKindV1::Stop => {
                stop_gamepad_rumble_effects(&mut self.gamepad_rumble_effects, request.gamepad_id);
                Ok(())
            }
        }
    }

    #[cfg(not(feature = "gamepad-gilrs"))]
    pub(in crate::entry::runtime_entry_app) fn apply_runtime_gamepad_rumble_request(
        &mut self,
        _request: ZrRuntimeGamepadRumbleRequestV1,
    ) -> Result<(), &'static str> {
        Err("runtime_gamepad_rumble_feature_disabled")
    }
}

#[cfg(feature = "gamepad-gilrs")]
fn admit_rumble_effect(active_effect_count: usize) -> Result<(), &'static str> {
    if active_effect_count >= RUMBLE_EFFECTS_MAX_PER_GAMEPAD {
        Err("runtime_gamepad_rumble_effect_limit_reached")
    } else {
        Ok(())
    }
}

#[cfg(feature = "gamepad-gilrs")]
fn resolve_gilrs_gamepad_id(
    gilrs: &gilrs::Gilrs,
    runtime_gamepad_id: u64,
) -> Option<gilrs::GamepadId> {
    gilrs
        .gamepads()
        .find_map(|(id, _)| (gamepad_id(id) == runtime_gamepad_id).then_some(id))
}

#[cfg(feature = "gamepad-gilrs")]
fn append_base_effects(
    effect_builder: &mut EffectBuilder,
    strong_motor: f32,
    weak_motor: f32,
    duration: Ticks,
) -> bool {
    let strong = motor_magnitude(strong_motor);
    let weak = motor_magnitude(weak_motor);
    if strong == 0 && weak == 0 {
        return false;
    }
    if strong > 0 {
        effect_builder.add_effect(BaseEffect {
            kind: BaseEffectType::Strong { magnitude: strong },
            scheduling: Replay {
                play_for: duration,
                ..Default::default()
            },
            ..Default::default()
        });
    }
    if weak > 0 {
        effect_builder.add_effect(BaseEffect {
            kind: BaseEffectType::Weak { magnitude: weak },
            scheduling: Replay {
                play_for: duration,
                ..Default::default()
            },
            ..Default::default()
        });
    }
    true
}

#[cfg(feature = "gamepad-gilrs")]
fn motor_magnitude(value: f32) -> u16 {
    let clamped = if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    };
    (clamped * f32::from(u16::MAX)) as u16
}

#[cfg(feature = "gamepad-gilrs")]
fn rumble_force_feedback_error(error: ForceFeedbackError) -> &'static str {
    match error {
        ForceFeedbackError::FfNotSupported(_) => {
            "runtime_gamepad_rumble_force_feedback_not_supported"
        }
        ForceFeedbackError::Disconnected(_) => "runtime_gamepad_rumble_gamepad_disconnected",
        ForceFeedbackError::InvalidDistanceModel(_) => {
            "runtime_gamepad_rumble_invalid_distance_model"
        }
        ForceFeedbackError::SendFailed => "runtime_gamepad_rumble_effect_channel_failed",
        ForceFeedbackError::Other => "runtime_gamepad_rumble_effect_failed",
        _ => "runtime_gamepad_rumble_effect_failed",
    }
}

#[cfg(feature = "gamepad-gilrs")]
fn stop_gamepad_rumble_effects(
    gamepad_rumble_effects: &mut Option<RunningRumbleEffects>,
    runtime_gamepad_id: u64,
) {
    let Some(effect_map) = gamepad_rumble_effects.as_mut() else {
        return;
    };
    let Some(effects) = effect_map.remove(&runtime_gamepad_id) else {
        return;
    };
    for effect in effects {
        if let Err(error) = effect.effect.stop() {
            write_warn(
                "runtime_gamepad",
                format!(
                    "runtime_gamepad_rumble_stop_failed:gamepad_id={runtime_gamepad_id}:{}",
                    rumble_force_feedback_error(error)
                ),
            );
        }
    }
    if effect_map.is_empty() {
        *gamepad_rumble_effects = None;
    }
}

#[cfg(feature = "gamepad-gilrs")]
pub(in crate::entry::runtime_entry_app) fn clear_finished_rumble_effects(
    gamepad_rumble_effects: Option<&mut RunningRumbleEffects>,
) {
    let Some(effect_map) = gamepad_rumble_effects else {
        return;
    };
    let now = Instant::now();
    effect_map.values_mut().for_each(|effects| {
        effects.retain(|effect| effect.deadline > now);
    });
    effect_map.retain(|_, effects| !effects.is_empty());
}

#[cfg(feature = "gamepad-gilrs")]
pub(in crate::entry::runtime_entry_app) fn clear_gamepad_rumble_effects_for_gamepad(
    gamepad_rumble_effects: &mut Option<RunningRumbleEffects>,
    runtime_gamepad_id: u64,
) {
    stop_gamepad_rumble_effects(gamepad_rumble_effects, runtime_gamepad_id);
}

#[cfg(feature = "gamepad-gilrs")]
pub(in crate::entry::runtime_entry_app) fn clear_gamepad_rumble_effects(
    gamepad_rumble_effects: &mut Option<RunningRumbleEffects>,
) {
    if let Some(mut effect_map) = gamepad_rumble_effects.take() {
        for effects in effect_map.values_mut() {
            for effect in effects.drain(..) {
                if let Err(error) = effect.effect.stop() {
                    write_warn(
                        "runtime_gamepad",
                        format!(
                            "runtime_gamepad_rumble_shutdown_stop_failed:{}",
                            rumble_force_feedback_error(error)
                        ),
                    );
                }
            }
        }
    }
}

#[cfg(all(test, feature = "gamepad-gilrs"))]
mod tests {
    use super::*;

    #[test]
    fn rumble_effect_admission_has_a_fixed_per_gamepad_limit() {
        assert_eq!(RUMBLE_EFFECTS_MAX_PER_GAMEPAD, 32);
        assert_eq!(
            admit_rumble_effect(RUMBLE_EFFECTS_MAX_PER_GAMEPAD - 1),
            Ok(())
        );
        assert_eq!(
            admit_rumble_effect(RUMBLE_EFFECTS_MAX_PER_GAMEPAD),
            Err("runtime_gamepad_rumble_effect_limit_reached")
        );
    }

    #[test]
    fn rumble_add_admits_before_backend_creation_and_publish() {
        let source = include_str!("rumble.rs")
            .split_once("\n#[cfg(all(test, feature = \"gamepad-gilrs\"))]")
            .map(|(production, _)| production)
            .expect("rumble production source precedes its test module");
        let source = source
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();

        let admission = source
            .find("admit_rumble_effect(active_effect_count)?;")
            .expect("rumble Add checks the per-gamepad hard limit");
        let finish = source
            .find(".finish(gamepads)")
            .expect("rumble Add creates a backend effect only after admission");
        let play = source
            .find("effect.play().map_err(rumble_force_feedback_error)?;")
            .expect("rumble Add plays an admitted effect");
        let publish = source
            .find(".push(RunningRumbleEffect{")
            .expect("rumble Add publishes the running effect after play succeeds");

        assert!(admission < finish && finish < play && play < publish);
        assert!(source
            .contains("ZrRuntimeGamepadRumbleRequestKindV1::Stop=>{stop_gamepad_rumble_effects("));
        assert!(source.contains("effects.retain(|effect|effect.deadline>now);"));
        assert!(source.contains("for effect in effects.drain(..){"));
    }
}
