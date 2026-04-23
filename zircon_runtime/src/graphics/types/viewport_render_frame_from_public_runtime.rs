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
            hybrid_gi_prepare: None,
            hybrid_gi_scene_prepare: None,
            hybrid_gi_resolve_runtime: None,
            virtual_geometry_cluster_selections: None,
            virtual_geometry_cluster_selections_source: None,
            virtual_geometry_prepare: None,
            virtual_geometry_debug_snapshot: None,
        }
    }
}
