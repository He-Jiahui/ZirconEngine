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
