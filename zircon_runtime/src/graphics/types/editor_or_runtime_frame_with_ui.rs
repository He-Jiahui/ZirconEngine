use crate::ui::surface::UiRenderExtract;

use super::editor_or_runtime_frame::EditorOrRuntimeFrame;

impl EditorOrRuntimeFrame {
    pub fn with_ui(mut self, ui: Option<UiRenderExtract>) -> Self {
        self.ui = ui;
        self
    }
}
