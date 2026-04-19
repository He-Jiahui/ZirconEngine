use crate::ui::PublicRuntimeFrame;

use super::editor_or_runtime_frame::EditorOrRuntimeFrame;

impl From<PublicRuntimeFrame> for EditorOrRuntimeFrame {
    fn from(frame: PublicRuntimeFrame) -> Self {
        let scene = frame.extract.to_scene_snapshot();
        Self {
            scene,
            extract: frame.extract,
            viewport_size: frame.viewport_size,
            ui: frame.ui,
            hybrid_gi_prepare: None,
            hybrid_gi_resolve_runtime: None,
            virtual_geometry_prepare: None,
        }
    }
}
