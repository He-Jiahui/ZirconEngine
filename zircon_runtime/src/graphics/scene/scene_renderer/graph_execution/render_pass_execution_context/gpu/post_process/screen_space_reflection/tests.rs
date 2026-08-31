use super::ssr_parent_pyramid_mip_passes;
use crate::render_graph::RenderGraphAttachmentOps;

#[test]
fn ssr_resolve_reads_previous_history_through_the_graph_resolver() {
    let source = include_str!("../screen_space_reflection.rs");
    let start = source
        .find("fn record_screen_space_reflection_resolve_to_resource")
        .expect("SSR resolve function");
    let end = source[start..]
        .find("fn record_screen_space_reflection_reflection_pyramid_coarse_to_resource")
        .map(|offset| start + offset)
        .expect("next SSR function");
    let resolve = &source[start..end];

    assert!(
        resolve.contains("PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION")
    );
    assert!(resolve.contains("Self::optional_texture_view_by_name("));
    assert!(!resolve.contains("history.screen_space_reflection_view"));
}

#[test]
fn post_process_consumers_do_not_read_history_owner_views_directly() {
    let ssr = include_str!("../screen_space_reflection.rs");
    assert!(!ssr.contains("history.map(|history| &history.global_illumination_view)"));
    assert!(!ssr.contains("history.map(|history| &history.screen_space_reflection_view)"));

    let post_process = include_str!("../../post_process.rs");
    let start = post_process
        .find("fn record_post_process_stack")
        .expect("uber post-process function");
    let end = post_process[start..]
        .find("fn record_color_lut_bake_to_resource")
        .map(|offset| start + offset)
        .expect("next post-process function");
    let uber = &post_process[start..end];
    assert!(uber.contains("HISTORY_PREVIOUS_HYBRID_GI"));
    assert!(uber.contains("Self::optional_texture_view_by_name("));
    assert!(!uber.contains("history.global_illumination_view"));
    assert!(!uber.contains("history.screen_space_reflection_view"));
}

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
    assert!(
        passes[1..]
            .iter()
            .all(|pass| pass.attachment_ops == RenderGraphAttachmentOps::clear_store())
    );
}
