use std::sync::Arc;

use crate::core::framework::render::UiRenderSubmission;

use super::viewport_render_frame::ViewportRenderFrame;

impl ViewportRenderFrame {
    pub fn with_ui(mut self, ui: Option<Arc<UiRenderSubmission>>) -> Self {
        self.ui = ui;
        self
    }
}
