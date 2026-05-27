use super::super::*;

#[test]
fn window_input_and_host_backend_tokens_stay_stable() {
    assert_eq!(WindowBackend::Winit.as_str(), "winit");
    assert_eq!(WindowBackend::BrowserCanvas.as_str(), "browser_canvas");
    assert_eq!(WindowBackend::Headless.as_str(), "headless");
    assert_eq!(
        MonitorBackend::WinitMonitorHandles.as_str(),
        "winit_monitor_handles"
    );
    assert_eq!(
        MonitorBackend::BrowserScreenDetails.as_str(),
        "browser_screen_details"
    );
    assert_eq!(
        WindowEventBackend::WinitWindowEvents.as_str(),
        "winit_window_events"
    );
    assert_eq!(
        WindowEventBackend::BrowserWindowEvents.as_str(),
        "browser_window_events"
    );
    assert_eq!(
        WindowLifecycleBackend::WinitWindowEvents.as_str(),
        "winit_window_events"
    );
    assert_eq!(
        WindowLifecycleBackend::BrowserWindowEvents.as_str(),
        "browser_window_events"
    );
    assert_eq!(
        WindowMetricsBackend::WinitWindowEvents.as_str(),
        "winit_window_events"
    );
    assert_eq!(
        WindowMetricsBackend::BrowserResizeObserver.as_str(),
        "browser_resize_observer"
    );
    assert_eq!(ImeBackend::WinitIme.as_str(), "winit_ime");
    assert_eq!(ImeBackend::BrowserIme.as_str(), "browser_ime");
    assert_eq!(
        KeyboardEventBackend::WinitWindowEvents.as_str(),
        "winit_window_events"
    );
    assert_eq!(
        KeyboardEventBackend::BrowserKeyboardEvents.as_str(),
        "browser_keyboard_events"
    );
    assert_eq!(
        CursorBoundaryBackend::WinitWindowEvents.as_str(),
        "winit_window_events"
    );
    assert_eq!(
        CursorBoundaryBackend::BrowserPointerEvents.as_str(),
        "browser_pointer_events"
    );
    assert_eq!(
        CursorOptionsBackend::WinitWindowOptions.as_str(),
        "winit_window_options"
    );
    assert_eq!(
        CursorOptionsBackend::BrowserCursorOptions.as_str(),
        "browser_cursor_options"
    );
    assert_eq!(
        FileDragDropBackend::WinitWindowEvents.as_str(),
        "winit_window_events"
    );
    assert_eq!(
        FileDragDropBackend::BrowserDragEvents.as_str(),
        "browser_drag_events"
    );
}

#[test]
fn input_source_backend_tokens_stay_stable() {
    assert_eq!(
        MouseButtonBackend::WinitWindowEvents.as_str(),
        "winit_window_events"
    );
    assert_eq!(
        MouseButtonBackend::BrowserPointerEvents.as_str(),
        "browser_pointer_events"
    );
    assert_eq!(
        MouseWheelBackend::WinitWindowEvents.as_str(),
        "winit_window_events"
    );
    assert_eq!(
        MouseWheelBackend::BrowserWheelEvents.as_str(),
        "browser_wheel_events"
    );
    assert_eq!(
        TouchEventBackend::WinitWindowEvents.as_str(),
        "winit_window_events"
    );
    assert_eq!(
        TouchEventBackend::BrowserTouchEvents.as_str(),
        "browser_touch_events"
    );
    assert_eq!(
        GestureEventBackend::WinitWindowEvents.as_str(),
        "winit_window_events"
    );
    assert_eq!(
        GestureEventBackend::BrowserGestureEvents.as_str(),
        "browser_gesture_events"
    );
    assert_eq!(
        PointerPositionBackend::WinitWindowEvents.as_str(),
        "winit_window_events"
    );
    assert_eq!(
        PointerPositionBackend::BrowserPointerEvents.as_str(),
        "browser_pointer_events"
    );
    assert_eq!(
        RawMouseMotionBackend::WinitDeviceEvents.as_str(),
        "winit_device_events"
    );
    assert_eq!(
        RawMouseMotionBackend::BrowserPointerLock.as_str(),
        "browser_pointer_lock"
    );
    assert_eq!(
        InputBackend::WinitWindowEvents.as_str(),
        "winit_window_events"
    );
    assert_eq!(InputBackend::BrowserEvents.as_str(), "browser_events");
    assert_eq!(InputBackend::SyntheticOnly.as_str(), "synthetic_only");
}

#[test]
fn gamepad_linux_and_event_loop_tokens_stay_stable() {
    assert_eq!(GamepadBackend::Gilrs.as_str(), "gilrs");
    assert_eq!(
        GamepadBackend::BrowserGamepadApi.as_str(),
        "browser_gamepad_api"
    );
    assert_eq!(
        GamepadEventBackend::GilrsEventPolling.as_str(),
        "gilrs_event_polling"
    );
    assert_eq!(
        GamepadEventBackend::BrowserGamepadApiPolling.as_str(),
        "browser_gamepad_api_polling"
    );
    assert_eq!(
        GamepadRumbleBackend::GilrsForceFeedback.as_str(),
        "gilrs_force_feedback"
    );
    assert_eq!(
        GamepadRumbleBackend::BrowserGamepadHaptics.as_str(),
        "browser_gamepad_haptics"
    );
    assert_eq!(LinuxWindowProtocol::X11.as_str(), "x11");
    assert_eq!(LinuxWindowProtocol::Wayland.as_str(), "wayland");
    assert_eq!(EventLoopPolicy::Game.as_str(), "game");
    assert_eq!(EventLoopPolicy::DesktopApp.as_str(), "desktop_app");
    assert_eq!(EventLoopPolicy::Mobile.as_str(), "mobile");
    assert_eq!(EventLoopPolicy::Continuous.as_str(), "continuous");
    assert_eq!(EventLoopPolicy::Headless.as_str(), "headless");
}

#[test]
fn diagnostic_status_prefixes_stay_stable() {
    let report = PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform())
        .report(
            PlatformTarget::Windows,
            crate::RuntimeTargetMode::ClientRuntime,
        );
    let lines = report.diagnostic_lines();

    assert!(lines
        .iter()
        .any(|line| line == "platform.window_backend=supported:winit"));
    assert!(lines
        .iter()
        .any(|line| line == "platform.gesture_events=feature_disabled:input-gestures"));
    assert!(lines.iter().any(|line| line
        == "platform.cursor_options=unavailable:desktop cursor options host-request backend is not implemented yet"));
    assert!(!lines.iter().any(|line| line.contains("Supported(")));
    assert!(!lines.iter().any(|line| line.contains("FeatureDisabled")));
    assert!(!lines.iter().any(|line| line.contains("Unavailable")));
}
