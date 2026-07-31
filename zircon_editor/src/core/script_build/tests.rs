use std::path::PathBuf;

use super::{
    ScriptBuildCompletionError, ScriptBuildEnqueueError, ScriptBuildOrchestrator,
    ScriptBuildOutcome, ScriptBuildPhase, ScriptBuildRequest, ScriptBuildStep,
    ScriptBuildStepDispatch, ScriptBuildTrigger, MAX_INCREMENTAL_SCRIPT_WATCH_PATHS,
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
fn watch_changes_slide_the_debounce_deadline_and_deduplicate_paths() {
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
fn script_build_snapshots_share_the_last_outcome() {
    let source = include_str!("orchestrator.rs");

    assert!(source.contains("last_outcome: Option<Arc<ScriptBuildOutcome>>"));
}

#[test]
fn play_waits_for_the_active_build_and_resumes_only_after_success() {
    let mut orchestrator = ScriptBuildOrchestrator::default();
    let command_id = orchestrator.enqueue_command().unwrap();
    let command = ready(&mut orchestrator, 0);
    assert_eq!(command.request_id(), command_id);

    let play_id = orchestrator.enqueue_play().unwrap();
    assert_eq!(orchestrator.snapshot().queued_request_count(), 1);
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
        assert!(!completion.resume_play());
    }

    let play = ready(&mut orchestrator, 0);
    assert_eq!(play.request_id(), play_id);
    assert_eq!(play.trigger(), ScriptBuildTrigger::Play);
    assert!(play.play_after_build());

    let mut play_step = Some(play);
    for step_index in 0..=2 {
        let current_step = play_step
            .take()
            .unwrap_or_else(|| ready(&mut orchestrator, 0));
        assert_eq!(current_step.step_index(), step_index);
        let completion = orchestrator
            .complete(current_step, ScriptBuildOutcome::Succeeded)
            .unwrap();
        assert_eq!(completion.request_completed(), step_index == 2);
        assert_eq!(completion.resume_play(), step_index == 2);
    }
    assert_eq!(orchestrator.snapshot().phase(), ScriptBuildPhase::Succeeded);
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
    orchestrator.enqueue_command().unwrap();
    orchestrator.enqueue_play().unwrap();
    orchestrator.notify_watch_change("Scripts/ChangedDuringBuild.zr", 50);

    let completion = orchestrator
        .complete(
            active_dispatch,
            ScriptBuildOutcome::Failed {
                summary: "compile error".to_string(),
            },
        )
        .unwrap();

    assert_eq!(completion.dropped_queued_request_count(), 2);
    assert_eq!(completion.dropped_watch_path_count(), 1);
    assert!(!completion.resume_play());
    assert_eq!(orchestrator.snapshot().phase(), ScriptBuildPhase::Failed);
    assert!(orchestrator.take_ready(u64::MAX).unwrap().is_none());
}

#[test]
fn wrong_completion_id_preserves_the_active_request() {
    let mut orchestrator = ScriptBuildOrchestrator::default();
    let active_id = orchestrator.enqueue_command().unwrap();
    let _active_dispatch = ready(&mut orchestrator, 0);
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
