use zircon_math::UVec2;
use zircon_scene::{RenderFrameExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle};

use super::editor_or_runtime_frame::EditorOrRuntimeFrame;

impl EditorOrRuntimeFrame {
    pub fn from_snapshot(scene: RenderSceneSnapshot, viewport_size: impl Into<UVec2>) -> Self {
        let viewport_size = viewport_size.into();
        let extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(0), scene.clone());
        Self {
            scene,
            extract,
            viewport_size: UVec2::new(viewport_size.x.max(1), viewport_size.y.max(1)),
            hybrid_gi_prepare: None,
            virtual_geometry_prepare: None,
        }
    }
}
