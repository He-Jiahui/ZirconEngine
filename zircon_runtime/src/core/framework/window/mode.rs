use serde::{Deserialize, Serialize};

use super::{WindowMonitorSelection, WindowVideoModeSelection};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowMode {
    #[default]
    Windowed,
    BorderlessFullscreen,
    BorderlessFullscreenOn(WindowMonitorSelection),
    Fullscreen,
    FullscreenOn {
        monitor: WindowMonitorSelection,
        video_mode: WindowVideoModeSelection,
    },
}

impl WindowMode {
    pub const fn borderless_fullscreen_on(monitor: WindowMonitorSelection) -> Self {
        Self::BorderlessFullscreenOn(monitor)
    }

    pub const fn fullscreen_on(
        monitor: WindowMonitorSelection,
        video_mode: WindowVideoModeSelection,
    ) -> Self {
        Self::FullscreenOn {
            monitor,
            video_mode,
        }
    }
}
