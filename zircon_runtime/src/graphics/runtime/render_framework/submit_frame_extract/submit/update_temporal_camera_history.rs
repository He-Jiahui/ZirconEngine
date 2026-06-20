use crate::graphics::ViewportRenderFrame;

use super::super::super::viewport_record::{ViewportCameraHistoryKey, ViewportRecord};

pub(super) fn update_temporal_camera_history_after_success(
    record: &mut ViewportRecord,
    frame: &ViewportRenderFrame,
    camera_history_key: &ViewportCameraHistoryKey,
    advance_temporal_frame_index: bool,
) {
    let mut camera = frame.effective_camera();
    camera.temporal_jitter = Default::default();
    record.replace_motion_vector_camera(camera_history_key.clone(), camera);
    if advance_temporal_frame_index {
        record.advance_temporal_frame_index();
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        RenderFrameExtract, RenderViewportDescriptor, RenderWorldSnapshotHandle,
        TemporalJitterSample,
    };
    use crate::core::math::{Transform, UVec2, Vec2, Vec3};
    use crate::graphics::runtime::render_framework::viewport_record::ViewportCameraHistoryKey;
    use crate::graphics::runtime::render_framework::viewport_record::ViewportRecord;
    use crate::graphics::ViewportRenderFrame;
    use crate::scene::world::World;

    use super::update_temporal_camera_history_after_success;

    #[test]
    fn successful_submit_records_camera_history_for_next_frame() {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(7),
            World::new().to_render_snapshot(),
        );
        extract.view.camera.transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        extract.view.camera.temporal_jitter = TemporalJitterSample {
            offset_pixels: Vec2::new(0.25, -0.5),
            sequence_index: 4,
        };
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));
        let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));

        let key = ViewportCameraHistoryKey::from_camera(frame.camera());

        update_temporal_camera_history_after_success(&mut record, &frame, &key, true);

        assert_eq!(record.temporal_frame_index(), 1);
        assert_eq!(
            record
                .motion_vector_camera(&key)
                .map(|camera| camera.transform),
            Some(frame.effective_camera().transform)
        );
        assert_eq!(
            record
                .motion_vector_camera(&key)
                .map(|camera| camera.temporal_jitter),
            Some(TemporalJitterSample::default())
        );
    }

    #[test]
    fn successful_non_terminal_submit_records_camera_without_advancing_viewport_index() {
        let extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(8),
            World::new().to_render_snapshot(),
        );
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));
        let key = ViewportCameraHistoryKey::from_camera(frame.camera());
        let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));

        update_temporal_camera_history_after_success(&mut record, &frame, &key, false);

        assert_eq!(record.temporal_frame_index(), 0);
        assert!(record.motion_vector_camera(&key).is_some());
    }
}
