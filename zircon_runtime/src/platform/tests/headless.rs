use super::super::*;

#[test]
fn server_runtime_stays_headless() {
    let report = PlatformCapabilityMatrix::new(PlatformFeatureSelection::headless()).report(
        PlatformTarget::Linux,
        crate::RuntimeTargetMode::ServerRuntime,
    );

    assert_eq!(
        report.window_backend,
        CapabilityStatus::Supported(WindowBackend::Headless)
    );
    assert_eq!(
        report.monitor_inventory,
        CapabilityStatus::Unavailable {
            reason: "headless target has no monitor inventory backend"
        }
    );
    assert_eq!(
        report.window_events,
        CapabilityStatus::Unavailable {
            reason: "headless target has no window event host backend"
        }
    );
    assert_eq!(
        report.window_lifecycle,
        CapabilityStatus::Unavailable {
            reason: "headless target has no window lifecycle host backend"
        }
    );
    assert_eq!(
        report.window_metrics,
        CapabilityStatus::Unavailable {
            reason: "headless target has no window metrics host backend"
        }
    );
    assert_eq!(
        report.ime,
        CapabilityStatus::Unavailable {
            reason: "headless target has no ime host backend"
        }
    );
    assert_eq!(
        report.keyboard_events,
        CapabilityStatus::Unavailable {
            reason: "headless target has no keyboard event host backend"
        }
    );
    assert_eq!(
        report.cursor_boundary,
        CapabilityStatus::Unavailable {
            reason: "headless target has no cursor boundary host backend"
        }
    );
    assert_eq!(
        report.cursor_options,
        CapabilityStatus::Unavailable {
            reason: "headless target has no cursor options host backend"
        }
    );
    assert_eq!(
        report.mouse_buttons,
        CapabilityStatus::Unavailable {
            reason: "headless target has no mouse button host backend"
        }
    );
    assert_eq!(
        report.mouse_wheel,
        CapabilityStatus::Unavailable {
            reason: "headless target has no mouse wheel host backend"
        }
    );
    assert_eq!(
        report.touch_events,
        CapabilityStatus::Unavailable {
            reason: "headless target has no touch event host backend"
        }
    );
    assert_eq!(
        report.gesture_events,
        CapabilityStatus::Unavailable {
            reason: "headless target has no gesture event host backend"
        }
    );
    assert_eq!(
        report.pointer_position,
        CapabilityStatus::Unavailable {
            reason: "headless target has no pointer position host backend"
        }
    );
    assert_eq!(
        report.raw_mouse_motion,
        CapabilityStatus::Unavailable {
            reason: "headless target has no raw mouse motion host backend"
        }
    );
    assert_eq!(report.event_loop_policy, EventLoopPolicy::Headless);
    assert_eq!(
        report.mouse_input,
        CapabilityStatus::FeatureDisabled {
            feature: "input-mouse"
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
    assert_eq!(
        report.file_drag_drop,
        CapabilityStatus::Unavailable {
            reason: "headless target has no file drag/drop host backend"
        }
    );
}
