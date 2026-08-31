#[cfg(test)]
use std::sync::Arc;

use crate::core::framework::render::{
    cubemap_capture_camera, CameraRenderDescriptor, CubemapFace, RenderEnvironmentCaptureRequest,
    RenderFrameExtract, RenderOverlayExtract, RenderWorldSnapshotHandle, SceneViewportRenderPacket,
};
use crate::core::math::UVec2;
use crate::graphics::runtime::render_framework::EnvironmentCaptureWorkItem;
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
mod tests {
    use super::*;
    use crate::core::framework::render::{
        EnvironmentExtract, PreviewEnvironmentExtract, RenderLayerSet, RenderSceneGeometryExtract,
        ViewProjectionMatrixPair, ViewportCameraSnapshot,
    };
    use crate::core::math::Vec3;

    #[test]
    fn six_face_selection_reuses_one_moved_scene_extract() {
        let request = RenderEnvironmentCaptureRequest::new("probe", [4.0, 5.0, 6.0], 7)
            .unwrap()
            .with_face_size(256)
            .unwrap();
        let mut batch = EnvironmentCaptureSceneBatch::new(test_scene(), request);
        let extract_identity = batch.extract_identity();

        for face in CubemapFace::ALL {
            let view = batch.select_face(face);

            assert_eq!(view.face(), face);
            assert!(view.reverse_raster_winding());
            assert_eq!(view.frame().viewport_size, UVec2::splat(256));
            assert_eq!(
                view.frame().render_region().physical_size(),
                UVec2::splat(256)
            );
            assert_eq!(view.frame().overlays(), &RenderOverlayExtract::default());
            assert_eq!(batch.extract_identity(), extract_identity);
        }
        assert_eq!(batch.selected_face(), Some(CubemapFace::NegativeZ));
    }

    #[test]
    fn capture_uses_authored_lighting_independent_of_viewport_preview_mode() {
        let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1).unwrap();
        let batch = EnvironmentCaptureSceneBatch::new(test_scene(), request);

        assert!(batch.frame().preview().lighting_enabled);
    }

    #[test]
    fn capture_mask_is_installed_before_draw_build_and_kept_for_every_face() {
        let capture_layers = RenderLayerSet::from_layers([3, 37]);
        let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1)
            .unwrap()
            .with_capture_layer_mask(capture_layers.clone());
        let mut batch = EnvironmentCaptureSceneBatch::new(test_scene(), request);

        let initial_descriptor = batch
            .frame()
            .extract
            .view
            .selected_camera_descriptor()
            .unwrap();
        assert_eq!(initial_descriptor.culling_mask, capture_layers);
        assert_eq!(initial_descriptor.volume_mask, RenderLayerSet::default());

        for face in CubemapFace::ALL {
            let view = batch.select_face(face);
            let descriptor = view
                .frame()
                .extract
                .view
                .selected_camera_descriptor()
                .unwrap();
            assert_eq!(descriptor.culling_mask, capture_layers);
            assert_eq!(descriptor.volume_mask, RenderLayerSet::default());
        }
    }

    #[test]
    fn selected_face_projects_its_canonical_center_without_temporal_state() {
        let request = RenderEnvironmentCaptureRequest::new("probe", [4.0, 5.0, 6.0], 7)
            .unwrap()
            .with_clip_planes(0.25, 500.0)
            .unwrap();
        let origin = Vec3::from_array(request.position());
        let mut batch = EnvironmentCaptureSceneBatch::new(test_scene(), request);

        for face in CubemapFace::ALL {
            let axes = face.projection_axes();
            let view = batch.select_face(face);
            let pair = ViewProjectionMatrixPair::from_camera(
                &view.frame().effective_camera(),
                view.frame().viewport_size,
            );
            let clip = pair
                .clip_from_world_unjittered
                .project_point3(origin + Vec3::from_array(axes.forward));

            assert!(clip.x.abs() <= 0.00001);
            assert!(clip.y.abs() <= 0.00001);
            assert!(view.frame().previous_motion_vector_camera().is_none());
            assert!(view.frame().ui.is_none());
        }
    }

    fn test_scene() -> SceneViewportRenderPacket {
        let environment = EnvironmentExtract::default();
        SceneViewportRenderPacket {
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
            preview: PreviewEnvironmentExtract::from_environment(
                &environment,
                false,
                crate::core::math::Vec4::ZERO,
            ),
            environment,
            virtual_geometry_debug: None,
        }
    }
}
