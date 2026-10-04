use std::sync::Arc;

use crate::core::framework::render::UiRenderSubmission;

use super::viewport_render_frame::ViewportRenderFrame;

impl ViewportRenderFrame {
    /// 将终端屏幕空间 UI 绑定到此帧；相机栈循环只把它交给获选的终端提交。
    pub fn with_ui(mut self, ui: Option<Arc<UiRenderSubmission>>) -> Self {
        self.ui = ui;
        self
    }
}
