use woc_client::{
    is_chat_long_press, is_move_autorun_near, is_move_autorun_push, is_recenter_double_tap,
    map_touch_joystick, map_touch_look_vector, pinch_zoom_delta, resolve_touch_interface,
    touch_interface_mode_from_setting, GamepadMoveFlags, GamepadStickVector, TouchInterfaceMode,
    CHAT_LONG_PRESS_MS, MOVE_AUTORUN_REVEAL_THRESHOLD, MOVE_AUTORUN_THRESHOLD,
    RECENTER_DOUBLE_TAP_MS, TOUCH_JOYSTICK_DEADZONE,
};

#[test]
fn touch_joystick_maps_neutral_cardinal_and_diagonal_vectors() {
    assert_eq!(
        map_touch_joystick(0.0, 0.0, None),
        GamepadMoveFlags::default()
    );
    assert_eq!(
        map_touch_joystick(0.05, -0.08, None),
        GamepadMoveFlags::default()
    );
    assert!(map_touch_joystick(0.0, -1.0, None).forward);
    assert!(map_touch_joystick(0.0, 1.0, None).back);
    assert!(map_touch_joystick(-1.0, 0.0, None).strafe_left);
    assert!(map_touch_joystick(1.0, 0.0, None).strafe_right);
    assert_eq!(
        map_touch_joystick(0.7, -0.7, None),
        GamepadMoveFlags {
            forward: true,
            back: false,
            strafe_left: false,
            strafe_right: true,
        }
    );
}

#[test]
fn touch_joystick_honors_the_persisted_deadzone_setting() {
    assert!(map_touch_joystick(0.0, -0.3, None).forward);
    assert_eq!(
        map_touch_joystick(0.0, -0.3, Some(0.4)),
        GamepadMoveFlags::default()
    );
    assert!(map_touch_joystick(0.0, -0.15, Some(0.1)).forward);
    assert_eq!(TOUCH_JOYSTICK_DEADZONE, 0.22);
}

#[test]
fn autorun_reveals_before_its_stronger_lock_threshold() {
    assert!(is_move_autorun_push(
        -MOVE_AUTORUN_THRESHOLD,
        MOVE_AUTORUN_THRESHOLD
    ));
    assert!(!is_move_autorun_push(
        -MOVE_AUTORUN_THRESHOLD + 0.01,
        MOVE_AUTORUN_THRESHOLD
    ));
    assert!(is_move_autorun_near(
        -MOVE_AUTORUN_REVEAL_THRESHOLD,
        MOVE_AUTORUN_REVEAL_THRESHOLD
    ));
    assert!(!is_move_autorun_push(
        -MOVE_AUTORUN_REVEAL_THRESHOLD,
        MOVE_AUTORUN_THRESHOLD
    ));
}

#[test]
fn interface_mode_setting_and_override_match_the_target() {
    assert_eq!(
        touch_interface_mode_from_setting(0.0),
        TouchInterfaceMode::Auto
    );
    assert_eq!(
        touch_interface_mode_from_setting(1.0),
        TouchInterfaceMode::Desktop
    );
    assert_eq!(
        touch_interface_mode_from_setting(2.0),
        TouchInterfaceMode::Touch
    );
    assert_eq!(
        touch_interface_mode_from_setting(f64::NAN),
        TouchInterfaceMode::Auto
    );

    assert!(resolve_touch_interface(TouchInterfaceMode::Auto, true));
    assert!(!resolve_touch_interface(TouchInterfaceMode::Auto, false));
    assert!(!resolve_touch_interface(TouchInterfaceMode::Desktop, true));
    assert!(resolve_touch_interface(TouchInterfaceMode::Touch, false));
}

#[test]
fn chat_long_press_includes_the_threshold_boundary() {
    assert!(!is_chat_long_press(
        CHAT_LONG_PRESS_MS - 1,
        CHAT_LONG_PRESS_MS
    ));
    assert!(is_chat_long_press(CHAT_LONG_PRESS_MS, CHAT_LONG_PRESS_MS));
    assert!(is_chat_long_press(
        CHAT_LONG_PRESS_MS + 500,
        CHAT_LONG_PRESS_MS
    ));
}

#[test]
fn recenter_requires_a_prior_stationary_tap_inside_the_window() {
    assert!(is_recenter_double_tap(
        1_000,
        1_000 + RECENTER_DOUBLE_TAP_MS - 50,
        false,
        RECENTER_DOUBLE_TAP_MS
    ));
    assert!(!is_recenter_double_tap(
        1_000,
        1_100,
        true,
        RECENTER_DOUBLE_TAP_MS
    ));
    assert!(!is_recenter_double_tap(
        1_000,
        1_000 + RECENTER_DOUBLE_TAP_MS + 1,
        false,
        RECENTER_DOUBLE_TAP_MS
    ));
    assert!(!is_recenter_double_tap(
        0,
        120,
        false,
        RECENTER_DOUBLE_TAP_MS
    ));
}

#[test]
fn touch_look_keeps_analog_vector_outside_the_deadzone() {
    assert_eq!(
        map_touch_look_vector(0.02, 0.03, None),
        GamepadStickVector::default()
    );
    let vector = map_touch_look_vector(0.45, -0.25, None);
    assert!((vector.x - 0.36).abs() < 1.0e-9);
    assert!((vector.y + 0.2).abs() < 1.0e-9);
}

#[test]
fn pinch_zoom_ignores_jitter_and_preserves_direction() {
    assert_eq!(pinch_zoom_delta(120.0, 130.0, None, None), 0.0);
    assert_eq!(pinch_zoom_delta(120.0, 109.0, None, None), 0.0);
    assert!((pinch_zoom_delta(100.0, 150.0, None, None) + 1.33).abs() < 1.0e-9);
    assert!((pinch_zoom_delta(150.0, 100.0, None, None) - 1.33).abs() < 1.0e-9);
}
