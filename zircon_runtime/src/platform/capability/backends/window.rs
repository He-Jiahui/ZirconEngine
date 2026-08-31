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
