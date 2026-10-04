#[test]
fn production_writeback_is_cpu_only_after_the_backend_completion_poll() {
    let source = include_str!("../ibl_bake_runtime_writeback.rs");
    let production = source
        .split_once("#[cfg(test)]\npub(in crate::graphics::scene::scene_renderer) fn write_ibl")
        .map(|(production, _)| production)
        .expect("runtime writeback must retain a test-only synchronous helper boundary");

    assert!(production.contains("readback.poll_ready()"));
    assert!(!production.contains("wgpu::Buffer"));
    assert!(!production.contains("map_async("));
    assert!(!production.contains("device.poll("));
    assert!(!production.contains("queue.submit("));
    assert!(!production.contains("take_command_buffer("));
}

#[test]
fn capture_writeback_reuses_bounded_poll_owner_without_graph_resource_access() {
    let source = include_str!("../ibl_bake_runtime_writeback.rs");
    let capture = source
        .split_once("prepare_from_capture_target(")
        .and_then(|(_, tail)| {
            tail.split_once("pub(in crate::graphics::scene::scene_renderer) fn commit_submitted")
        })
        .map(|(capture, _)| capture)
        .expect("capture writeback preparation must remain an explicit owner");

    assert!(capture.contains("prepare_ibl_bake_artifact_wgpu_readback_from_capture_target"));
    assert!(capture.contains("self.pending.len() >= MAX_PENDING_IBL_BAKE_RUNTIME_WRITEBACKS"));
    assert!(capture.contains("allow_readback_failure: true"));
    assert!(!capture.contains("RenderGraphExecutionResources"));
    assert!(!capture.contains("owned_texture("));

    let completion = source
        .split_once("pub(in crate::graphics::scene::scene_renderer) fn poll_completed")
        .map(|(_, tail)| tail)
        .expect("runtime writeback must retain one bounded completion owner");
    assert!(completion.contains("Err(_) if pending.allow_readback_failure => continue"));
    assert!(completion.contains("if pending.allow_readback_failure"));
}
