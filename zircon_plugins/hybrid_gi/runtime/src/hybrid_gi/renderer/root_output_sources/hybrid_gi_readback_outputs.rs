use crate::hybrid_gi::renderer::{HybridGiGpuReadback, HybridGiGpuReadbackCompletionParts};
use zircon_runtime::core::framework::render::RenderHybridGiReadbackOutputs;

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

    // 读取会消费唯一的 GPU 回读；重复读取只返回空默认值。
    pub(in crate::hybrid_gi::renderer) fn take_neutral_readback_outputs(
        &mut self,
    ) -> RenderHybridGiReadbackOutputs {
        self.gpu_readback
            .take()
            .map(RenderHybridGiReadbackOutputs::from)
            .unwrap_or_default()
    }

    #[cfg(test)]
    pub(crate) fn take_gpu_readback(&mut self) -> Option<HybridGiGpuReadback> {
        self.gpu_readback.take()
    }
}

#[cfg(test)]
#[path = "tests/hybrid_gi_readback_outputs.rs"]
mod tests;
