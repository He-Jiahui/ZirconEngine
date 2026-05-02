use crate::ui::PublicRuntimeFrame;

use super::viewport_render_frame::ViewportRenderFrame;

impl From<PublicRuntimeFrame> for ViewportRenderFrame {
    fn from(frame: PublicRuntimeFrame) -> Self {
        let scene = frame.extract.to_scene_snapshot();
        Self {
            scene,
            extract: frame.extract,
            viewport_size: frame.viewport_size,
            ui: frame.ui,
            virtual_geometry_debug_snapshot: None,
        }
    }
}
