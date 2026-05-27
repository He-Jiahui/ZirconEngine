use super::super::*;

#[test]
fn no_platform_features_fixture_disables_every_declared_backend() {
    assert_eq!(
        PlatformFeatureSelection::none(),
        PlatformFeatureSelection {
            platform_window: false,
            platform_winit: false,
            platform_headless: false,
            platform_x11: false,
            platform_wayland: false,
            platform_android_game_activity: false,
            platform_android_native_activity: false,
            platform_web: false,
            input_mouse: false,
            input_keyboard: false,
            input_touch: false,
            input_gestures: false,
            input_gamepad: false,
            gamepad_gilrs: false,
            gamepad_browser: false,
        }
    );
}

#[test]
fn bevy_default_platform_fixture_declares_window_input_and_desktop_gamepad_policy() {
    assert_eq!(
        PlatformFeatureSelection::bevy_default_platform(),
        PlatformFeatureSelection {
            platform_window: true,
            platform_winit: true,
            platform_headless: false,
            platform_x11: true,
            platform_wayland: true,
            platform_android_game_activity: true,
            platform_android_native_activity: false,
            platform_web: true,
            input_mouse: true,
            input_keyboard: true,
            input_touch: true,
            input_gestures: false,
            input_gamepad: true,
            gamepad_gilrs: true,
            gamepad_browser: false,
        }
    );
}

#[test]
fn headless_platform_fixture_enables_only_the_headless_topology_gate() {
    assert_eq!(
        PlatformFeatureSelection::headless(),
        PlatformFeatureSelection {
            platform_headless: true,
            ..PlatformFeatureSelection::none()
        }
    );
}

#[test]
fn compiled_feature_snapshot_reads_the_active_cargo_feature_set() {
    assert_eq!(
        PlatformFeatureSelection::from_compiled_features(),
        PlatformFeatureSelection {
            platform_window: cfg!(feature = "platform-window"),
            platform_winit: cfg!(feature = "platform-winit"),
            platform_headless: cfg!(feature = "platform-headless"),
            platform_x11: cfg!(feature = "platform-x11"),
            platform_wayland: cfg!(feature = "platform-wayland"),
            platform_android_game_activity: cfg!(feature = "platform-android-game-activity"),
            platform_android_native_activity: cfg!(feature = "platform-android-native-activity"),
            platform_web: cfg!(feature = "platform-web"),
            input_mouse: cfg!(feature = "input-mouse"),
            input_keyboard: cfg!(feature = "input-keyboard"),
            input_touch: cfg!(feature = "input-touch"),
            input_gestures: cfg!(feature = "input-gestures"),
            input_gamepad: cfg!(feature = "input-gamepad"),
            gamepad_gilrs: cfg!(feature = "gamepad-gilrs"),
            gamepad_browser: cfg!(feature = "gamepad-browser"),
        }
    );
}
