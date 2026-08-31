use super::super::*;

#[test]
fn capability_status_supported_predicate_only_accepts_supported_values() {
    assert!(CapabilityStatus::Supported(WindowBackend::Winit).is_supported());
    assert!(!CapabilityStatus::<WindowBackend>::FeatureDisabled {
        feature: "platform-window"
    }
    .is_supported());
    assert!(!CapabilityStatus::<WindowBackend>::Unavailable {
        reason: "window backend unavailable"
    }
    .is_supported());
}

#[test]
fn default_desktop_report_support_predicate_tracks_implemented_host_paths() {
    let report = PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform())
        .report(
            PlatformTarget::Windows,
            crate::core::framework::platform::RuntimeTargetMode::ClientRuntime,
        );

    assert!(report.window_backend.is_supported());
    assert!(report.monitor_inventory.is_supported());
    assert!(report.window_events.is_supported());
    assert!(report.window_lifecycle.is_supported());
    assert!(report.window_metrics.is_supported());
    assert!(report.ime.is_supported());
    assert!(report.keyboard_events.is_supported());
    assert!(report.cursor_boundary.is_supported());
    assert!(report.cursor_options.is_supported());
    assert!(report.mouse_buttons.is_supported());
    assert!(report.mouse_wheel.is_supported());
    assert!(report.touch_events.is_supported());
    assert!(report.pointer_position.is_supported());
    assert!(report.raw_mouse_motion.is_supported());
    assert!(report.mouse_input.is_supported());
    assert!(report.keyboard_input.is_supported());
    assert!(report.touch_input.is_supported());
    assert!(report.gamepad_input.is_supported());
    assert!(report.gamepad_events.is_supported());
    assert!(report.file_drag_drop.is_supported());

    assert!(!report.gesture_events.is_supported());
    assert!(!report.gesture_input.is_supported());
    assert!(report.gamepad_rumble.is_supported());
    assert!(!report.linux_x11.is_supported());
    assert!(!report.linux_wayland.is_supported());
}

#[test]
fn headless_fixture_support_predicate_keeps_disabled_input_unavailable() {
    let report = PlatformCapabilityMatrix::new(PlatformFeatureSelection::headless()).report(
        PlatformTarget::Linux,
        crate::core::framework::platform::RuntimeTargetMode::ServerRuntime,
    );

    assert!(report.window_backend.is_supported());
    assert!(!report.monitor_inventory.is_supported());
    assert!(!report.window_events.is_supported());
    assert!(!report.window_lifecycle.is_supported());
    assert!(!report.window_metrics.is_supported());
    assert!(!report.ime.is_supported());
    assert!(!report.keyboard_events.is_supported());
    assert!(!report.mouse_buttons.is_supported());
    assert!(!report.mouse_wheel.is_supported());
    assert!(!report.touch_events.is_supported());
    assert!(!report.gesture_events.is_supported());
    assert!(!report.pointer_position.is_supported());
    assert!(!report.raw_mouse_motion.is_supported());
    assert!(!report.mouse_input.is_supported());
    assert!(!report.keyboard_input.is_supported());
    assert!(!report.touch_input.is_supported());
    assert!(!report.gesture_input.is_supported());
    assert!(!report.gamepad_input.is_supported());
    assert!(!report.gamepad_events.is_supported());
    assert!(!report.gamepad_rumble.is_supported());
    assert!(!report.file_drag_drop.is_supported());
}

#[test]
fn synthetic_input_backend_counts_as_supported_when_input_gate_is_enabled() {
    let report = PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform())
        .report(
            PlatformTarget::Headless,
            crate::core::framework::platform::RuntimeTargetMode::ClientRuntime,
        );

    assert!(report.mouse_input.is_supported());
    assert!(report.keyboard_input.is_supported());
    assert!(report.touch_input.is_supported());
    assert!(!report.gesture_input.is_supported());
    assert!(!report.mouse_buttons.is_supported());
    assert!(!report.raw_mouse_motion.is_supported());
    assert!(!report.gamepad_input.is_supported());
    assert!(!report.file_drag_drop.is_supported());
}

#[test]
fn disabled_platform_config_masks_candidate_capabilities_and_injected_preferences() {
    let config = PlatformConfig {
        enabled: false,
        target: PlatformTarget::Windows,
        target_mode: crate::core::framework::platform::RuntimeTargetMode::ClientRuntime,
        features: PlatformFeatureSelection::bevy_default_platform(),
    };

    let report = config.planning_capability_report_with_preference_storage_backend(
        crate::core::framework::platform::PreferenceStorageBackendKind::AtomicFile,
    );

    assert_eq!(
        report.window_backend,
        CapabilityStatus::FeatureDisabled {
            feature: "platform"
        }
    );
    assert_eq!(
        report.persistent_preferences,
        CapabilityStatus::FeatureDisabled {
            feature: "platform"
        }
    );
    assert_eq!(report.event_loop_policy, EventLoopPolicy::Headless);
    assert!(
        report
            .diagnostic_lines()
            .iter()
            .all(|line| !line.contains("=supported:")),
        "disabled platform config must not publish a supported runtime capability"
    );
}
