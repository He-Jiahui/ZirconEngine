use super::ParticleGpuFeedback;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParticleRuntimeFeedback {
    gpu_feedback: Option<ParticleGpuFeedback>,
}

impl ParticleRuntimeFeedback {
    pub fn new(gpu_feedback: Option<ParticleGpuFeedback>) -> Self {
        Self { gpu_feedback }
    }

    pub fn gpu_feedback(&self) -> Option<&ParticleGpuFeedback> {
        self.gpu_feedback.as_ref()
    }

    pub fn into_gpu_feedback(self) -> Option<ParticleGpuFeedback> {
        self.gpu_feedback
    }
}
