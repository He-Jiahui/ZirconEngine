use crate::core::framework::render::{
    RenderFrameExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
};
use crate::core::math::UVec2;

use super::viewport_render_frame::ViewportRenderFrame;

impl ViewportRenderFrame {
    pub fn from_snapshot(scene: RenderSceneSnapshot, viewport_size: impl Into<UVec2>) -> Self {
        let viewport_size = viewport_size.into();
        let extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(0), scene.clone());
        Self {
            scene,
            extract,
            viewport_size: UVec2::new(viewport_size.x.max(1), viewport_size.y.max(1)),
            ui: None,
            output_target: Default::default(),
            previous_motion_vector_camera: None,
            previous_motion_vector_object_history: None,
            virtual_geometry_debug_snapshot: None,
            prepared_runtime_sidebands: Default::default(),
        }
    }
}
