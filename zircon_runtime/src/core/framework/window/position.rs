use serde::{Deserialize, Serialize};

use super::WindowMonitorSelection;

/// 原生窗口创建前的定位意图；居中由 App 根据可用显示器信息计算。
/// 无法取得显示器时保留后端自动定位，不把位置坐标误作已观测结果。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowPosition {
    #[default]
    Automatic,
    Centered,
    CenteredOn(WindowMonitorSelection),
    At {
        x: i32,
        y: i32,
    },
}

impl WindowPosition {
    pub const fn centered_on(monitor: WindowMonitorSelection) -> Self {
        Self::CenteredOn(monitor)
    }
}
