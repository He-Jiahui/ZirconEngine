use super::super::super::{EnvironmentExtract, RenderOverlayExtract};
use super::super::virtual_geometry::RenderVirtualGeometryDebugState;
use super::{PreviewEnvironmentExtract, RenderSceneGeometryExtract};

#[derive(Clone, Debug, PartialEq)]
pub struct SceneViewportRenderPacket {
    pub scene: RenderSceneGeometryExtract,
    pub overlays: RenderOverlayExtract,
    pub environment: EnvironmentExtract,
    pub preview: PreviewEnvironmentExtract,
    pub virtual_geometry_debug: Option<RenderVirtualGeometryDebugState>,
}
