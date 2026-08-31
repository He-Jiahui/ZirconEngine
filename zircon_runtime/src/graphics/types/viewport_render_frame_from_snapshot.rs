use crate::core::framework::render::{
    RenderFrameExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
};
use crate::core::math::UVec2;
use std::sync::Arc;

use super::{
    ViewportCameraStackAttachmentPolicy, ViewportCameraStackOutputPolicy, ViewportRenderRegion,
    viewport_render_frame::ViewportRenderFrame,
};

impl ViewportRenderFrame {
    pub fn from_snapshot(scene: RenderSceneSnapshot, viewport_size: impl Into<UVec2>) -> Self {
        let viewport_size = viewport_size.into();
        let extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(0), scene.clone());
        Self {
            scene,
            extract: Arc::new(extract),
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
            environment_source_cubemap_override: None,
            particle_previous_sprites_override: None,
            prepared_runtime_sidebands: Default::default(),
            camera_stack_attachment_policy: ViewportCameraStackAttachmentPolicy::default(),
            camera_stack_output_policy: ViewportCameraStackOutputPolicy::default(),
            render_region: ViewportRenderRegion::from_camera(None, viewport_size),
        }
    }
}
