use crate::graphics::ViewportMotionVectorObjectHistory;

use super::viewport_record::ViewportRecord;

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn motion_vector_object_history(
        &self,
    ) -> Option<&ViewportMotionVectorObjectHistory> {
        self.motion_vector_object_history.as_ref()
    }

    pub(in crate::graphics::runtime::render_framework) fn replace_motion_vector_object_history(
        &mut self,
        history: ViewportMotionVectorObjectHistory,
    ) {
        self.motion_vector_object_history = Some(history);
    }
}
