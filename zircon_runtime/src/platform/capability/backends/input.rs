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
