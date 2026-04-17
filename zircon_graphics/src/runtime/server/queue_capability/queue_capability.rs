use zircon_render_server::RenderQueueCapability;
use zircon_rhi::RenderQueueClass;

pub(in crate::runtime::server) fn queue_capability(
    queue: RenderQueueClass,
) -> RenderQueueCapability {
    match queue {
        RenderQueueClass::Graphics => RenderQueueCapability::Graphics,
        RenderQueueClass::Compute => RenderQueueCapability::Compute,
        RenderQueueClass::Copy => RenderQueueCapability::Copy,
    }
}
