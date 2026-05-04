use crate::{HybridGiRuntimeFeedback, ParticleRuntimeFeedback, VirtualGeometryRuntimeFeedback};

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) struct RuntimeFeedbackBatch
{
    hybrid_gi_feedback: HybridGiRuntimeFeedback,
    particle_feedback: ParticleRuntimeFeedback,
    virtual_geometry_feedback: VirtualGeometryRuntimeFeedback,
}

impl RuntimeFeedbackBatch {
    pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn new(
        hybrid_gi_feedback: HybridGiRuntimeFeedback,
        particle_feedback: ParticleRuntimeFeedback,
        virtual_geometry_feedback: VirtualGeometryRuntimeFeedback,
    ) -> Self {
        Self {
            hybrid_gi_feedback,
            particle_feedback,
            virtual_geometry_feedback,
        }
    }

    pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn into_parts(
        self,
    ) -> (
        HybridGiRuntimeFeedback,
        ParticleRuntimeFeedback,
        VirtualGeometryRuntimeFeedback,
    ) {
        (
            self.hybrid_gi_feedback,
            self.particle_feedback,
            self.virtual_geometry_feedback,
        )
    }
}
