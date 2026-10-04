//! 粒子提交后的 GPU 回读边界；渲染器反馈在成功提交后转换为统计快照。
mod gpu_feedback;
mod runtime_feedback;

pub use gpu_feedback::ParticleGpuFeedback;
pub use runtime_feedback::ParticleRuntimeFeedback;
