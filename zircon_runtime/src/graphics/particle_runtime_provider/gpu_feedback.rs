use crate::core::framework::render::RenderParticleGpuReadbackOutputs;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// 单帧粒子 GPU 回读的有所有权载体；空结果由提交汇总层过滤，调用者消费后生成粒子统计。
pub struct ParticleGpuFeedback {
    readback_outputs: RenderParticleGpuReadbackOutputs,
}

impl ParticleGpuFeedback {
    pub fn new(readback_outputs: RenderParticleGpuReadbackOutputs) -> Self {
        Self { readback_outputs }
    }

    pub fn is_empty(&self) -> bool {
        self.readback_outputs.is_empty()
    }

    pub fn readback_outputs(&self) -> &RenderParticleGpuReadbackOutputs {
        &self.readback_outputs
    }

    pub fn into_readback_outputs(self) -> RenderParticleGpuReadbackOutputs {
        self.readback_outputs
    }
}
