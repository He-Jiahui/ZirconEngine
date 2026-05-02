use crate::VirtualGeometryRuntimeFeedback;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) struct RuntimeFeedbackBatch
{
    virtual_geometry_feedback: VirtualGeometryRuntimeFeedback,
}

impl RuntimeFeedbackBatch {
    pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn new(
        virtual_geometry_feedback: VirtualGeometryRuntimeFeedback,
    ) -> Self {
        Self {
            virtual_geometry_feedback,
        }
    }

    pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn into_parts(
        self,
    ) -> VirtualGeometryRuntimeFeedback {
        self.virtual_geometry_feedback
    }
}
