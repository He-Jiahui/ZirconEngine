#[test]
fn half_resolution_transparency_context_uses_declared_graph_resources() {
    let source = include_str!("../half_res_transparency.rs");

    assert!(source.contains("HALF_RES_TRANSPARENCY_COLOR"));
    assert!(source.contains("HALF_RES_TRANSPARENCY_DEPTH"));
    assert!(source.contains("RenderGraphResourceAccessKind::Write"));
    assert!(source.contains("self.append_pre_submit_buffer_uploads("));
}
