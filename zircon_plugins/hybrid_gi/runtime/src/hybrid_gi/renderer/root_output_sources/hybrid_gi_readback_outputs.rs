use crate::hybrid_gi::renderer::{HybridGiGpuReadback, HybridGiGpuReadbackCompletionParts};

#[derive(Default)]
pub(super) struct HybridGiReadbackOutputs {
    gpu_readback: Option<HybridGiGpuReadback>,
}

impl HybridGiReadbackOutputs {
    pub(in crate::hybrid_gi::renderer) fn store_gpu_readback(
        &mut self,
        readback: Option<HybridGiGpuReadback>,
    ) {
        self.gpu_readback = readback;
    }

    pub(in crate::hybrid_gi::renderer) fn take_gpu_completion_parts(
        &mut self,
    ) -> Option<HybridGiGpuReadbackCompletionParts> {
        self.gpu_readback
            .take()
            .map(HybridGiGpuReadback::into_completion_parts)
    }

    #[cfg(test)]
    pub(crate) fn take_gpu_readback(&mut self) -> Option<HybridGiGpuReadback> {
        self.gpu_readback.take()
    }
}
