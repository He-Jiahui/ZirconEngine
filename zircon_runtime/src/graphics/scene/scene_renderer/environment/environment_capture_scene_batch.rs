#[cfg(test)]
use std::sync::Arc;

use crate::core::framework::render::{
    cubemap_capture_camera, CameraRenderDescriptor, CubemapFace, RenderEnvironmentCaptureRequest,
    RenderFrameExtract, RenderOverlayExtract, RenderWorldSnapshotHandle, SceneViewportRenderPacket,
};
use crate::core::math::UVec2;
use crate::graphics::runtime::EnvironmentCaptureWorkItem;
use crate::graphics::types::{
    ViewportCameraStackAttachmentPolicy, ViewportRenderFrame, ViewportRenderRegion,
};

/// One moved scene extract reused by all six cubemap render passes.
///
/// The batch owns no GPU resources. It establishes the CPU-side capture
/// contract so the recorder can prepare scene resources and mesh draws once,
/// then change only the camera and per-face uniform binding between passes.
pub(in crate::graphics) struct EnvironmentCaptureSceneBatch {
    request: RenderEnvironmentCaptureRequest,
    frame: ViewportRenderFrame,
    selected_face: Option<CubemapFace>,
}

pub(in crate::graphics) struct EnvironmentCaptureSceneView<'a> {
    face: CubemapFace,
    frame: &'a ViewportRenderFrame,
    reverse_raster_winding: bool,
}

impl EnvironmentCaptureSceneBatch {
    pub(in crate::graphics) fn from_work_item(
        work_item: EnvironmentCaptureWorkItem,
    ) -> (
        crate::core::framework::render::RenderEnvironmentCaptureHandle,
        Self,
    ) {
        let (handle, scene, request) = work_item.into_parts();
        (handle, Self::new(scene, request))
    }

    pub(in crate::graphics) fn new(
        mut scene: SceneViewportRenderPacket,
        request: RenderEnvironmentCaptureRequest,
    ) -> Self {
        scene.overlays = RenderOverlayExtract::default();
        scene.virtual_geometry_debug = None;
        // A reflection capture is a lighting product, not a viewport preview mode. The
        // request currently has no emissive-only policy, so retain authored ambient and
        // direct lighting even when the source viewport is showing an unlit preview.
        scene.preview.lighting_enabled = true;

        let face_size = UVec2::splat(request.face_size());
        let mut extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(0), scene)
                .with_viewport_size(face_size);
        extract
            .view
            .selected_camera_descriptor_mut()
            .expect("environment capture extract must carry its source camera")
            .culling_mask = request.capture_layer_mask().clone();
        let frame = ViewportRenderFrame::from_extract(extract, face_size);

        Self {
            request,
            frame,
            selected_face: None,
        }
    }

    pub(in crate::graphics) fn request(&self) -> &RenderEnvironmentCaptureRequest {
        &self.request
    }

    pub(in crate::graphics) fn frame(&self) -> &ViewportRenderFrame {
        &self.frame
    }

    pub(in crate::graphics) fn selected_face(&self) -> Option<CubemapFace> {
        self.selected_face
    }

    pub(in crate::graphics) fn select_face(
        &mut self,
        face: CubemapFace,
    ) -> EnvironmentCaptureSceneView<'_> {
        let capture = cubemap_capture_camera(face, &self.request);
        let mut descriptor = CameraRenderDescriptor::from_camera_payload(None, capture.camera);
        descriptor.culling_mask = self.request.capture_layer_mask().clone();
        let target_size = UVec2::splat(self.request.face_size());

        self.frame.select_camera_descriptor(descriptor);
        let descriptor = self
            .frame
            .extract
            .view
            .selected_camera_descriptor()
            .expect("environment capture frame must retain its selected camera");
        self.frame.camera_stack_attachment_policy =
            ViewportCameraStackAttachmentPolicy::from_camera(descriptor);
        self.frame.render_region = ViewportRenderRegion::from_camera(Some(descriptor), target_size);
        self.frame.previous_motion_vector_camera = None;
        self.selected_face = Some(face);

        EnvironmentCaptureSceneView {
            face,
            frame: &self.frame,
            reverse_raster_winding: capture.reverses_winding,
        }
    }

    #[cfg(test)]
    fn extract_identity(&self) -> *const RenderFrameExtract {
        Arc::as_ptr(&self.frame.extract)
    }
}

impl EnvironmentCaptureSceneView<'_> {
    pub(in crate::graphics) fn face(&self) -> CubemapFace {
        self.face
    }

    pub(in crate::graphics) fn frame(&self) -> &ViewportRenderFrame {
        self.frame
    }

    pub(in crate::graphics) fn reverse_raster_winding(&self) -> bool {
        self.reverse_raster_winding
    }
}

#[cfg(test)]
#[path = "tests/environment_capture_scene_batch.rs"]
mod tests;
