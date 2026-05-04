use crate::core::framework::render::RenderFrameExtract;
use crate::core::math::UVec2;

use super::viewport_render_frame::ViewportRenderFrame;

impl ViewportRenderFrame {
    pub fn from_extract(extract: RenderFrameExtract, viewport_size: impl Into<UVec2>) -> Self {
        let viewport_size = viewport_size.into();
        let scene = extract.to_scene_snapshot();
        Self {
            scene,
            extract,
            viewport_size: UVec2::new(viewport_size.x.max(1), viewport_size.y.max(1)),
            ui: None,
            virtual_geometry_debug_snapshot: None,
            prepared_runtime_sidebands: Default::default(),
        }
    }
}
