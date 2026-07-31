use crate::core::framework::render::ViewportCameraSnapshot;

use super::{ViewportCameraHistoryKey, viewport_record::ViewportRecord};

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn motion_vector_camera(
        &self,
        key: &ViewportCameraHistoryKey,
    ) -> Option<&ViewportCameraSnapshot> {
        self.motion_vector_cameras.get(key)
    }

    pub(in crate::graphics::runtime::render_framework) fn replace_motion_vector_camera(
        &mut self,
        key: ViewportCameraHistoryKey,
        camera: ViewportCameraSnapshot,
    ) {
        self.motion_vector_cameras.insert(key, camera);
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        CameraRenderDescriptor, RenderCameraTarget, RenderViewportDescriptor, RenderViewportRect,
        ViewportCameraSnapshot,
    };
    use crate::core::math::{Transform, UVec2, Vec3};

    use super::super::camera_history_key::ViewportCameraHistoryKey;
    use super::ViewportRecord;

    #[test]
    fn viewport_record_keeps_motion_vector_camera_per_camera_key() {
        let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));
        let left_key = camera_key(1, UVec2::ZERO);
        let right_key = camera_key(1, UVec2::new(32, 0));
        let mut left_camera = ViewportCameraSnapshot::default();
        left_camera.transform = Transform::from_translation(Vec3::new(-1.0, 0.0, 4.0));
        let mut right_camera = ViewportCameraSnapshot::default();
        right_camera.transform = Transform::from_translation(Vec3::new(1.0, 0.0, 4.0));
        let left_transform = left_camera.transform;
        let right_transform = right_camera.transform;

        record.replace_motion_vector_camera(left_key.clone(), left_camera);
        record.replace_motion_vector_camera(right_key.clone(), right_camera);

        assert_eq!(
            record
                .motion_vector_camera(&left_key)
                .map(|camera| camera.transform),
            Some(left_transform)
        );
        assert_eq!(
            record
                .motion_vector_camera(&right_key)
                .map(|camera| camera.transform),
            Some(right_transform)
        );
    }

    fn camera_key(entity: u64, position: UVec2) -> ViewportCameraHistoryKey {
        let mut descriptor = CameraRenderDescriptor::from_camera_payload(
            Some(entity),
            ViewportCameraSnapshot::default(),
        );
        descriptor.target = RenderCameraTarget::PrimarySurface;
        descriptor.viewport_rect = Some(RenderViewportRect::new(position, UVec2::new(32, 64)));
        ViewportCameraHistoryKey::from_camera(&descriptor)
    }
}
