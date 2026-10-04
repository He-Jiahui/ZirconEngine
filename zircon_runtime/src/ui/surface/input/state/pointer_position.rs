use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{dispatch::UiPointerSource, layout::UiPoint};

use super::UiSurfaceInputState;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiSurfacePointerPositionState {
    pub point: UiPoint,
}

impl UiSurfaceInputState {
    /// 保留可见光标的最近位置供窗口失焦/离开时合成取消事件；触屏接触点不代表光标。
    pub fn record_pointer_position(&mut self, source: UiPointerSource, point: UiPoint) {
        if source.is_touch_like() {
            return;
        }

        self.last_cursor_point = Some(UiSurfacePointerPositionState { point });
    }

    pub fn last_cursor_point(&self) -> Option<UiPoint> {
        self.last_cursor_point.map(|state| state.point)
    }

    pub fn clear_last_cursor_point(&mut self) {
        self.last_cursor_point = None;
    }
}
