use crate::work_paths::viewer_test_artifact_root;

use super::{
    write_terminal_outcome, TerminalArtifactState, TerminalCleanupState, TerminalErrorCategory,
    TerminalOutcome, TerminalPhase, TerminalStatus,
};

#[test]
fn terminal_failures_have_stable_nonzero_exit_codes() {
    for category in [
        TerminalErrorCategory::Startup,
        TerminalErrorCategory::Platform,
        TerminalErrorCategory::SceneLoad,
        TerminalErrorCategory::Pipeline,
        TerminalErrorCategory::Rendering,
        TerminalErrorCategory::Artifact,
        TerminalErrorCategory::Capture,
        TerminalErrorCategory::Presentation,
        TerminalErrorCategory::EventLoop,
        TerminalErrorCategory::TaskShutdown,
    ] {
        let outcome =
            TerminalOutcome::failure(TerminalPhase::Render, category, "simulated failure");

        assert_eq!(outcome.status(), TerminalStatus::Failed);
        assert_ne!(outcome.exit_code(), 0, "{category:?}");
    }
}

#[test]
fn terminal_user_cancellation_is_distinct_from_success_and_failure() {
    let cancelled = TerminalOutcome::cancelled();

    assert_eq!(cancelled.status(), TerminalStatus::Cancelled);
    assert_ne!(
        cancelled.exit_code(),
        TerminalOutcome::succeeded().exit_code()
    );
    assert_ne!(
        cancelled.exit_code(),
        TerminalOutcome::failure(
            TerminalPhase::Render,
            TerminalErrorCategory::Rendering,
            "simulated failure",
        )
        .exit_code()
    );
}

#[test]
fn terminal_record_is_atomic_and_retains_outcome_context() {
    let root = viewer_test_artifact_root("terminal-outcome");
    let path = root.join("viewer-terminal-outcome.json");
    let previous = TerminalOutcome::cancelled();
    write_terminal_outcome(&path, &previous).expect("write initial terminal record");

    let outcome = TerminalOutcome::failure(
        TerminalPhase::ScreenshotWrite,
        TerminalErrorCategory::Artifact,
        "cannot publish ready PNG",
    )
    .with_source_chain([
        "zircon_shader_pbr_viewer",
        "environment_only_pbr_preview",
        "hdri:E:/fixtures/lakes.hdr",
    ])
    .with_artifacts(
        TerminalArtifactState::NotCommitted,
        TerminalArtifactState::NotRequested,
        TerminalArtifactState::NotRequested,
    )
    .with_cleanup(
        TerminalCleanupState::BackgroundLoaderShutdownTimedOut,
        Some("scene loader exceeded the shutdown drain deadline".to_string()),
    );
    write_terminal_outcome(&path, &outcome).expect("replace terminal record atomically");

    let record: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).expect("read terminal record"))
            .expect("terminal record JSON");
    assert_eq!(
        record["schema"],
        "zircon_shader_pbr_viewer_terminal_outcome_v1"
    );
    assert_eq!(record["status"], "failed");
    assert_eq!(record["phase"], "screenshot_write");
    assert_eq!(record["error_category"], "artifact");
    assert_eq!(record["screenshot_artifact"], "not_committed");
    assert_eq!(
        record["cleanup_state"],
        "background_loader_shutdown_timed_out"
    );
    assert_eq!(
        record["cleanup_error"],
        "scene loader exceeded the shutdown drain deadline"
    );
    assert_eq!(record["source_chain"][2], "hdri:E:/fixtures/lakes.hdr");

    let staged = std::fs::read_dir(&root)
        .expect("read terminal record parent")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|candidate| candidate != &path)
        .collect::<Vec<_>>();
    assert!(
        staged.is_empty(),
        "terminal write left visible staging files: {staged:?}"
    );

    std::fs::remove_dir_all(root).expect("remove terminal outcome fixture");
}

#[test]
fn shutdown_timeout_promotes_nonfatal_outcomes_to_a_task_shutdown_failure() {
    let outcome = TerminalOutcome::cancelled().with_cleanup(
        TerminalCleanupState::BackgroundLoaderShutdownTimedOut,
        Some("scene loader did not finish after cancellation".to_string()),
    );

    assert_eq!(outcome.status(), TerminalStatus::Failed);
    assert_eq!(outcome.phase, TerminalPhase::TaskShutdown);
    assert_eq!(
        outcome.error_category,
        Some(TerminalErrorCategory::TaskShutdown)
    );
    assert_eq!(outcome.exit_code(), 20);
    assert_eq!(
        outcome.cleanup_error.as_deref(),
        Some("scene loader did not finish after cancellation")
    );
}

#[test]
fn cleanup_failure_does_not_replace_an_existing_render_failure() {
    let outcome = TerminalOutcome::failure(
        TerminalPhase::Render,
        TerminalErrorCategory::Rendering,
        "environment-only PBR render failed",
    )
    .with_cleanup(
        TerminalCleanupState::BackgroundLoaderShutdownTimedOut,
        Some("scene loader did not finish after cancellation".to_string()),
    );

    assert_eq!(outcome.status(), TerminalStatus::Failed);
    assert_eq!(outcome.phase, TerminalPhase::Render);
    assert_eq!(
        outcome.error_category,
        Some(TerminalErrorCategory::Rendering)
    );
    assert_eq!(outcome.exit_code(), 15);
    assert_eq!(
        outcome.cleanup_error.as_deref(),
        Some("scene loader did not finish after cancellation")
    );
}
