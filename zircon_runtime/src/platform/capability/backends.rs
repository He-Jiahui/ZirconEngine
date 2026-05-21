#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowBackend {
    Winit,
    BrowserCanvas,
    Headless,
}

impl WindowBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Winit => "winit",
            Self::BrowserCanvas => "browser_canvas",
            Self::Headless => "headless",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonitorBackend {
    WinitMonitorHandles,
    BrowserScreenDetails,
}

impl MonitorBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WinitMonitorHandles => "winit_monitor_handles",
            Self::BrowserScreenDetails => "browser_screen_details",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowEventBackend {
    WinitWindowEvents,
    BrowserWindowEvents,
}

impl WindowEventBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WinitWindowEvents => "winit_window_events",
            Self::BrowserWindowEvents => "browser_window_events",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowLifecycleBackend {
    WinitWindowEvents,
    BrowserWindowEvents,
}

impl WindowLifecycleBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WinitWindowEvents => "winit_window_events",
            Self::BrowserWindowEvents => "browser_window_events",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowMetricsBackend {
    WinitWindowEvents,
    BrowserResizeObserver,
}

impl WindowMetricsBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WinitWindowEvents => "winit_window_events",
            Self::BrowserResizeObserver => "browser_resize_observer",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImeBackend {
    WinitIme,
    BrowserIme,
}

impl ImeBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WinitIme => "winit_ime",
            Self::BrowserIme => "browser_ime",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyboardEventBackend {
    WinitWindowEvents,
    BrowserKeyboardEvents,
}

impl KeyboardEventBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WinitWindowEvents => "winit_window_events",
            Self::BrowserKeyboardEvents => "browser_keyboard_events",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorBoundaryBackend {
    WinitWindowEvents,
    BrowserPointerEvents,
}

impl CursorBoundaryBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WinitWindowEvents => "winit_window_events",
            Self::BrowserPointerEvents => "browser_pointer_events",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorOptionsBackend {
    WinitWindowOptions,
    BrowserCursorOptions,
}

impl CursorOptionsBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WinitWindowOptions => "winit_window_options",
            Self::BrowserCursorOptions => "browser_cursor_options",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButtonBackend {
    WinitWindowEvents,
    BrowserPointerEvents,
}

impl MouseButtonBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WinitWindowEvents => "winit_window_events",
            Self::BrowserPointerEvents => "browser_pointer_events",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseWheelBackend {
    WinitWindowEvents,
    BrowserWheelEvents,
}

impl MouseWheelBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WinitWindowEvents => "winit_window_events",
            Self::BrowserWheelEvents => "browser_wheel_events",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchEventBackend {
    WinitWindowEvents,
    BrowserTouchEvents,
}

impl TouchEventBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WinitWindowEvents => "winit_window_events",
            Self::BrowserTouchEvents => "browser_touch_events",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GestureEventBackend {
    WinitWindowEvents,
    BrowserGestureEvents,
}

impl GestureEventBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WinitWindowEvents => "winit_window_events",
            Self::BrowserGestureEvents => "browser_gesture_events",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerPositionBackend {
    WinitWindowEvents,
    BrowserPointerEvents,
}

impl PointerPositionBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WinitWindowEvents => "winit_window_events",
            Self::BrowserPointerEvents => "browser_pointer_events",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawMouseMotionBackend {
    WinitDeviceEvents,
    BrowserPointerLock,
}

impl RawMouseMotionBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WinitDeviceEvents => "winit_device_events",
            Self::BrowserPointerLock => "browser_pointer_lock",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputBackend {
    WinitWindowEvents,
    BrowserEvents,
    SyntheticOnly,
}

impl InputBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WinitWindowEvents => "winit_window_events",
            Self::BrowserEvents => "browser_events",
            Self::SyntheticOnly => "synthetic_only",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamepadBackend {
    Gilrs,
    BrowserGamepadApi,
}

impl GamepadBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Gilrs => "gilrs",
            Self::BrowserGamepadApi => "browser_gamepad_api",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamepadEventBackend {
    GilrsEventPolling,
    BrowserGamepadApiPolling,
}

impl GamepadEventBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GilrsEventPolling => "gilrs_event_polling",
            Self::BrowserGamepadApiPolling => "browser_gamepad_api_polling",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamepadRumbleBackend {
    GilrsForceFeedback,
    BrowserGamepadHaptics,
}

impl GamepadRumbleBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GilrsForceFeedback => "gilrs_force_feedback",
            Self::BrowserGamepadHaptics => "browser_gamepad_haptics",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileDragDropBackend {
    WinitWindowEvents,
    BrowserDragEvents,
}

impl FileDragDropBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WinitWindowEvents => "winit_window_events",
            Self::BrowserDragEvents => "browser_drag_events",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinuxWindowProtocol {
    X11,
    Wayland,
}

impl LinuxWindowProtocol {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::X11 => "x11",
            Self::Wayland => "wayland",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventLoopPolicy {
    Game,
    DesktopApp,
    Mobile,
    Continuous,
    Headless,
}

impl EventLoopPolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Game => "game",
            Self::DesktopApp => "desktop_app",
            Self::Mobile => "mobile",
            Self::Continuous => "continuous",
            Self::Headless => "headless",
        }
    }
}
