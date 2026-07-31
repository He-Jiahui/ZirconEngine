use crate::core::framework::render::{CameraRenderDescriptor, RenderFrameExtract};

use super::super::super::viewport_record::ViewportCameraHistoryKey;

pub(super) fn camera_history_key_for_extract(
    extract: &RenderFrameExtract,
) -> ViewportCameraHistoryKey {
    if let Some(descriptor) = extract.view.selected_camera_descriptor() {
        return ViewportCameraHistoryKey::from_camera(descriptor);
    }
    let descriptor = CameraRenderDescriptor::from_camera_payload(
        extract.view.scene_camera_entity,
        extract.view.camera.clone(),
    );
    ViewportCameraHistoryKey::from_camera(&descriptor)
}

#[cfg(test)]
mod tests {
    #[test]
    fn selected_camera_history_key_borrows_the_existing_descriptor() {
        let source = include_str!("camera_history_key.rs");

        assert!(source.contains("if let Some(descriptor)"));
        assert!(!source.contains(concat!("selected_camera_descriptor()", ".", "cloned()")));
    }
}
