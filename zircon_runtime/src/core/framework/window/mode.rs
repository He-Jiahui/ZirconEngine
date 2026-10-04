use serde::{Deserialize, Serialize};

use super::{WindowMonitorSelection, WindowVideoModeSelection};

/// 启动时的全屏意图；App 在创建原生窗口时解析显示器与视频模式。
/// 找不到独占模式时，入口会按当前后端策略退回无边框模式。
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
