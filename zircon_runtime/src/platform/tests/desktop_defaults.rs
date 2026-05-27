use super::super::*;

#[test]
fn client_desktop_default_platform_declares_window_input_and_gilrs() {
    let report = PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform())
        .report(
            PlatformTarget::Windows,
            crate::RuntimeTargetMode::ClientRuntime,
        );

    assert_eq!(
        report.window_backend,
        CapabilityStatus::Supported(WindowBackend::Winit)
    );
    assert_eq!(
        report.monitor_inventory,
        CapabilityStatus::Supported(MonitorBackend::WinitMonitorHandles)
    );
    assert_eq!(
        report.window_events,
        CapabilityStatus::Supported(WindowEventBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.window_lifecycle,
        CapabilityStatus::Supported(WindowLifecycleBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.window_metrics,
        CapabilityStatus::Supported(WindowMetricsBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.ime,
        CapabilityStatus::Supported(ImeBackend::WinitIme)
    );
    assert_eq!(
        report.keyboard_events,
        CapabilityStatus::Supported(KeyboardEventBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.cursor_boundary,
        CapabilityStatus::Supported(CursorBoundaryBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.cursor_options,
        CapabilityStatus::Unavailable {
            reason: "desktop cursor options host-request backend is not implemented yet"
        }
    );
    assert_eq!(
        report.mouse_buttons,
        CapabilityStatus::Supported(MouseButtonBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.mouse_wheel,
        CapabilityStatus::Supported(MouseWheelBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.touch_events,
        CapabilityStatus::Supported(TouchEventBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.gesture_events,
        CapabilityStatus::FeatureDisabled {
            feature: "input-gestures"
        }
    );
    assert_eq!(
        report.pointer_position,
        CapabilityStatus::Supported(PointerPositionBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.raw_mouse_motion,
        CapabilityStatus::Supported(RawMouseMotionBackend::WinitDeviceEvents)
    );
    assert_eq!(report.event_loop_policy, EventLoopPolicy::Game);
    assert_eq!(
        report.mouse_input,
        CapabilityStatus::Supported(InputBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.keyboard_input,
        CapabilityStatus::Supported(InputBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.touch_input,
        CapabilityStatus::Supported(InputBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.gamepad_input,
        CapabilityStatus::Supported(GamepadBackend::Gilrs)
    );
    assert_eq!(
        report.gamepad_events,
        CapabilityStatus::Supported(GamepadEventBackend::GilrsEventPolling)
    );
    assert_eq!(
        report.gamepad_rumble,
        CapabilityStatus::Supported(GamepadRumbleBackend::GilrsForceFeedback)
    );
    assert_eq!(
        report.file_drag_drop,
        CapabilityStatus::Supported(FileDragDropBackend::WinitWindowEvents)
    );
}
