#[test]
fn compiled_frame_tracks_scene_submission_after_finishing_the_receipt() {
    let source = include_str!("../frame_submission_owner.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("compiled frame owner must retain a test boundary");
    let finish = production
        .find("submission_transaction.finish(scene_submission)")
        .expect("compiled frame must finish its submission receipt");
    let track = production
        .find(".track(frame_generation, scene_submission)")
        .expect("compiled frame must track scene completion");

    assert!(finish < track);
    assert_eq!(
        production
            .matches("self.poll_frame_submission_completions()?")
            .count(),
        1
    );
    assert!(production.contains("drain_pipeline_creation_diagnostics();"));
    assert!(!production.contains("drain_pipeline_creation_diagnostics(&self.backend.device)"));
}

#[test]
fn pre_submit_failure_discards_history_whose_clear_never_reached_the_scene_packet() {
    let source = include_str!("../frame_submission_owner.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("frame submission owner test boundary");
    let prepared = production
        .find("history_initialization_command_buffer.is_some()")
        .expect("history clear preparation receipt");
    let render = production
        .find("core.render_compiled_scene(")
        .expect("compiled scene boundary");
    let failure = production
        .find("if history_initialization_needs_abort_cleanup")
        .expect("pre-submit history cleanup");
    let remove = production[failure..]
        .find("self.history_targets.remove(&handle)")
        .map(|offset| failure + offset)
        .expect("unsubmitted history must be removed");

    assert!(prepared < render);
    assert!(render < failure);
    assert!(failure < remove);
    assert!(production.contains("GraphicsError::FrameFailedAfterSceneSubmission"));
}
