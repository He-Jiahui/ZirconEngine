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
            output_target: Default::default(),
            previous_motion_vector_camera: None,
            previous_motion_vector_object_history: None,
            frame_visibility: None,
            virtual_geometry_debug_snapshot: None,
            prepared_runtime_sidebands: Default::default(),
        }
    }
}
