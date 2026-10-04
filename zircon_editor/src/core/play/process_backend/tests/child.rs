#[test]
fn play_child_captures_and_terminates_the_persistent_tree_before_joining_output() {
    let source = include_str!("../child.rs");
    let attach = source
        .find("ProcessTreeLease::attach_and_start")
        .expect("spawn must attach the process tree");
    let output_capture = source
        .find("PlayOutputPump::capture")
        .expect("spawn must capture output");
    assert!(attach < output_capture);

    let finish = source
        .split("pub(super) fn finish")
        .nth(1)
        .and_then(|body| body.split("pub(super) fn stop").next())
        .expect("finish body should remain available");
    let terminate = finish
        .find("self.terminate_tree()")
        .expect("terminal finish must terminate the tree");
    let finalize = finish
        .find("self.finish_terminal()")
        .expect("terminal finish must finalize owned output and snapshot state");
    assert!(terminate < finalize);
}

#[test]
fn terminal_cleanup_retry_preserves_the_terminated_tree_phase() {
    let source = include_str!("../child.rs");
    let terminate = source
        .split("fn terminate_tree")
        .nth(1)
        .and_then(|body| body.split("fn finish_terminal").next())
        .expect("play child tree termination implementation");
    let finalize = source
        .split("fn finish_terminal")
        .nth(1)
        .and_then(|body| body.split("pub(super) fn cleanup_on_drop").next())
        .expect("play child terminal finalization implementation");

    assert!(terminate.contains("PlayProcessTree::Terminated => Ok(())"));
    assert!(terminate.contains("self.tree = PlayProcessTree::Terminated"));
    assert!(!finalize.contains("self.tree"));
    assert!(finalize.contains("play snapshot cleanup remains pending"));
}

#[test]
fn start_failure_retains_every_unfinished_cleanup_owner() {
    let source = include_str!("../child.rs");

    assert!(source.contains("enum PlayProcessCleanup"));
    assert!(source.contains("CaptureFailed(PlayOutputCaptureError)"));
    assert!(source.contains("cleanup: Some(PlayProcessCleanup::Child(child))"));
    assert!(source.contains("cleanup: Some(PlayProcessCleanup::Scene(scene))"));
    assert!(!source.contains("terminate_untracked_spawn"));
    assert!(!source.contains("terminate_and_reap"));
}
