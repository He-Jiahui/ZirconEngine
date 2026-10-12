use crate::render_graph::{
    RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps, RenderGraphAttachmentStoreOp,
};

use super::sprite_subpass_attachment_ops;

#[test]
fn sprite_subpasses_apply_graph_attachment_ops_only_to_outer_draws() {
    let graph_ops = RenderGraphAttachmentOps::clear_discard();

    assert_eq!(
        sprite_subpass_attachment_ops(graph_ops, 0, 3),
        RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Clear,
            store: RenderGraphAttachmentStoreOp::Store,
        }
    );
    assert_eq!(
        sprite_subpass_attachment_ops(graph_ops, 1, 3),
        RenderGraphAttachmentOps::load_store()
    );
    assert_eq!(
        sprite_subpass_attachment_ops(graph_ops, 2, 3),
        RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Discard,
        }
    );
}
