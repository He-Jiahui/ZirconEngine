use zircon_rhi::RenderQueueClass;

pub(super) fn flagship_baseline_supported(caps: &zircon_rhi::RenderBackendCaps) -> bool {
    caps.supports_offscreen && caps.supports_queue(RenderQueueClass::Graphics)
}
