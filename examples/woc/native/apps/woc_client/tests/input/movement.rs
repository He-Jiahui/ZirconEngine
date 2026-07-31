use woc_client::{
    keyboard_movement_flags, resolve_movement_input, GamepadMoveFlags, Keybinds,
    MovementInputFlags, MovementInputSources,
};

fn keyboard(forward: bool, turn_left: bool, strafe_left: bool, jump: bool) -> MovementInputFlags {
    MovementInputFlags {
        forward,
        turn_left,
        strafe_left,
        jump,
        ..MovementInputFlags::default()
    }
}

#[test]
fn held_keyboard_flags_use_physical_codes_and_only_reserve_the_exact_attack_move_binding() {
    let mut bindings = Keybinds::default();
    assert_eq!(
        keyboard_movement_flags(&bindings, &["KeyW", "KeyA", "KeyE", "Space"], false),
        MovementInputFlags {
            forward: true,
            turn_left: true,
            strafe_right: true,
            jump: true,
            ..MovementInputFlags::default()
        }
    );
    assert!(!keyboard_movement_flags(&bindings, &["KeyA"], true).turn_left);

    assert!(bindings.bind("attackMove", 0, "Shift+KeyA"));
    assert!(keyboard_movement_flags(&bindings, &["KeyA"], true).turn_left);
}

#[test]
fn suspended_movement_keeps_only_autorun_before_controller_override() {
    let resolved = resolve_movement_input(MovementInputSources {
        suspended: true,
        autorun: true,
        controller_override: Some(MovementInputFlags {
            back: true,
            jump: true,
            ..MovementInputFlags::default()
        }),
        keyboard: keyboard(true, true, true, true),
        touch: GamepadMoveFlags {
            strafe_right: true,
            ..GamepadMoveFlags::default()
        },
        ..MovementInputSources::default()
    });
    assert_eq!(
        resolved,
        MovementInputFlags {
            forward: true,
            ..MovementInputFlags::default()
        }
    );
}

#[test]
fn controller_override_wins_over_every_merged_held_source() {
    let controller = MovementInputFlags {
        back: true,
        turn_right: true,
        jump: true,
        ..MovementInputFlags::default()
    };
    assert_eq!(
        resolve_movement_input(MovementInputSources {
            controller_override: Some(controller),
            keyboard: keyboard(true, true, true, true),
            pointer_forward: true,
            autorun: true,
            latched_jump: true,
            touch: GamepadMoveFlags {
                forward: true,
                ..GamepadMoveFlags::default()
            },
            gamepad: GamepadMoveFlags {
                strafe_right: true,
                ..GamepadMoveFlags::default()
            },
            ..MovementInputSources::default()
        }),
        controller
    );
}

#[test]
fn ordinary_mode_ors_all_held_sources_and_keeps_keyboard_turns() {
    let resolved = resolve_movement_input(MovementInputSources {
        keyboard: keyboard(false, true, true, false),
        pointer_forward: true,
        latched_jump: true,
        touch: GamepadMoveFlags {
            back: true,
            ..GamepadMoveFlags::default()
        },
        gamepad: GamepadMoveFlags {
            strafe_right: true,
            ..GamepadMoveFlags::default()
        },
        ..MovementInputSources::default()
    });
    assert_eq!(
        resolved,
        MovementInputFlags {
            forward: true,
            back: true,
            turn_left: true,
            strafe_left: true,
            strafe_right: true,
            jump: true,
            ..MovementInputFlags::default()
        }
    );
}

#[test]
fn mouse_camera_and_mouselook_fold_keyboard_turns_into_strafe() {
    let sources = MovementInputSources {
        keyboard: MovementInputFlags {
            turn_left: true,
            turn_right: true,
            ..MovementInputFlags::default()
        },
        ..MovementInputSources::default()
    };
    assert_eq!(
        resolve_movement_input(MovementInputSources {
            mouse_camera: true,
            ..sources
        }),
        MovementInputFlags {
            strafe_left: true,
            strafe_right: true,
            ..MovementInputFlags::default()
        }
    );
    assert_eq!(
        resolve_movement_input(MovementInputSources {
            mouselook: true,
            ..sources
        }),
        MovementInputFlags {
            strafe_left: true,
            strafe_right: true,
            ..MovementInputFlags::default()
        }
    );
}
