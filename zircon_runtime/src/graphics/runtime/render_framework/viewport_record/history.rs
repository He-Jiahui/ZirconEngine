use crate::graphics::runtime::ViewportFrameHistory;

use super::{viewport_record::ViewportRecord, ViewportCameraHistoryKey};

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn history(
        &self,
        key: &ViewportCameraHistoryKey,
    ) -> Option<&ViewportFrameHistory> {
        self.camera_histories.get(key)
    }

    pub(in crate::graphics::runtime::render_framework) fn history_mut(
        &mut self,
        key: &ViewportCameraHistoryKey,
    ) -> Option<&mut ViewportFrameHistory> {
        self.camera_histories.get_mut(key)
    }

    pub(in crate::graphics::runtime::render_framework) fn replace_history(
        &mut self,
        key: ViewportCameraHistoryKey,
        history: ViewportFrameHistory,
    ) {
        self.camera_histories.insert(key, history);
    }

    pub(in crate::graphics::runtime::render_framework) fn into_histories(
        self,
    ) -> impl Iterator<Item = ViewportFrameHistory> {
        self.camera_histories.into_values()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        CameraRenderDescriptor, FrameHistoryHandle, RenderCameraTarget, RenderPipelineHandle,
        RenderViewportDescriptor, RenderViewportRect, ViewportCameraSnapshot,
    };
    use crate::core::math::UVec2;
    use crate::graphics::runtime::{FrameHistoryValidationKey, ViewportFrameHistory};
    use crate::graphics::visibility::VisibilityStaticIndex;
    use crate::graphics::VisibilityHistorySnapshot;

    use super::super::camera_history_key::ViewportCameraHistoryKey;
    use super::ViewportRecord;

    #[test]
    fn viewport_record_keeps_histories_per_camera_key() {
        let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));
        let left_key = camera_key(10, UVec2::ZERO);
        let right_key = camera_key(10, UVec2::new(32, 0));
        let left_static_index = VisibilityStaticIndex::new(8.0);
        let right_static_index = VisibilityStaticIndex::new(32.0);

        record.replace_history(left_key.clone(), history(1, left_static_index.clone()));
        record.replace_history(right_key.clone(), history(2, right_static_index.clone()));

        let left_history = record
            .history(&left_key)
            .expect("left camera history should be retained");
        let right_history = record
            .history(&right_key)
            .expect("right camera history should be retained");

        assert_eq!(left_history.handle(), FrameHistoryHandle::new(1));
        assert_eq!(right_history.handle(), FrameHistoryHandle::new(2));
        assert_eq!(left_history.static_index(), &left_static_index);
        assert_eq!(right_history.static_index(), &right_static_index);
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

    fn history(handle: u64, static_index: VisibilityStaticIndex) -> ViewportFrameHistory {
        ViewportFrameHistory::new(
            FrameHistoryHandle::new(handle),
            UVec2::new(64, 64),
            UVec2::new(64, 64),
            RenderPipelineHandle::new(1),
            handle,
            Vec::new(),
            VisibilityHistorySnapshot::default(),
            static_index,
            VisibilityStaticIndex::default(),
            std::sync::Arc::new(FrameHistoryValidationKey::default()),
        )
    }
}
