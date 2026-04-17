use zircon_math::UVec2;
use zircon_scene::RenderFrameExtract;

use super::editor_or_runtime_frame::EditorOrRuntimeFrame;

impl EditorOrRuntimeFrame {
    pub fn from_extract(extract: RenderFrameExtract, viewport_size: impl Into<UVec2>) -> Self {
        let viewport_size = viewport_size.into();
        let scene = extract.to_legacy_snapshot();
        Self {
            scene,
            extract,
            viewport_size: UVec2::new(viewport_size.x.max(1), viewport_size.y.max(1)),
            hybrid_gi_prepare: None,
            virtual_geometry_prepare: None,
        }
    }
}
