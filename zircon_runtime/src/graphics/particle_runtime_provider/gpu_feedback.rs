use crate::core::framework::render::RenderParticleGpuReadbackOutputs;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
