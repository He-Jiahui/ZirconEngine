use serde::{Deserialize, Serialize};

// TODO: [CR-WINDOW-0001] 确认这些标志是否应统一驱动主机行为：当前 Runtime
// 只读取 clears_hover，布局、重绘与关闭仍按事件种类分支处理，两处映射可能漂移。
/// 中性窗口事件的声明式影响提示，供主机映射到各自的脏标志与生命周期处理。
/// 它描述事件意图，不包含平台类型或具体 UI 树操作。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiWindowEventImpact {
    pub input_state_dirty: bool,
    pub layout_metrics_dirty: bool,
    pub clears_hover: bool,
    pub requests_redraw: bool,
    pub close_requested: bool,
}

impl UiWindowEventImpact {
    pub const fn clean() -> Self {
        Self {
            input_state_dirty: false,
            layout_metrics_dirty: false,
            clears_hover: false,
            requests_redraw: false,
            close_requested: false,
        }
    }

    pub const fn input_state() -> Self {
        Self {
            input_state_dirty: true,
            layout_metrics_dirty: false,
            clears_hover: false,
            requests_redraw: false,
            close_requested: false,
        }
    }

    pub const fn layout_metrics() -> Self {
        Self {
            input_state_dirty: false,
            layout_metrics_dirty: true,
            clears_hover: false,
            requests_redraw: false,
            close_requested: false,
        }
    }

    pub const fn redraw() -> Self {
        Self {
            input_state_dirty: false,
            layout_metrics_dirty: false,
            clears_hover: false,
            requests_redraw: true,
            close_requested: false,
        }
    }

    pub const fn close_requested() -> Self {
        Self {
            input_state_dirty: false,
            layout_metrics_dirty: false,
            clears_hover: false,
            requests_redraw: false,
            close_requested: true,
        }
    }

    pub const fn with_redraw(mut self) -> Self {
        self.requests_redraw = true;
        self
    }

    pub const fn with_hover_clear(mut self) -> Self {
        self.clears_hover = true;
        self
    }
}
