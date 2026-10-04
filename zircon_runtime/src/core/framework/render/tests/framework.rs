#[test]
fn default_capture_poll_is_explicitly_nonblocking() {
    let source = include_str!("../framework.rs");
    let poll_source = source
        .split("fn poll_captured_frame_if_newer")
        .nth(1)
        .expect("render framework declares a capture polling contract");

    assert!(poll_source.contains("Ok(None)"));
    assert!(!poll_source.contains("capture_frame_if_newer"));
}
