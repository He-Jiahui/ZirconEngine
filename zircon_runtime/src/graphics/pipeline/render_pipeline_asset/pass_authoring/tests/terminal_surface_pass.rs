#[test]
fn terminal_outputs_are_non_cullable_graph_side_effects() {
    let source = include_str!("../terminal_surface_pass.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("surface-present test boundary");

    assert!(source.contains("RenderCameraTarget::PrimarySurface"));
    assert!(source.contains("RenderPassStage::Present"));
    assert!(source.contains("allow_culling: false"));
    assert!(source.contains("has_side_effects: true"));
    assert!(source.contains("graph.read_external("));
    assert!(source.contains("author_output_target_direct_import_pass("));
    assert!(source.contains("OUTPUT_TARGET_DIRECT_IMPORT_EXECUTOR_ID"));
    assert!(source.contains("import_present_external_texture_with_binding"));
    assert!(source.contains("RenderGraphResourceAccessIntent::sampled_texture"));
    assert!(source.contains("RenderGraphResourceAccessIntent::ColorAttachment"));
    assert!(!source.contains("RenderGraphResourceAccessIntent::CopySource"));
    assert!(!source.contains("RenderGraphResourceAccessIntent::CopyDestination"));
}
