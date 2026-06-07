use super::viewport_motion_vector_object_history::ViewportMotionVectorObjectHistory;
use super::viewport_render_frame::ViewportRenderFrame;

impl ViewportRenderFrame {
    pub(crate) fn with_previous_motion_vector_object_history(
        mut self,
        history: Option<ViewportMotionVectorObjectHistory>,
    ) -> Self {
        self.previous_motion_vector_object_history = history;
        self
    }
}
