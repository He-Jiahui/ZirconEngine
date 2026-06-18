use crate::ui::PublicRuntimeFrame;

use super::{
    viewport_render_frame::ViewportRenderFrame, ViewportCameraStackAttachmentPolicy,
    ViewportCameraStackOutputPolicy, ViewportRenderRegion,
};

impl From<PublicRuntimeFrame> for ViewportRenderFrame {
    fn from(frame: PublicRuntimeFrame) -> Self {
        let scene = frame.extract.to_scene_snapshot();
        let camera_stack_attachment_policy = frame
            .extract
            .view
            .selected_camera_descriptor()
            .map(ViewportCameraStackAttachmentPolicy::from_camera)
            .unwrap_or_default();
        let render_region = ViewportRenderRegion::from_camera(
            frame.extract.view.selected_camera_descriptor(),
            frame.viewport_size,
        );
        Self {
            scene,
            extract: frame.extract,
            viewport_size: frame.viewport_size,
            shader_quality: Default::default(),
            ui: frame.ui,
            output_target: Default::default(),
            previous_motion_vector_camera: None,
            frame_visibility: None,
            virtual_geometry_debug_snapshot: None,
            prepared_runtime_sidebands: Default::default(),
            camera_stack_attachment_policy,
            camera_stack_output_policy: ViewportCameraStackOutputPolicy::default(),
            render_region,
        }
    }
}
