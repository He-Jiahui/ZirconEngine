// BUG: [CR-GRAPHICS-SCENECORE-0002] 源码守卫测试断言单行调用文本，但生产代码将 acquire_frame_target 与 present_frame_target 分行；该测试在当前源码上必然失败。证据：本文件 22-24、37-38、61-65 行。
#[test]
fn direct_surface_errors_retain_the_scene_submission_receipt() {
    let source = include_str!("../scene_renderer_viewport_surface.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("direct surface test boundary");

    assert!(source.contains("surface.acquire_frame_target()"));
    assert!(source.contains("let poll_receipt = self.poll_frame_submission_completions()?"));
    assert!(source.contains("self.render_frame_to_offscreen_target_after_poll("));
    assert!(source.contains("surface.present_frame_target("));
    assert!(source.contains("surface.discard_frame_target(surface_target, source)"));
    assert!(source.contains("finalize_surface_presentation("));
    assert!(!source.contains("surface.present_texture("));
}
