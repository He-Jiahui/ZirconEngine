use zircon_framework::render::RenderQueueCapability;
use zircon_rhi::RenderQueueClass;

pub(in crate::runtime::render_framework) fn queue_capability(
    queue: RenderQueueClass,
) -> RenderQueueCapability {
    match queue {
        RenderQueueClass::Graphics => RenderQueueCapability::Graphics,
        RenderQueueClass::Compute => RenderQueueCapability::Compute,
        RenderQueueClass::Copy => RenderQueueCapability::Copy,
    }
}
