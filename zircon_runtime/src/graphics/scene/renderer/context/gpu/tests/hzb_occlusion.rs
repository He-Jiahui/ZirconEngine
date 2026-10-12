#[test]
fn hzb_occlusion_dispatch_record_reports_compaction_output_writes() {
    let source = include_str!("../hzb_occlusion.rs");

    assert!(source.contains("HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE.to_string()"));
    assert!(source.contains("HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE.to_string()"));
    assert!(source.contains("HZB_OCCLUSION_DRAW_COUNT_RESOURCE.to_string()"));
    assert!(source.contains("HZB_OCCLUSION_STATS_RESOURCE.to_string()"));
}

#[test]
fn hzb_occlusion_returns_uploads_and_commit_tokens_to_the_graph_owner() {
    let source = include_str!("../hzb_occlusion.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("HZB graph context source");

    assert!(!production.contains("self.queue"));
    assert!(production.contains("self.append_pre_submit_buffer_uploads("));
    assert!(production.contains("self.hzb_occlusion_params_commits.extend("));
}
