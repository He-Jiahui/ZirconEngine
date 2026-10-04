use super::ParticleGpuFeedback;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// 粒子渲染结果的可选反馈；没有 GPU 回读时仍保留明确的空反馈供提交记录阶段处理。
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
