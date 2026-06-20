use crate::core::framework::render::{CameraRenderDescriptor, RenderFrameExtract};

use super::super::super::viewport_record::ViewportCameraHistoryKey;

pub(super) fn camera_history_key_for_extract(
    extract: &RenderFrameExtract,
) -> ViewportCameraHistoryKey {
    let descriptor = extract
        .view
        .selected_camera_descriptor()
        .cloned()
        .unwrap_or_else(|| {
            CameraRenderDescriptor::from_camera_payload(
                extract.view.scene_camera_entity,
                extract.view.camera.clone(),
            )
        });
    ViewportCameraHistoryKey::from_camera(&descriptor)
}
