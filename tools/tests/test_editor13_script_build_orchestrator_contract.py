from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def source(relative: str) -> str:
    path = ROOT / relative
    return path.read_text(encoding="utf-8") if path.exists() else ""


class Editor13ScriptBuildOrchestratorContractTests(unittest.TestCase):
    def test_core_exposes_one_folder_backed_script_build_owner(self) -> None:
        core = source("zircon_editor/src/core/mod.rs")
        module = source("zircon_editor/src/core/script_build/mod.rs")

        self.assertIn("pub mod script_build;", core)
        for required in [
            "ScriptBuildCompletion",
            "ScriptBuildEnqueueError",
            "ScriptBuildOrchestrator",
            "ScriptBuildOutcome",
            "ScriptBuildPhase",
            "ScriptBuildRequest",
            "ScriptBuildRequestId",
            "ScriptBuildSnapshot",
            "ScriptBuildStep",
            "ScriptBuildStepDispatch",
            "ScriptBuildTrigger",
        ]:
            self.assertIn(required, module)

    def test_request_contract_names_all_three_trigger_sources(self) -> None:
        request = source("zircon_editor/src/core/script_build/request.rs")

        self.assertIn("pub enum ScriptBuildTrigger", request)
        for trigger in ["Watch", "Command", "Play"]:
            self.assertIn(trigger, request)
        self.assertIn("play_after_build", request)
        self.assertIn("Vec<PathBuf>", request)
        for step in ["CompileModules", "ValidateLedger", "RefreshBindings"]:
            self.assertIn(step, request)

    def test_orchestrator_owns_debounce_queue_and_failure_stop_policy(self) -> None:
        orchestrator = source("zircon_editor/src/core/script_build/orchestrator.rs")

        for required in [
            "DEFAULT_SCRIPT_WATCH_DEBOUNCE_MS",
            "DEFAULT_SCRIPT_WATCH_MAX_LATENCY_MS",
            "MAX_INCREMENTAL_SCRIPT_WATCH_PATHS",
            "MAX_INCREMENTAL_SCRIPT_WATCH_PATH_BYTES",
            "BTreeSet<PathBuf>",
            "queued_request: Option<ScriptBuildRequest>",
            "watch_first_observed_at_ms",
            "saturating_add",
            ".min(hard_deadline_ms)",
            "merge_queued_request",
            "pending_watch_paths.clear()",
            "queued_request.take()",
        ]:
            self.assertIn(required, orchestrator)
        self.assertNotIn("VecDeque<ScriptBuildRequest>", orchestrator)

        request = source("zircon_editor/src/core/script_build/request.rs")
        self.assertIn("promote_trigger", request)
        self.assertIn("ScriptBuildOutcome::Cancelled", source("zircon_editor/src/core/script_build/tests.rs"))

    def test_async_completion_is_bound_to_the_dispatched_step(self) -> None:
        orchestrator = source("zircon_editor/src/core/script_build/orchestrator.rs")

        self.assertIn("StepMismatch", orchestrator)
        self.assertIn("dispatch: ScriptBuildStepDispatch", orchestrator)
        self.assertNotIn("dispatch: &ScriptBuildStepDispatch", orchestrator)
        self.assertIn("reserve_request_id()?", orchestrator)
        self.assertNotIn("pub fn complete(\n        &mut self,\n        request_id:", orchestrator)

        request = source("zircon_editor/src/core/script_build/request.rs")
        dispatch_declaration = request.split("pub struct ScriptBuildStepDispatch", maxsplit=1)[0]
        dispatch_derives = dispatch_declaration.rsplit("#[derive(", maxsplit=1)[-1].split(")]", maxsplit=1)[0]
        self.assertNotIn("Clone", dispatch_derives)

    def test_behavior_suite_covers_batching_ordering_and_failure_stop(self) -> None:
        tests = source("zircon_editor/src/core/script_build/tests.rs")

        for test_name in [
            "watch_changes_slide_within_first_event_latency_and_deduplicate_paths",
            "continuous_watch_storm_honors_first_event_max_latency",
            "default_watch_max_latency_is_hard_bounded",
            "command_flushes_pending_watch_changes_into_one_request",
            "watch_batch_over_incremental_limit_requests_full_module_compile",
            "incremental_limit_keeps_twenty_sorted_paths",
            "watch_path_byte_budget_falls_back_to_full_rebuild",
            "play_flushes_pending_watch_changes_into_one_request",
            "duplicate_command_and_play_share_the_active_generation_and_latest_play_intent",
            "newer_source_changes_coalesce_into_one_queued_play_generation",
            "million_explicit_requests_keep_one_single_flight_generation",
            "full_rebuild_expiry_then_later_watch_change_uses_next_generation",
            "successful_build_dispatches_compile_validate_refresh_in_order",
            "failure_drops_queued_and_debouncing_followups",
            "cancellation_is_an_explicit_outcome_and_drops_play_resume_intent",
            "wrong_completion_id_preserves_the_active_request",
            "stale_step_completion_is_rejected_after_next_step_dispatch",
            "request_id_exhaustion_preserves_pending_watch_state",
        ]:
            self.assertIn(f"fn {test_name}", tests)

    def test_runtime_interface_exposes_typed_script_diagnostics(self) -> None:
        interface = source("zircon_runtime_interface/src/lib.rs")
        diagnostics = source(
            "zircon_runtime_interface/src/script_diagnostics/mod.rs"
        )

        self.assertIn("pub mod script_diagnostics;", interface)
        for required in [
            "ScriptDiagnostic",
            "ScriptDiagnosticSeverity",
            "ScriptSourceLocation",
        ]:
            self.assertIn(required, interface)
            self.assertIn(required, diagnostics)
        for severity in ["Info", "Warning", "Error"]:
            self.assertIn(severity, diagnostics)
        for field in ["code", "module", "message", "location", "path", "line", "column"]:
            self.assertIn(f"pub {field}:", diagnostics)
        self.assertIn("Serialize", diagnostics)
        self.assertIn("Deserialize", diagnostics)

    def test_diagnostic_sink_projects_to_the_canonical_bounded_log(self) -> None:
        sink = source("zircon_editor/src/core/script_build/diagnostics_sink.rs")

        for required in [
            "Arc<EditorLogService>",
            "LogSource::script_build()",
            "ScriptDiagnosticSeverity::Info => LogSeverity::Info",
            "ScriptDiagnosticSeverity::Warning => LogSeverity::Warning",
            "ScriptDiagnosticSeverity::Error => LogSeverity::Error",
            "LogJump::script_location",
            "generation: completion.generation()",
            "request_id: completion.request_id()",
            "step_index: completion.completed_step_index()",
            "key < cursor.key",
            "key == cursor.key",
        ]:
            self.assertIn(required, sink)
        self.assertNotIn("Vec<Log", sink)
        self.assertNotIn("VecDeque", sink)
        self.assertNotIn("HashMap", sink)
        self.assertNotIn("HashSet", sink)

    def test_diagnostic_behavior_suite_covers_projection_and_boundedness(self) -> None:
        tests = source("zircon_editor/src/core/script_build/tests.rs")

        for test_name in [
            "accepted_compile_diagnostics_project_severity_module_and_source_jump",
            "stale_completion_cannot_produce_a_diagnostic_projection_fact",
            "replayed_completion_does_not_duplicate_diagnostics",
            "delayed_accepted_completion_is_stale_after_a_new_generation_is_projected",
            "compile_failure_logs_before_refresh_and_stops_the_request",
            "diagnostic_storm_uses_the_canonical_bounded_log_store_only",
        ]:
            self.assertIn(f"fn {test_name}", tests)


if __name__ == "__main__":
    unittest.main()
