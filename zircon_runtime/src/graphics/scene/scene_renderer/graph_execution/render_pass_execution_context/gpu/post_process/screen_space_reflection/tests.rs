use super::ssr_parent_pyramid_mip_passes;
use crate::render_graph::RenderGraphAttachmentOps;

#[test]
fn ssr_parent_pyramid_mip_passes_are_empty_for_single_mip_parent() {
    assert_eq!(
        ssr_parent_pyramid_mip_passes(1, RenderGraphAttachmentOps::load_store())
            .collect::<Vec<_>>(),
        Vec::new()
    );
}

#[test]
fn ssr_parent_pyramid_mip_passes_preserve_graph_alias_ops_for_mip_one() {
    let passes = ssr_parent_pyramid_mip_passes(2, RenderGraphAttachmentOps::load_store())
        .collect::<Vec<_>>();

    assert_eq!(passes.len(), 1);
    assert_eq!(passes[0].source_mip_level, 0);
    assert_eq!(passes[0].target_mip_level, 1);
    assert_eq!(
        passes[0].attachment_ops,
        RenderGraphAttachmentOps::load_store()
    );
}

#[test]
fn ssr_parent_pyramid_mip_passes_clear_later_mips_after_graph_alias_mip() {
    let passes = ssr_parent_pyramid_mip_passes(5, RenderGraphAttachmentOps::load_store())
        .collect::<Vec<_>>();

    assert_eq!(
        passes
            .iter()
            .map(|pass| (pass.source_mip_level, pass.target_mip_level))
            .collect::<Vec<_>>(),
        vec![(0, 1), (1, 2), (2, 3), (3, 4)]
    );
    assert_eq!(
        passes[0].attachment_ops,
        RenderGraphAttachmentOps::load_store()
    );
    assert!(passes[1..]
        .iter()
        .all(|pass| pass.attachment_ops == RenderGraphAttachmentOps::clear_store()));
}
