use crate::core::framework::render::{
    RenderFrameExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
};
use crate::core::math::UVec2;

use super::viewport_render_frame::ViewportRenderFrame;

impl ViewportRenderFrame {
    pub fn from_snapshot(scene: RenderSceneSnapshot, viewport_size: impl Into<UVec2>) -> Self {
        Self::from_extract(
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(0), scene),
            viewport_size,
        )
    }
}
