#[test]
fn terminal_poll_releases_the_active_lock_before_join_and_cleanup() {
    let source = include_str!("../mod.rs");
    let body = source
        .split("fn poll(&self)")
        .nth(1)
        .and_then(|body| body.split("impl Drop").next())
        .expect("process poll body should remain available");
    let release = body
        .find("drop(active)")
        .expect("poll should release active lock");
    let finish = body
        .find("child.finish(status)")
        .expect("poll should finish child");

    assert!(release < finish);
}

#[test]
fn production_poll_path_rejects_play_without_an_owned_child() {
    let source = include_str!("../mod.rs");
    let start = source
        .find("impl PlayBackend for ProcessPlayBackend")
        .expect("process backend implementation should remain available");
    let end = source
        .find("impl Drop for ProcessPlayBackend")
        .expect("process backend drop implementation should remain available");
    let production = &source[start..end];

    assert!(production.contains("runtime preview process is not active while Play is running"));
    assert!(production.contains("ActivePlayProcess::Idle"));
    assert!(!production.contains("diagnostics: Vec::new()"));
}
