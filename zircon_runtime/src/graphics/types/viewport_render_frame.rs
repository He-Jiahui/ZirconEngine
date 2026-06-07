use crate::core::framework::render::{
    RenderFrameExtract, RenderPreparedRuntimeSidebands, RenderSceneSnapshot,
    RenderVirtualGeometryDebugSnapshot, ViewportCameraSnapshot,
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
    pub(crate) output_target: super::ViewportRenderOutputTarget,
    pub(crate) previous_motion_vector_camera: Option<ViewportCameraSnapshot>,
    pub(crate) previous_motion_vector_object_history:
        Option<super::ViewportMotionVectorObjectHistory>,
    pub(crate) virtual_geometry_debug_snapshot: Option<RenderVirtualGeometryDebugSnapshot>,
    pub(crate) prepared_runtime_sidebands: RenderPreparedRuntimeSidebands,
}

impl ViewportRenderFrame {
    pub(crate) fn prepared_runtime_sidebands(&self) -> &RenderPreparedRuntimeSidebands {
        &self.prepared_runtime_sidebands
    }

    pub(crate) fn output_target(&self) -> super::ViewportRenderOutputTarget {
        self.output_target
    }

    pub(crate) fn texture_writeback_plan(
        &self,
        target_format: Option<&str>,
    ) -> super::ViewportTextureWritebackPlan {
        self.output_target.writeback_plan(target_format)
    }

    pub(crate) fn camera(&self) -> &crate::core::framework::render::ViewportCameraSnapshot {
        &self.extract.view.camera
    }

    pub(crate) fn previous_motion_vector_camera(&self) -> Option<&ViewportCameraSnapshot> {
        self.previous_motion_vector_camera.as_ref()
    }

    pub(crate) fn previous_motion_vector_object_history(
        &self,
    ) -> Option<&super::ViewportMotionVectorObjectHistory> {
        self.previous_motion_vector_object_history.as_ref()
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

    pub(crate) fn ambient_lights(
        &self,
    ) -> &[crate::core::framework::render::RenderAmbientLightSnapshot] {
        &self.extract.lighting.ambient_lights
    }

    pub(crate) fn overlays(&self) -> &crate::core::framework::render::RenderOverlayExtract {
        &self.extract.debug.overlays
    }

    pub(crate) fn preview(&self) -> &crate::core::framework::render::PreviewEnvironmentExtract {
        &self.extract.post_process.preview
    }
}
