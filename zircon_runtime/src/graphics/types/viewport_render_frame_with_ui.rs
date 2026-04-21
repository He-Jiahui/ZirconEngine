use crate::ui::surface::UiRenderExtract;

use super::viewport_render_frame::ViewportRenderFrame;

impl ViewportRenderFrame {
    pub fn with_ui(mut self, ui: Option<UiRenderExtract>) -> Self {
        self.ui = ui;
        self
    }
}
