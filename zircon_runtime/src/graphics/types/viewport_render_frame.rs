use crate::core::framework::render::{
    CameraRenderDescriptor, RenderFrameExtract, RenderOverlayExtract,
    RenderPreparedRuntimeSidebands, RenderSceneSnapshot, RenderVirtualGeometryDebugSnapshot,
    ShaderQualityTier, ViewportCameraSnapshot,
};
use crate::core::math::UVec2;
use crate::graphics::visibility::FrameVisibility;
use std::sync::Arc;
use zircon_runtime_interface::ui::surface::UiRenderExtract;

use super::{
    ViewportCameraStackAttachmentPolicy, ViewportCameraStackOutputPolicy, ViewportRenderRegion,
};

#[derive(Clone, Debug)]
pub struct ViewportRenderFrame {
    pub scene: RenderSceneSnapshot,
    pub extract: Arc<RenderFrameExtract>,
    pub viewport_size: UVec2,
    pub(crate) shader_quality: ShaderQualityTier,
    /// Screen-space runtime UI payload selected for this viewport target.
    pub ui: Option<UiRenderExtract>,
    pub(crate) output_target: super::ViewportRenderOutputTarget,
    pub(crate) previous_motion_vector_camera: Option<ViewportCameraSnapshot>,
    pub(crate) frame_visibility: Option<FrameVisibility>,
    pub(crate) virtual_geometry_debug_snapshot: Option<Arc<RenderVirtualGeometryDebugSnapshot>>,
    pub(crate) runtime_overlay_override: Option<RenderOverlayExtract>,
    pub(crate) prepared_runtime_sidebands: RenderPreparedRuntimeSidebands,
    pub(crate) camera_stack_attachment_policy: ViewportCameraStackAttachmentPolicy,
    pub(crate) camera_stack_output_policy: ViewportCameraStackOutputPolicy,
    pub(crate) render_region: ViewportRenderRegion,
}

impl ViewportRenderFrame {
    pub(crate) fn extract_mut(&mut self) -> &mut RenderFrameExtract {
        Arc::make_mut(&mut self.extract)
    }

    pub(crate) fn prepared_runtime_sidebands(&self) -> &RenderPreparedRuntimeSidebands {
        &self.prepared_runtime_sidebands
    }

    pub(crate) fn prepared_runtime_sidebands_mut(&mut self) -> &mut RenderPreparedRuntimeSidebands {
        &mut self.prepared_runtime_sidebands
    }

    pub(crate) fn shader_quality(&self) -> ShaderQualityTier {
        self.shader_quality
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

    pub(crate) fn camera(&self) -> &CameraRenderDescriptor {
        self.extract
            .view
            .selected_camera_descriptor()
            .expect("viewport render frame must carry a selected camera descriptor")
    }

    pub(crate) fn effective_camera(&self) -> ViewportCameraSnapshot {
        self.extract.view.selected_effective_camera()
    }

    pub(crate) fn previous_motion_vector_camera(&self) -> Option<&ViewportCameraSnapshot> {
        self.previous_motion_vector_camera.as_ref()
    }

    pub(crate) fn camera_stack_attachment_policy(&self) -> ViewportCameraStackAttachmentPolicy {
        self.camera_stack_attachment_policy
    }

    pub(crate) fn camera_stack_output_policy(&self) -> ViewportCameraStackOutputPolicy {
        self.camera_stack_output_policy
    }

    pub(crate) fn render_region(&self) -> ViewportRenderRegion {
        self.render_region
    }

    pub(crate) fn frame_visibility(&self) -> Option<&FrameVisibility> {
        self.frame_visibility.as_ref()
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

    pub(crate) fn point_lights(
        &self,
    ) -> &[crate::core::framework::render::RenderPointLightSnapshot] {
        &self.extract.lighting.point_lights
    }

    pub(crate) fn ambient_lights(
        &self,
    ) -> &[crate::core::framework::render::RenderAmbientLightSnapshot] {
        &self.extract.lighting.ambient_lights
    }

    pub(crate) fn overlays(&self) -> &crate::core::framework::render::RenderOverlayExtract {
        self.runtime_overlay_override
            .as_ref()
            .unwrap_or(&self.extract.debug.overlays)
    }

    pub(crate) fn preview(&self) -> &crate::core::framework::render::PreviewEnvironmentExtract {
        &self.extract.post_process.preview
    }

    pub(crate) fn environment(&self) -> &crate::core::framework::render::EnvironmentExtract {
        &self.extract.environment
    }
}
