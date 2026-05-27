use super::super::*;

#[test]
fn platform_window_gate_propagates_to_window_owned_capabilities() {
    let mut features = PlatformFeatureSelection::bevy_default_platform();
    features.platform_window = false;

    let report = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::Windows,
        crate::RuntimeTargetMode::ClientRuntime,
    );

    assert_eq!(
        report.window_backend,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-window"
        }
    );
    assert_eq!(
        report.monitor_inventory,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-window"
        }
    );
    assert_eq!(
        report.window_events,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-window"
        }
    );
    assert_eq!(
        report.window_lifecycle,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-window"
        }
    );
    assert_eq!(
        report.window_metrics,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-window"
        }
    );
    assert_eq!(
        report.pointer_position,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-window"
        }
    );
    assert_eq!(
        report.file_drag_drop,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-window"
        }
    );
}

#[test]
fn platform_winit_gate_propagates_to_desktop_window_host_capabilities() {
    let mut features = PlatformFeatureSelection::bevy_default_platform();
    features.platform_winit = false;

    let report = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::Linux,
        crate::RuntimeTargetMode::ClientRuntime,
    );

    assert_eq!(
        report.window_backend,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-winit"
        }
    );
    assert_eq!(
        report.ime,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-winit"
        }
    );
    assert_eq!(
        report.keyboard_events,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-winit"
        }
    );
    assert_eq!(
        report.cursor_boundary,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-winit"
        }
    );
    assert_eq!(
        report.mouse_buttons,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-winit"
        }
    );
    assert_eq!(
        report.mouse_wheel,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-winit"
        }
    );
    assert_eq!(
        report.raw_mouse_motion,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-winit"
        }
    );
}

#[test]
fn input_feature_gates_take_precedence_over_available_host_capability() {
    let mut features = PlatformFeatureSelection::bevy_default_platform();
    features.input_mouse = false;
    features.input_keyboard = false;
    features.input_touch = false;
    features.input_gestures = false;
    features.input_gamepad = false;

    let report = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::Windows,
        crate::RuntimeTargetMode::ClientRuntime,
    );

    assert_eq!(
        report.mouse_input,
        CapabilityStatus::FeatureDisabled {
            feature: "input-mouse"
        }
    );
    assert_eq!(
        report.mouse_buttons,
        CapabilityStatus::FeatureDisabled {
            feature: "input-mouse"
        }
    );
    assert_eq!(
        report.mouse_wheel,
        CapabilityStatus::FeatureDisabled {
            feature: "input-mouse"
        }
    );
    assert_eq!(
        report.raw_mouse_motion,
        CapabilityStatus::FeatureDisabled {
            feature: "input-mouse"
        }
    );
    assert_eq!(
        report.keyboard_input,
        CapabilityStatus::FeatureDisabled {
            feature: "input-keyboard"
        }
    );
    assert_eq!(
        report.keyboard_events,
        CapabilityStatus::FeatureDisabled {
            feature: "input-keyboard"
        }
    );
    assert_eq!(
        report.touch_input,
        CapabilityStatus::FeatureDisabled {
            feature: "input-touch"
        }
    );
    assert_eq!(
        report.touch_events,
        CapabilityStatus::FeatureDisabled {
            feature: "input-touch"
        }
    );
    assert_eq!(
        report.gesture_input,
        CapabilityStatus::FeatureDisabled {
            feature: "input-gestures"
        }
    );
    assert_eq!(
        report.gesture_events,
        CapabilityStatus::FeatureDisabled {
            feature: "input-gestures"
        }
    );
    assert_eq!(
        report.gamepad_input,
        CapabilityStatus::FeatureDisabled {
            feature: "input-gamepad"
        }
    );
    assert_eq!(
        report.gamepad_events,
        CapabilityStatus::FeatureDisabled {
            feature: "input-gamepad"
        }
    );
    assert_eq!(
        report.gamepad_rumble,
        CapabilityStatus::FeatureDisabled {
            feature: "input-gamepad"
        }
    );
}

#[test]
fn browser_gamepad_backend_requires_platform_web_gate() {
    let mut features = PlatformFeatureSelection::bevy_default_platform();
    features.gamepad_browser = true;
    features.platform_web = false;

    let report = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::Wasm,
        crate::RuntimeTargetMode::ClientRuntime,
    );

    assert_eq!(
        report.window_backend,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-web"
        }
    );
    assert_eq!(
        report.gamepad_input,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-web"
        }
    );
    assert_eq!(
        report.gamepad_events,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-web"
        }
    );
    assert_eq!(
        report.gamepad_rumble,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-web"
        }
    );
}
