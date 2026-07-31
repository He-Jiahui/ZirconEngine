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
            "MAX_INCREMENTAL_SCRIPT_WATCH_PATHS",
            "BTreeSet<PathBuf>",
            "VecDeque<ScriptBuildRequest>",
            "saturating_add",
            "pending_watch_paths.clear()",
            "queued_requests.clear()",
        ]:
            self.assertIn(required, orchestrator)

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
            "watch_changes_slide_the_debounce_deadline_and_deduplicate_paths",
            "command_flushes_pending_watch_changes_into_one_request",
            "watch_batch_over_incremental_limit_requests_full_module_compile",
            "incremental_limit_keeps_twenty_sorted_paths",
            "play_flushes_pending_watch_changes_into_one_request",
            "play_waits_for_the_active_build_and_resumes_only_after_success",
            "successful_build_dispatches_compile_validate_refresh_in_order",
            "failure_drops_queued_and_debouncing_followups",
            "wrong_completion_id_preserves_the_active_request",
            "stale_step_completion_is_rejected_after_next_step_dispatch",
            "request_id_exhaustion_preserves_pending_watch_state",
        ]:
            self.assertIn(f"fn {test_name}", tests)


if __name__ == "__main__":
    unittest.main()
