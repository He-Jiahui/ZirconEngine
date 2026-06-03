use crate::render_graph::{
    RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps, RenderGraphAttachmentStoreOp,
};

pub(crate) fn color_attachment_operations(
    attachment_ops: RenderGraphAttachmentOps,
    clear_color: wgpu::Color,
) -> wgpu::Operations<wgpu::Color> {
    wgpu::Operations {
        load: match attachment_ops.load {
            RenderGraphAttachmentLoadOp::Load => wgpu::LoadOp::Load,
            RenderGraphAttachmentLoadOp::Clear => wgpu::LoadOp::Clear(clear_color),
        },
        store: store_op(attachment_ops.store),
    }
}

pub(crate) fn depth_attachment_operations(
    attachment_ops: RenderGraphAttachmentOps,
    clear_depth: f32,
) -> wgpu::Operations<f32> {
    wgpu::Operations {
        load: match attachment_ops.load {
            RenderGraphAttachmentLoadOp::Load => wgpu::LoadOp::Load,
            RenderGraphAttachmentLoadOp::Clear => wgpu::LoadOp::Clear(clear_depth),
        },
        store: store_op(attachment_ops.store),
    }
}

pub(crate) fn store_op(store: RenderGraphAttachmentStoreOp) -> wgpu::StoreOp {
    match store {
        RenderGraphAttachmentStoreOp::Store => wgpu::StoreOp::Store,
        RenderGraphAttachmentStoreOp::Discard => wgpu::StoreOp::Discard,
    }
}
