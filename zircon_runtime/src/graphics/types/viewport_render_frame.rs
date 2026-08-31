use crate::core::framework::render::{
    CameraRenderDescriptor, PostProcessExtract, RenderFrameExtract, RenderOverlayExtract,
    RenderParticlePreviousSpriteSnapshot, RenderPipelinePhase, RenderPreparedRuntimeSidebands,
    RenderSceneSnapshot, RenderViewFamilyPipeline, RenderVirtualGeometryDebugSnapshot,
    ShaderQualityTier, SourceCubemapEnvironment, UiRenderSubmission, ViewportCameraSnapshot,
    VolumetricFogSettings,
};
use crate::core::math::UVec2;
use crate::graphics::visibility::FrameVisibility;
use std::sync::Arc;

use super::{
    ViewportCameraStackAttachmentPolicy, ViewportCameraStackOutputPolicy, ViewportRenderRegion,
};

#[derive(Clone, Debug, Default)]
pub(crate) struct RendererPostProcessSnapshot {
    post_process: PostProcessExtract,
    volumetric_fog: VolumetricFogSettings,
}

impl RendererPostProcessSnapshot {
    pub(crate) fn new(
        post_process: PostProcessExtract,
        volumetric_fog: VolumetricFogSettings,
    ) -> Self {
        Self {
            post_process,
            volumetric_fog,
        }
    }

    pub(crate) fn post_process(&self) -> &PostProcessExtract {
        &self.post_process
    }

    pub(crate) fn volumetric_fog(&self) -> VolumetricFogSettings {
        self.volumetric_fog
    }
}

#[derive(Clone, Debug)]
pub struct ViewportRenderFrame {
    pub scene: RenderSceneSnapshot,
    pub extract: Arc<RenderFrameExtract>,
    pub viewport_size: UVec2,
    pub(crate) shader_quality: ShaderQualityTier,
    pub(crate) texture_mip_bias: u8,
    pub(crate) texture_max_anisotropy: u8,
    /// Screen-space runtime UI payload selected for this viewport target.
    pub ui: Option<Arc<UiRenderSubmission>>,
    pub(crate) output_target: super::ViewportRenderOutputTarget,
    pub(crate) previous_motion_vector_camera: Option<ViewportCameraSnapshot>,
    pub(crate) frame_visibility: Option<FrameVisibility>,
    pub(crate) virtual_geometry_debug_snapshot: Option<Arc<RenderVirtualGeometryDebugSnapshot>>,
    pub(crate) runtime_overlay_override: Option<RenderOverlayExtract>,
    pub(crate) post_process_override: Option<Arc<RendererPostProcessSnapshot>>,
    pub(crate) environment_source_cubemap_override: Option<SourceCubemapEnvironment>,
    pub(crate) particle_previous_sprites_override:
        Option<Vec<RenderParticlePreviousSpriteSnapshot>>,
    pub(crate) prepared_runtime_sidebands: RenderPreparedRuntimeSidebands,
    pub(crate) camera_stack_attachment_policy: ViewportCameraStackAttachmentPolicy,
    pub(crate) camera_stack_output_policy: ViewportCameraStackOutputPolicy,
    pub(crate) render_region: ViewportRenderRegion,
}

impl ViewportRenderFrame {
    pub(in crate::graphics) fn select_camera_descriptor(&mut self, camera: CameraRenderDescriptor) {
        self.extract = Arc::new(self.extract.for_camera_submission(camera));
    }

    pub(crate) fn with_post_process_override(
        mut self,
        post_process: Arc<RendererPostProcessSnapshot>,
    ) -> Self {
        self.post_process_override = Some(post_process);
        self
    }

    pub(crate) fn post_process(&self) -> &PostProcessExtract {
        self.post_process_override
            .as_deref()
            .map(RendererPostProcessSnapshot::post_process)
            .unwrap_or(&self.extract.post_process)
    }

    pub(crate) fn volumetric_fog(&self) -> VolumetricFogSettings {
        if let Some(snapshot) = self.post_process_override.as_deref() {
            return snapshot.volumetric_fog();
        }
        if let Some(settings) = self.extract.lighting.advanced_lighting.volumetric {
            return settings;
        }
        let camera = self.extract.view.selected_effective_camera();
        self.extract
            .post_process
            .resolved_settings_for_camera(
                camera.transform.translation,
                self.extract.view.selected_camera_volume_layers(),
            )
            .map(|settings| settings.volumetric_fog)
            .unwrap_or_default()
    }

    pub(crate) fn with_environment_source_cubemap_override(
        mut self,
        source_cubemap: Option<SourceCubemapEnvironment>,
    ) -> Self {
        self.environment_source_cubemap_override = source_cubemap;
        self
    }

    pub(crate) fn source_cubemap_environment(&self) -> Option<&SourceCubemapEnvironment> {
        self.environment_source_cubemap_override
            .as_deref()
            .or_else(|| self.extract.environment.skybox.source_cubemap_environment())
    }

    pub(crate) fn previous_particle_sprites(&self) -> &[RenderParticlePreviousSpriteSnapshot] {
        self.particle_previous_sprites_override
            .as_deref()
            .unwrap_or(&self.extract.particles.previous_sprites)
    }

    pub(crate) fn with_particle_previous_sprites_override(
        mut self,
        previous_sprites: Option<Vec<RenderParticlePreviousSpriteSnapshot>>,
    ) -> Self {
        self.particle_previous_sprites_override = previous_sprites;
        self
    }

    pub(crate) fn take_particle_previous_sprites_override(
        &mut self,
    ) -> Option<Vec<RenderParticlePreviousSpriteSnapshot>> {
        self.particle_previous_sprites_override.take()
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

    pub(crate) fn texture_mip_bias(&self) -> u8 {
        self.texture_mip_bias
    }

    pub(crate) fn texture_max_anisotropy(&self) -> u8 {
        self.texture_max_anisotropy
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

    pub(crate) fn view_family_pipeline(&self) -> &RenderViewFamilyPipeline {
        self.extract.view.view_family_pipeline()
    }

    pub(crate) fn render_region_for_phase(
        &self,
        phase: RenderPipelinePhase,
    ) -> Option<ViewportRenderRegion> {
        self.view_family_pipeline()
            .output_target_for_phase(phase)
            .map(ViewportRenderRegion::from_view_family_target)
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
        &self.post_process().preview
    }

    pub(crate) fn environment(&self) -> &crate::core::framework::render::EnvironmentExtract {
        &self.extract.environment
    }
}
