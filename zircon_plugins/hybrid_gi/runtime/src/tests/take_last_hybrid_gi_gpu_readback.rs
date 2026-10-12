use crate::hybrid_gi::renderer::HybridGiGpuReadback;

#[cfg(test)]
pub(crate) fn take_hybrid_gi_gpu_readback(
    readback: &mut Option<HybridGiGpuReadback>,
) -> Option<HybridGiGpuReadback> {
    readback.take()
}
