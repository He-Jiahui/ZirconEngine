//! 视口记录维护按相机键分区的历史租约，表面更换及销毁时由上层统一释放。
use crate::graphics::runtime::ViewportFrameHistory;

use super::{viewport_record::ViewportRecord, ViewportCameraHistoryKey};

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn history(
        &self,
        key: &ViewportCameraHistoryKey,
    ) -> Option<&ViewportFrameHistory> {
        self.camera_histories.get(key)
    }

    pub(in crate::graphics::runtime::render_framework) fn history_mut(
        &mut self,
        key: &ViewportCameraHistoryKey,
    ) -> Option<&mut ViewportFrameHistory> {
        self.camera_histories.get_mut(key)
    }

    pub(in crate::graphics::runtime::render_framework) fn replace_history(
        &mut self,
        key: ViewportCameraHistoryKey,
        history: ViewportFrameHistory,
    ) {
        self.camera_histories.insert(key, history);
    }

    pub(in crate::graphics::runtime::render_framework) fn into_histories(
        self,
    ) -> impl Iterator<Item = ViewportFrameHistory> {
        self.camera_histories.into_values()
    }
}

#[cfg(test)]
#[path = "tests/history.rs"]
mod tests;
