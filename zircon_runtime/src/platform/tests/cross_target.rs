use super::super::*;

#[test]
fn mobile_and_browser_capabilities_are_explicit() {
    let mobile = PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform())
        .report(
            PlatformTarget::Android,
            crate::RuntimeTargetMode::ClientRuntime,
        );

    assert_eq!(mobile.event_loop_policy, EventLoopPolicy::Mobile);
    assert_eq!(
        mobile.touch_input,
        CapabilityStatus::Supported(InputBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.monitor_inventory,
        CapabilityStatus::Supported(MonitorBackend::WinitMonitorHandles)
    );
    assert_eq!(
        mobile.window_events,
        CapabilityStatus::Supported(WindowEventBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.window_lifecycle,
        CapabilityStatus::Supported(WindowLifecycleBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.window_metrics,
        CapabilityStatus::Supported(WindowMetricsBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.ime,
        CapabilityStatus::Unavailable {
            reason: "mobile ime host backend is not implemented yet"
        }
    );
    assert_eq!(
        mobile.keyboard_events,
        CapabilityStatus::Supported(KeyboardEventBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.cursor_boundary,
        CapabilityStatus::Supported(CursorBoundaryBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.cursor_options,
        CapabilityStatus::Unavailable {
            reason: "mobile cursor options host-request backend is not implemented yet"
        }
    );
    assert_eq!(
        mobile.mouse_buttons,
        CapabilityStatus::Supported(MouseButtonBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.mouse_wheel,
        CapabilityStatus::Supported(MouseWheelBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.touch_events,
        CapabilityStatus::Supported(TouchEventBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.gesture_events,
        CapabilityStatus::FeatureDisabled {
            feature: "input-gestures"
        }
    );
    assert_eq!(
        mobile.pointer_position,
        CapabilityStatus::Supported(PointerPositionBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.raw_mouse_motion,
        CapabilityStatus::Unavailable {
            reason: "mobile raw mouse motion host backend is not implemented yet"
        }
    );
    assert_eq!(
        mobile.gamepad_input,
        CapabilityStatus::Unavailable {
            reason: "mobile gamepad host backend is not implemented yet"
        }
    );
    assert_eq!(
        mobile.gamepad_events,
        CapabilityStatus::Unavailable {
            reason: "mobile gamepad event host backend is not implemented yet"
        }
    );
    assert_eq!(
        mobile.gamepad_rumble,
        CapabilityStatus::Unavailable {
            reason: "mobile gamepad rumble host backend is not implemented yet"
        }
    );
    assert_eq!(
        mobile.file_drag_drop,
        CapabilityStatus::Unavailable {
            reason: "mobile file drag/drop host backend is not implemented yet"
        }
    );

    let browser = PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform())
        .report(
            PlatformTarget::Wasm,
            crate::RuntimeTargetMode::ClientRuntime,
        );

    assert_eq!(
        browser.window_backend,
        CapabilityStatus::Supported(WindowBackend::BrowserCanvas)
    );
    assert_eq!(
        browser.monitor_inventory,
        CapabilityStatus::Unavailable {
            reason: "browser monitor inventory host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.window_events,
        CapabilityStatus::Unavailable {
            reason: "browser window event host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.window_lifecycle,
        CapabilityStatus::Unavailable {
            reason: "browser window lifecycle host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.window_metrics,
        CapabilityStatus::Unavailable {
            reason: "browser window metrics host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.ime,
        CapabilityStatus::Unavailable {
            reason: "browser ime host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.keyboard_events,
        CapabilityStatus::Unavailable {
            reason: "browser keyboard event host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.cursor_boundary,
        CapabilityStatus::Unavailable {
            reason: "browser cursor boundary host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.cursor_options,
        CapabilityStatus::Unavailable {
            reason: "browser cursor options host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.mouse_buttons,
        CapabilityStatus::Unavailable {
            reason: "browser mouse button host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.mouse_wheel,
        CapabilityStatus::Unavailable {
            reason: "browser mouse wheel host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.touch_events,
        CapabilityStatus::Unavailable {
            reason: "browser touch event host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.gesture_events,
        CapabilityStatus::FeatureDisabled {
            feature: "input-gestures"
        }
    );
    assert_eq!(
        browser.pointer_position,
        CapabilityStatus::Unavailable {
            reason: "browser pointer position host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.raw_mouse_motion,
        CapabilityStatus::Unavailable {
            reason: "browser raw mouse motion host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.gamepad_input,
        CapabilityStatus::FeatureDisabled {
            feature: "gamepad-browser"
        }
    );
    assert_eq!(
        browser.gamepad_events,
        CapabilityStatus::FeatureDisabled {
            feature: "gamepad-browser"
        }
    );
    assert_eq!(
        browser.gamepad_rumble,
        CapabilityStatus::FeatureDisabled {
            feature: "gamepad-browser"
        }
    );
    assert_eq!(
        browser.file_drag_drop,
        CapabilityStatus::Unavailable {
            reason: "browser file drag/drop host backend is not implemented yet"
        }
    );
}
