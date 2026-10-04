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
    let source = include_str!("../rumble.rs")
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
    assert!(
        source.contains("ZrRuntimeGamepadRumbleRequestKindV1::Stop=>{stop_gamepad_rumble_effects(")
    );
    assert!(source.contains("effects.retain(|effect|effect.deadline>now);"));
    assert!(source.contains("for effect in effects.drain(..){"));
}
