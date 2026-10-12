use crate::hybrid_gi::renderer::{HybridGiGpuReadback, HybridGiGpuReadbackCompletionParts};

pub(crate) fn take_hybrid_gi_gpu_completion_parts(
    readback: &mut Option<HybridGiGpuReadback>,
) -> Option<HybridGiGpuReadbackCompletionParts> {
    readback
        .take()
        .map(HybridGiGpuReadback::into_completion_parts)
}
