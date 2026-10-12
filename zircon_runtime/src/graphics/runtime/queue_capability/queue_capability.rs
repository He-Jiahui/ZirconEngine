use crate::core::framework::render::RenderQueueCapability;
use crate::rhi::RenderQueueClass;

pub(in crate::graphics::runtime::render_framework) fn queue_capability(
    queue: RenderQueueClass,
) -> RenderQueueCapability {
    match queue {
        RenderQueueClass::Graphics => RenderQueueCapability::Graphics,
        RenderQueueClass::Compute => RenderQueueCapability::Compute,
        RenderQueueClass::Copy => RenderQueueCapability::Copy,
    }
}
