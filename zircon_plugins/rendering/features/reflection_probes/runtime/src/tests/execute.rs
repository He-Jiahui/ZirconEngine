#[test]
fn capture_execution_is_a_nonblocking_framework_boundary() {
    let source = include_str!("../execute.rs");
    assert!(source.contains("request_environment_capture"));
    assert!(source.contains("poll_environment_capture"));
    assert!(source.contains("cancel_environment_capture"));
    assert!(source.contains("take_environment_capture_source_payload"));
    assert!(!source.contains(&["Scene", "Renderer"].concat()));
    assert!(!source.contains(&["render_scene", "_color_hdr"].concat()));
    assert!(!source.contains(&["IblSourceCubemap", "StagingStore"].concat()));
    assert!(!source.contains("Vec::with_capacity"));
}
