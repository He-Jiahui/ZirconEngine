use std::path::PathBuf;
use std::sync::Arc;

use zircon_runtime_interface::{ScriptDiagnostic, ScriptDiagnosticSeverity, ScriptSourceLocation};

use crate::core::logging::{
    EditorLogConfig, EditorLogService, LogChannel, LogFilter, LogJumpTarget, LogSeverity,
};

use super::{
    ScriptBuildCompletionError, ScriptBuildDiagnosticsSink, ScriptBuildEnqueueError,
    ScriptBuildOrchestrator, ScriptBuildOutcome, ScriptBuildPhase, ScriptBuildRequest,
    ScriptBuildStep, ScriptBuildStepDispatch, ScriptBuildTrigger,
    DEFAULT_SCRIPT_WATCH_MAX_LATENCY_MS, MAX_INCREMENTAL_SCRIPT_WATCH_PATHS,
    MAX_INCREMENTAL_SCRIPT_WATCH_PATH_BYTES,
};

fn paths(values: &[&str]) -> Vec<PathBuf> {
    values.iter().map(PathBuf::from).collect()
}

fn ready(orchestrator: &mut ScriptBuildOrchestrator, now_ms: u64) -> ScriptBuildStepDispatch {
    orchestrator
        .take_ready(now_ms)
        .expect("request id allocation should remain available")
        .expect("a script build step should be ready")
}

#[test]
fn watch_changes_slide_within_first_event_latency_and_deduplicate_paths() {
    let mut orchestrator = ScriptBuildOrchestrator::new(300);

    orchestrator.notify_watch_change("Scripts/A.zr", 100);
    orchestrator.notify_watch_change("Scripts/B.zr", 200);
    orchestrator.notify_watch_change("Scripts/A.zr", 250);

    let snapshot = orchestrator.snapshot();
    assert_eq!(snapshot.phase(), ScriptBuildPhase::Debouncing);
    assert_eq!(snapshot.pending_watch_path_count(), 2);
    assert_eq!(snapshot.watch_deadline_ms(), Some(550));
    assert!(orchestrator.take_ready(549).unwrap().is_none());

    let dispatch = ready(&mut orchestrator, 550);
    assert_eq!(dispatch.trigger(), ScriptBuildTrigger::Watch);
    assert_eq!(
        dispatch.step(),
        &ScriptBuildStep::CompileModules(paths(&["Scripts/A.zr", "Scripts/B.zr"]))
    );
    assert!(!dispatch.play_after_build());
}

#[test]
fn continuous_watch_storm_honors_first_event_max_latency() {
    let mut orchestrator = ScriptBuildOrchestrator::with_debounce_limits(300, 1_000);

    for observed_at_ms in (0..=900).step_by(100) {
        orchestrator.notify_watch_change("Scripts/ContinuouslyChanging.zr", observed_at_ms);
    }

    let snapshot = orchestrator.snapshot();
    assert_eq!(snapshot.watch_first_observed_at_ms(), Some(0));
    assert_eq!(snapshot.watch_deadline_ms(), Some(1_000));
    assert!(orchestrator.take_ready(999).unwrap().is_none());
    assert_eq!(
        ready(&mut orchestrator, 1_000).trigger(),
        ScriptBuildTrigger::Watch
    );
}

#[test]
fn default_watch_max_latency_is_hard_bounded() {
    assert!(DEFAULT_SCRIPT_WATCH_MAX_LATENCY_MS >= 300);
    let mut orchestrator = ScriptBuildOrchestrator::default();
    orchestrator.notify_watch_change("Scripts/First.zr", 10);
    orchestrator.notify_watch_change("Scripts/Late.zr", u64::MAX - 10);

    assert_eq!(
        orchestrator.snapshot().watch_deadline_ms(),
        Some(10 + DEFAULT_SCRIPT_WATCH_MAX_LATENCY_MS)
    );
}

#[test]
fn command_flushes_pending_watch_changes_into_one_request() {
    let mut orchestrator = ScriptBuildOrchestrator::default();
    orchestrator.notify_watch_change("Scripts/Player.zr", 100);

    let command_id = orchestrator.enqueue_command().unwrap();
    let dispatch = ready(&mut orchestrator, 100);

    assert_eq!(dispatch.request_id(), command_id);
    assert_eq!(dispatch.trigger(), ScriptBuildTrigger::Command);
    assert_eq!(
        dispatch.step(),
        &ScriptBuildStep::CompileModules(paths(&["Scripts/Player.zr"]))
    );
    assert_eq!(orchestrator.snapshot().pending_watch_path_count(), 0);
}

#[test]
fn play_flushes_pending_watch_changes_into_one_request() {
    let mut orchestrator = ScriptBuildOrchestrator::default();
    orchestrator.notify_watch_change("Scripts/PlayDependency.zr", 100);

    let play_id = orchestrator.enqueue_play().unwrap();
    let dispatch = ready(&mut orchestrator, 100);

    assert_eq!(dispatch.request_id(), play_id);
    assert_eq!(dispatch.trigger(), ScriptBuildTrigger::Play);
    assert_eq!(
        dispatch.step(),
        &ScriptBuildStep::CompileModules(paths(&["Scripts/PlayDependency.zr"]))
    );
    assert!(dispatch.play_after_build());
    assert_eq!(orchestrator.snapshot().pending_watch_path_count(), 0);
}

#[test]
fn incremental_limit_keeps_twenty_sorted_paths() {
    let mut orchestrator = ScriptBuildOrchestrator::default();
    for index in (0..MAX_INCREMENTAL_SCRIPT_WATCH_PATHS).rev() {
        orchestrator.notify_watch_change(format!("Scripts/Module{index:02}.zr"), 100);
    }

    assert_eq!(
        orchestrator.snapshot().pending_watch_path_count(),
        MAX_INCREMENTAL_SCRIPT_WATCH_PATHS
    );
    let dispatch = ready(&mut orchestrator, 400);
    let expected = (0..MAX_INCREMENTAL_SCRIPT_WATCH_PATHS)
        .map(|index| PathBuf::from(format!("Scripts/Module{index:02}.zr")))
        .collect::<Vec<_>>();

    assert_eq!(dispatch.step(), &ScriptBuildStep::CompileModules(expected));
}

#[test]
fn watch_batch_over_incremental_limit_requests_full_module_compile() {
    let mut orchestrator = ScriptBuildOrchestrator::default();
    for index in 0..=20 {
        orchestrator.notify_watch_change(format!("Scripts/Module{index:02}.zr"), 100);
    }

    let dispatch = ready(&mut orchestrator, 400);

    assert_eq!(dispatch.trigger(), ScriptBuildTrigger::Watch);
    assert_eq!(
        dispatch.step(),
        &ScriptBuildStep::CompileModules(Vec::new())
    );
}

#[test]
fn watch_path_storage_stops_growing_after_full_rebuild_is_required() {
    let mut orchestrator = ScriptBuildOrchestrator::default();
    for index in 0..10_000 {
        orchestrator.notify_watch_change(format!("Scripts/Module{index:05}.zr"), 100);
    }

    assert_eq!(
        orchestrator.snapshot().pending_watch_path_count(),
        MAX_INCREMENTAL_SCRIPT_WATCH_PATHS + 1
    );
    let dispatch = ready(&mut orchestrator, 400);
    assert_eq!(
        dispatch.step(),
        &ScriptBuildStep::CompileModules(Vec::new())
    );
}

#[test]
fn watch_path_byte_budget_falls_back_to_full_rebuild() {
    let mut orchestrator = ScriptBuildOrchestrator::default();
    let oversized_path = format!(
        "Scripts/{}.zr",
        "x".repeat(MAX_INCREMENTAL_SCRIPT_WATCH_PATH_BYTES)
    );

    orchestrator.notify_watch_change(oversized_path, 100);

    assert_eq!(
        orchestrator.snapshot().pending_watch_path_count(),
        MAX_INCREMENTAL_SCRIPT_WATCH_PATHS + 1
    );
    assert_eq!(
        ready(&mut orchestrator, 400).step(),
        &ScriptBuildStep::CompileModules(Vec::new())
    );
}

#[test]
fn script_build_snapshots_share_the_last_outcome() {
    let source = include_str!("orchestrator.rs");

    assert!(source.contains("last_outcome: Option<Arc<ScriptBuildOutcome>>"));
}

#[test]
fn duplicate_command_and_play_share_the_active_generation_and_latest_play_intent() {
    let mut orchestrator = ScriptBuildOrchestrator::default();
    let command_id = orchestrator.enqueue_command().unwrap();
    let command = ready(&mut orchestrator, 0);
    assert_eq!(command.request_id(), command_id);

    assert_eq!(orchestrator.enqueue_command().unwrap(), command_id);
    let play_id = orchestrator.enqueue_play().unwrap();
    assert_eq!(play_id, command_id);
    assert_eq!(orchestrator.snapshot().queued_request_count(), 0);
    assert!(orchestrator.take_ready(0).unwrap().is_none());

    let first_command_completion = orchestrator
        .complete(command, ScriptBuildOutcome::Succeeded)
        .unwrap();
    assert!(!first_command_completion.request_completed());
    for step_index in 1..=2 {
        let command_step = ready(&mut orchestrator, 0);
        assert_eq!(command_step.request_id(), command_id);
        assert_eq!(command_step.step_index(), step_index);
        let completion = orchestrator
            .complete(command_step, ScriptBuildOutcome::Succeeded)
            .unwrap();
        assert_eq!(completion.request_completed(), step_index == 2);
        assert_eq!(completion.resume_play(), step_index == 2);
    }
    assert_eq!(orchestrator.snapshot().phase(), ScriptBuildPhase::Succeeded);
}

#[test]
fn newer_source_changes_coalesce_into_one_queued_play_generation() {
    let mut orchestrator = ScriptBuildOrchestrator::default();
    let active_id = orchestrator.enqueue_command().unwrap();
    let active_compile = ready(&mut orchestrator, 0);

    orchestrator.notify_watch_change("Scripts/A.zr", 10);
    let queued_id = orchestrator.enqueue_command().unwrap();
    orchestrator.notify_watch_change("Scripts/B.zr", 20);
    assert_eq!(orchestrator.enqueue_play().unwrap(), queued_id);
    assert_ne!(queued_id, active_id);
    assert_eq!(orchestrator.snapshot().queued_request_count(), 1);
    assert_eq!(orchestrator.snapshot().pending_watch_path_count(), 0);

    orchestrator
        .complete(active_compile, ScriptBuildOutcome::Succeeded)
        .unwrap();
    for _ in 1..=2 {
        let dispatch = ready(&mut orchestrator, 20);
        orchestrator
            .complete(dispatch, ScriptBuildOutcome::Succeeded)
            .unwrap();
    }

    let queued_compile = ready(&mut orchestrator, 20);
    assert_eq!(queued_compile.request_id(), queued_id);
    assert_eq!(queued_compile.trigger(), ScriptBuildTrigger::Play);
    assert_eq!(
        queued_compile.step(),
        &ScriptBuildStep::CompileModules(paths(&["Scripts/A.zr", "Scripts/B.zr"]))
    );
    assert!(queued_compile.play_after_build());
}

#[test]
fn million_explicit_requests_keep_one_single_flight_generation() {
    let mut orchestrator = ScriptBuildOrchestrator::default();
    let generation_id = orchestrator.enqueue_command().unwrap();

    for index in 0..1_000_000 {
        let request_id = if index % 2 == 0 {
            orchestrator.enqueue_command().unwrap()
        } else {
            orchestrator.enqueue_play().unwrap()
        };
        assert_eq!(request_id, generation_id);
    }

    assert_eq!(orchestrator.snapshot().queued_request_count(), 1);
    let dispatch = ready(&mut orchestrator, 0);
    assert_eq!(dispatch.request_id(), generation_id);
    assert_eq!(dispatch.trigger(), ScriptBuildTrigger::Play);
    assert!(dispatch.play_after_build());
}

#[test]
fn full_rebuild_expiry_then_later_watch_change_uses_next_generation() {
    let mut orchestrator = ScriptBuildOrchestrator::default();
    for index in 0..=MAX_INCREMENTAL_SCRIPT_WATCH_PATHS {
        orchestrator.notify_watch_change(format!("Scripts/Module{index:02}.zr"), 0);
    }

    let first_compile = ready(&mut orchestrator, 300);
    let first_generation = first_compile.generation();
    assert_eq!(
        first_compile.step(),
        &ScriptBuildStep::CompileModules(Vec::new())
    );
    orchestrator.notify_watch_change("Scripts/Later.zr", 301);
    orchestrator
        .complete(first_compile, ScriptBuildOutcome::Succeeded)
        .unwrap();
    for _ in 1..=2 {
        let dispatch = ready(&mut orchestrator, 301);
        orchestrator
            .complete(dispatch, ScriptBuildOutcome::Succeeded)
            .unwrap();
    }

    let later_compile = ready(&mut orchestrator, 601);
    assert_ne!(later_compile.generation(), first_generation);
    assert_eq!(
        later_compile.step(),
        &ScriptBuildStep::CompileModules(paths(&["Scripts/Later.zr"]))
    );
}

#[test]
fn successful_build_dispatches_compile_validate_refresh_in_order() {
    let mut orchestrator = ScriptBuildOrchestrator::default();
    let request_id = orchestrator.enqueue_command().unwrap();
    let mut dispatched = Vec::new();

    for step_index in 0..=2 {
        let dispatch = ready(&mut orchestrator, 0);
        assert_eq!(dispatch.request_id(), request_id);
        assert_eq!(dispatch.step_index(), step_index);
        dispatched.push(dispatch.step().clone());
        let completion = orchestrator
            .complete(dispatch, ScriptBuildOutcome::Succeeded)
            .unwrap();
        assert_eq!(completion.completed_step_index(), step_index);
        assert_eq!(completion.request_completed(), step_index == 2);
    }

    assert_eq!(
        dispatched,
        vec![
            ScriptBuildStep::CompileModules(Vec::new()),
            ScriptBuildStep::ValidateLedger,
            ScriptBuildStep::RefreshBindings,
        ]
    );
}

#[test]
fn failure_drops_queued_and_debouncing_followups() {
    let mut orchestrator = ScriptBuildOrchestrator::default();
    orchestrator.enqueue_command().unwrap();
    let active_dispatch = ready(&mut orchestrator, 0);
    orchestrator.notify_watch_change("Scripts/Queued.zr", 40);
    orchestrator.enqueue_command().unwrap();
    orchestrator.enqueue_play().unwrap();
    orchestrator.notify_watch_change("Scripts/Debouncing.zr", 50);

    let completion = orchestrator
        .complete(
            active_dispatch,
            ScriptBuildOutcome::Failed {
                summary: "compile error".to_string(),
            },
        )
        .unwrap();

    assert_eq!(completion.dropped_queued_request_count(), 1);
    assert_eq!(completion.dropped_watch_path_count(), 1);
    assert!(!completion.resume_play());
    assert_eq!(orchestrator.snapshot().phase(), ScriptBuildPhase::Failed);
    assert!(orchestrator.take_ready(u64::MAX).unwrap().is_none());
}

#[test]
fn cancellation_is_an_explicit_outcome_and_drops_play_resume_intent() {
    let mut orchestrator = ScriptBuildOrchestrator::default();
    orchestrator.enqueue_command().unwrap();
    let active_dispatch = ready(&mut orchestrator, 0);
    orchestrator.notify_watch_change("Scripts/Queued.zr", 10);
    orchestrator.enqueue_play().unwrap();

    let completion = orchestrator
        .complete(
            active_dispatch,
            ScriptBuildOutcome::Cancelled {
                reason: "editor shutdown".to_string(),
            },
        )
        .unwrap();

    assert!(matches!(
        completion.outcome(),
        ScriptBuildOutcome::Cancelled { reason } if reason == "editor shutdown"
    ));
    assert_eq!(completion.dropped_queued_request_count(), 1);
    assert!(!completion.resume_play());
    assert_eq!(orchestrator.snapshot().phase(), ScriptBuildPhase::Cancelled);
}

#[test]
fn wrong_completion_id_preserves_the_active_request() {
    let mut orchestrator = ScriptBuildOrchestrator::default();
    let active_id = orchestrator.enqueue_command().unwrap();
    let _active_dispatch = ready(&mut orchestrator, 0);
    orchestrator.notify_watch_change("Scripts/NewGeneration.zr", 1);
    let queued_id = orchestrator.enqueue_play().unwrap();
    let queued_request = ScriptBuildRequest::new(queued_id, ScriptBuildTrigger::Play, Vec::new());
    let queued_dispatch = ScriptBuildStepDispatch::new(&queued_request, 0);

    let error = orchestrator
        .complete(queued_dispatch, ScriptBuildOutcome::Succeeded)
        .expect_err("a completion must identify the active request");

    assert_eq!(
        error,
        ScriptBuildCompletionError::RequestMismatch {
            expected: active_id,
            actual: queued_id,
        }
    );
    assert_eq!(orchestrator.snapshot().active_request_id(), Some(active_id));
    assert_eq!(orchestrator.snapshot().queued_request_count(), 1);
}

#[test]
fn stale_step_completion_is_rejected_after_next_step_dispatch() {
    let mut orchestrator = ScriptBuildOrchestrator::default();
    let request_id = orchestrator.enqueue_command().unwrap();
    let stale_request =
        ScriptBuildRequest::new(request_id, ScriptBuildTrigger::Command, Vec::new());
    let stale_compile_dispatch = ScriptBuildStepDispatch::new(&stale_request, 0);
    let compile_dispatch = ready(&mut orchestrator, 0);
    orchestrator
        .complete(compile_dispatch, ScriptBuildOutcome::Succeeded)
        .unwrap();
    let validate_dispatch = ready(&mut orchestrator, 0);

    let error = orchestrator
        .complete(stale_compile_dispatch, ScriptBuildOutcome::Succeeded)
        .expect_err("a stale compile completion must not advance validation");

    assert_eq!(
        error,
        ScriptBuildCompletionError::StepMismatch {
            request_id,
            expected: 1,
            actual: 0,
        }
    );
    assert_eq!(
        orchestrator.snapshot().active_request_id(),
        Some(request_id)
    );
    let completion = orchestrator
        .complete(validate_dispatch, ScriptBuildOutcome::Succeeded)
        .expect("the current validation completion remains valid");
    assert_eq!(completion.completed_step_index(), 1);
    assert!(!completion.request_completed());
}

#[test]
fn request_id_exhaustion_preserves_pending_watch_state() {
    let mut orchestrator = ScriptBuildOrchestrator::default();
    orchestrator.notify_watch_change("Scripts/Pending.zr", 100);
    orchestrator.exhaust_request_ids();
    let before = orchestrator.snapshot();

    assert_eq!(
        orchestrator.enqueue_command(),
        Err(ScriptBuildEnqueueError::RequestIdExhausted)
    );
    assert_eq!(orchestrator.snapshot(), before);
    assert_eq!(
        orchestrator.take_ready(400),
        Err(ScriptBuildEnqueueError::RequestIdExhausted)
    );
    assert_eq!(orchestrator.snapshot(), before);
    assert_eq!(before.phase(), ScriptBuildPhase::Debouncing);
    assert_eq!(before.pending_watch_path_count(), 1);
    assert_eq!(before.watch_deadline_ms(), Some(400));
}

fn diagnostic(
    severity: ScriptDiagnosticSeverity,
    code: &str,
    message: &str,
    location: Option<ScriptSourceLocation>,
) -> ScriptDiagnostic {
    ScriptDiagnostic::new(severity, code, "game.player", message, location)
}

fn bounded_log_service(entry_capacity: usize) -> Arc<EditorLogService> {
    Arc::new(EditorLogService::new(
        EditorLogConfig::new(entry_capacity, 256 * 1024).unwrap(),
    ))
}

#[test]
fn accepted_compile_diagnostics_project_severity_module_and_source_jump() {
    let service = bounded_log_service(8);
    let mut sink = ScriptBuildDiagnosticsSink::new(Arc::clone(&service));
    let mut orchestrator = ScriptBuildOrchestrator::default();
    orchestrator.enqueue_command().unwrap();
    let dispatch = ready(&mut orchestrator, 0);
    let completion = orchestrator
        .complete(dispatch, ScriptBuildOutcome::Succeeded)
        .unwrap();
    let diagnostics = [
        diagnostic(
            ScriptDiagnosticSeverity::Warning,
            "ZR1001",
            "unused local",
            None,
        ),
        diagnostic(
            ScriptDiagnosticSeverity::Error,
            "ZR2002",
            "type mismatch",
            Some(ScriptSourceLocation::new("res://scripts/player.zr", 12, 4)),
        ),
    ];

    let report = sink.project(&completion, &diagnostics, 41).unwrap();

    assert_eq!(report.emitted_count(), 2);
    assert_eq!(report.duplicate_count(), 0);
    assert!(!report.stale());
    let records = service.snapshot(&LogFilter::default());
    assert_eq!(records.len(), 2);
    assert_eq!(
        records[0].entry().source().channel(),
        LogChannel::ScriptBuild
    );
    assert_eq!(records[0].entry().severity(), LogSeverity::Warning);
    assert_eq!(records[1].entry().severity(), LogSeverity::Error);
    assert!(records[0].entry().message().contains("game.player"));
    assert!(records[1].entry().message().contains("ZR2002"));
    assert_eq!(records[1].entry().timestamp_frame(), 41);
    assert_eq!(
        records[1].entry().jump().map(|jump| jump.target()),
        Some(&LogJumpTarget::ScriptLocation {
            path: Arc::from("res://scripts/player.zr"),
            line: 12,
            column: 4,
        })
    );
}

#[test]
fn stale_completion_cannot_produce_a_diagnostic_projection_fact() {
    let service = bounded_log_service(8);
    let mut sink = ScriptBuildDiagnosticsSink::new(Arc::clone(&service));
    let mut orchestrator = ScriptBuildOrchestrator::default();
    let stale_id = orchestrator.enqueue_command().unwrap();
    let accepted_dispatch = ready(&mut orchestrator, 0);
    let stale_request = ScriptBuildRequest::new(stale_id, ScriptBuildTrigger::Command, Vec::new());
    let stale_dispatch = ScriptBuildStepDispatch::new(&stale_request, 0);
    orchestrator
        .complete(
            accepted_dispatch,
            ScriptBuildOutcome::Failed {
                summary: "stop".into(),
            },
        )
        .unwrap();
    orchestrator.enqueue_command().unwrap();
    let _current_dispatch = ready(&mut orchestrator, 0);

    let error = orchestrator
        .complete(stale_dispatch, ScriptBuildOutcome::Succeeded)
        .expect_err("a superseded completion must not produce an accepted completion fact");

    assert!(matches!(
        error,
        ScriptBuildCompletionError::RequestMismatch { .. }
    ));
    assert_eq!(sink.cursor_generation(), None);
    assert!(service.snapshot(&LogFilter::default()).is_empty());
}

#[test]
fn replayed_completion_does_not_duplicate_diagnostics() {
    let service = bounded_log_service(8);
    let mut sink = ScriptBuildDiagnosticsSink::new(Arc::clone(&service));
    let mut orchestrator = ScriptBuildOrchestrator::default();
    orchestrator.enqueue_command().unwrap();
    let dispatch = ready(&mut orchestrator, 0);
    let completion = orchestrator
        .complete(dispatch, ScriptBuildOutcome::Succeeded)
        .unwrap();
    let diagnostics = [diagnostic(
        ScriptDiagnosticSeverity::Warning,
        "ZR1001",
        "unused local",
        None,
    )];

    assert_eq!(
        sink.project(&completion, &diagnostics, 3)
            .unwrap()
            .emitted_count(),
        1
    );
    let replay = sink.project(&completion, &diagnostics, 3).unwrap();

    assert_eq!(replay.emitted_count(), 0);
    assert_eq!(replay.duplicate_count(), 1);
    assert!(!replay.stale());
    assert_eq!(service.snapshot(&LogFilter::default()).len(), 1);
}

#[test]
fn delayed_accepted_completion_is_stale_after_a_new_generation_is_projected() {
    let service = bounded_log_service(8);
    let mut sink = ScriptBuildDiagnosticsSink::new(Arc::clone(&service));
    let mut orchestrator = ScriptBuildOrchestrator::default();
    orchestrator.enqueue_command().unwrap();
    let dispatch = ready(&mut orchestrator, 0);
    let delayed_completion = orchestrator
        .complete(dispatch, ScriptBuildOutcome::Succeeded)
        .unwrap();
    let dispatch = ready(&mut orchestrator, 0);
    orchestrator
        .complete(dispatch, ScriptBuildOutcome::Succeeded)
        .unwrap();
    let dispatch = ready(&mut orchestrator, 0);
    orchestrator
        .complete(dispatch, ScriptBuildOutcome::Succeeded)
        .unwrap();
    orchestrator.enqueue_command().unwrap();
    let dispatch = ready(&mut orchestrator, 0);
    let current_completion = orchestrator
        .complete(dispatch, ScriptBuildOutcome::Succeeded)
        .unwrap();
    let diagnostic = diagnostic(
        ScriptDiagnosticSeverity::Warning,
        "ZR1001",
        "unused local",
        None,
    );
    sink.project(&current_completion, std::slice::from_ref(&diagnostic), 4)
        .unwrap();

    let stale = sink.project(&delayed_completion, &[diagnostic], 3).unwrap();

    assert!(stale.stale());
    assert_eq!(stale.emitted_count(), 0);
    assert_eq!(service.snapshot(&LogFilter::default()).len(), 1);
}

#[test]
fn compile_failure_logs_before_refresh_and_stops_the_request() {
    let service = bounded_log_service(8);
    let mut sink = ScriptBuildDiagnosticsSink::new(Arc::clone(&service));
    let mut orchestrator = ScriptBuildOrchestrator::default();
    orchestrator.enqueue_command().unwrap();
    let dispatch = ready(&mut orchestrator, 0);
    let completion = orchestrator
        .complete(
            dispatch,
            ScriptBuildOutcome::Failed {
                summary: "compile failed".into(),
            },
        )
        .unwrap();

    sink.project(
        &completion,
        &[diagnostic(
            ScriptDiagnosticSeverity::Error,
            "ZR2002",
            "type mismatch",
            None,
        )],
        7,
    )
    .unwrap();

    assert_eq!(orchestrator.snapshot().phase(), ScriptBuildPhase::Failed);
    assert!(orchestrator.take_ready(0).unwrap().is_none());
    let records = service.snapshot(&LogFilter::default());
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].entry().severity(), LogSeverity::Error);
}

#[test]
fn diagnostic_storm_uses_the_canonical_bounded_log_store_only() {
    let service = bounded_log_service(8);
    let mut sink = ScriptBuildDiagnosticsSink::new(Arc::clone(&service));
    let mut orchestrator = ScriptBuildOrchestrator::default();
    orchestrator.enqueue_command().unwrap();
    let dispatch = ready(&mut orchestrator, 0);
    let completion = orchestrator
        .complete(dispatch, ScriptBuildOutcome::Succeeded)
        .unwrap();
    let diagnostics = (0..256)
        .map(|index| {
            diagnostic(
                ScriptDiagnosticSeverity::Warning,
                "ZR1001",
                &format!("warning {index}"),
                None,
            )
        })
        .collect::<Vec<_>>();

    let report = sink.project(&completion, &diagnostics, 9).unwrap();

    assert_eq!(report.emitted_count(), diagnostics.len());
    assert_eq!(service.snapshot(&LogFilter::default()).len(), 8);
    assert_eq!(sink.cursor_generation(), Some(completion.generation()));
}
