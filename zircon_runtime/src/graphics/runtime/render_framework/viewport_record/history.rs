use crate::runtime::ViewportFrameHistory;

use super::viewport_record::ViewportRecord;

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn history(
        &self,
    ) -> Option<&ViewportFrameHistory> {
        self.history.as_ref()
    }

    pub(in crate::graphics::runtime::render_framework) fn history_mut(
        &mut self,
    ) -> Option<&mut ViewportFrameHistory> {
        self.history.as_mut()
    }

    pub(in crate::graphics::runtime::render_framework) fn replace_history(
        &mut self,
        history: ViewportFrameHistory,
    ) {
        self.history = Some(history);
    }

    pub(in crate::graphics::runtime::render_framework) fn into_history(
        self,
    ) -> Option<ViewportFrameHistory> {
        self.history
    }
}
