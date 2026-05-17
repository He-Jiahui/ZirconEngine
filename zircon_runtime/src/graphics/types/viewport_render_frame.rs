use crate::core::framework::render::{
    RenderFrameExtract, RenderPreparedRuntimeSidebands, RenderSceneSnapshot,
    RenderVirtualGeometryDebugSnapshot,
};
use crate::core::math::UVec2;
use zircon_runtime_interface::ui::surface::UiRenderExtract;

#[derive(Clone, Debug)]
pub struct ViewportRenderFrame {
    pub scene: RenderSceneSnapshot,
    pub extract: RenderFrameExtract,
    pub viewport_size: UVec2,
    /// Screen-space runtime UI payload selected for this viewport target.
    pub ui: Option<UiRenderExtract>,
    pub(crate) virtual_geometry_debug_snapshot: Option<RenderVirtualGeometryDebugSnapshot>,
    pub(crate) prepared_runtime_sidebands: RenderPreparedRuntimeSidebands,
}

impl ViewportRenderFrame {
    pub(crate) fn prepared_runtime_sidebands(&self) -> &RenderPreparedRuntimeSidebands {
        &self.prepared_runtime_sidebands
    }

    pub(crate) fn camera(&self) -> &crate::core::framework::render::ViewportCameraSnapshot {
        &self.extract.view.camera
    }

    pub(crate) fn meshes(&self) -> &[crate::core::framework::render::RenderMeshSnapshot] {
        &self.extract.geometry.meshes
    }

    pub(crate) fn sprites(&self) -> &[crate::core::framework::render::RenderSpriteSnapshot] {
        &self.extract.sprites.sprites
    }

    pub(crate) fn directional_lights(
        &self,
    ) -> &[crate::core::framework::render::RenderDirectionalLightSnapshot] {
        &self.extract.lighting.directional_lights
    }

    pub(crate) fn overlays(&self) -> &crate::core::framework::render::RenderOverlayExtract {
        &self.extract.debug.overlays
    }

    pub(crate) fn preview(&self) -> &crate::core::framework::render::PreviewEnvironmentExtract {
        &self.extract.post_process.preview
    }
}
