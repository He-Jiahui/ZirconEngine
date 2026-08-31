use crate::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderOverlayExtract,
    RenderSceneGeometryExtract, RenderSceneSnapshot, ViewportCameraSnapshot,
};
use crate::core::math::UVec2;
use std::sync::Arc;

use super::{
    ViewportCameraStackAttachmentPolicy, ViewportCameraStackOutputPolicy, ViewportRenderRegion,
    viewport_render_frame::ViewportRenderFrame,
};

impl ViewportRenderFrame {
    pub fn from_extract(mut extract: RenderFrameExtract, viewport_size: impl Into<UVec2>) -> Self {
        extract.view.sync_selected_descriptor_camera_payload();
        Self::from_shared_extract(Arc::new(extract), viewport_size)
    }

    pub(crate) fn from_shared_extract(
        extract: Arc<RenderFrameExtract>,
        viewport_size: impl Into<UVec2>,
    ) -> Self {
        let viewport_size = viewport_size.into();
        let camera_stack_attachment_policy = extract
            .view
            .selected_camera_descriptor()
            .map(ViewportCameraStackAttachmentPolicy::from_camera)
            .unwrap_or_default();
        let render_region = ViewportRenderRegion::from_camera(
            extract.view.selected_camera_descriptor(),
            viewport_size,
        );
        let scene = RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            environment: crate::core::framework::render::EnvironmentExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: crate::core::math::Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        };
        Self {
            scene,
            extract,
            viewport_size: UVec2::new(viewport_size.x.max(1), viewport_size.y.max(1)),
            shader_quality: Default::default(),
            texture_mip_bias: 0,
            texture_max_anisotropy: 16,
            ui: None,
            output_target: Default::default(),
            previous_motion_vector_camera: None,
            frame_visibility: None,
            virtual_geometry_debug_snapshot: None,
            runtime_overlay_override: None,
            post_process_override: None,
            environment_source_cubemap_override: None,
            particle_previous_sprites_override: None,
            prepared_runtime_sidebands: Default::default(),
            camera_stack_attachment_policy,
            camera_stack_output_policy: ViewportCameraStackOutputPolicy::default(),
            render_region,
        }
    }
}
