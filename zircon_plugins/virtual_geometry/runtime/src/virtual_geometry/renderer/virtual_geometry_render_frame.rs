use zircon_runtime::core::framework::render::{RenderFrameExtract, RenderSceneSnapshot};
use zircon_runtime::core::math::UVec2;

#[derive(Clone, Debug)]
pub(in crate::virtual_geometry) struct VirtualGeometryRenderFrame {
    pub scene: RenderSceneSnapshot,
    pub extract: RenderFrameExtract,
    pub viewport_size: UVec2,
}

impl VirtualGeometryRenderFrame {
    pub fn from_extract(extract: RenderFrameExtract, viewport_size: impl Into<UVec2>) -> Self {
        let viewport_size = viewport_size.into();
        let scene = extract.to_scene_snapshot();
        Self {
            scene,
            extract,
            viewport_size: UVec2::new(viewport_size.x.max(1), viewport_size.y.max(1)),
        }
    }
}
