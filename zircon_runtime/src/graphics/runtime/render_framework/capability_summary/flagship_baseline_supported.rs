use crate::rhi::RenderQueueClass;

pub(super) fn flagship_baseline_supported(caps: &crate::rhi::RenderBackendCaps) -> bool {
    caps.supports_offscreen && caps.supports_queue(RenderQueueClass::Graphics)
}
