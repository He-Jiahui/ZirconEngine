"""Static contract for task panic-message preservation and failure progress."""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCOPE = ROOT / "zircon_runtime/src/core/runtime/tasks/task_graph/scope.rs"
HANDLE = ROOT / "zircon_runtime/src/core/runtime/tasks/task_graph/task_handle.rs"
PANIC_TESTS = ROOT / "zircon_runtime/src/core/runtime/tasks/task_graph/scope/tests/panic_payload.rs"
SCOPE_TESTS = ROOT / "zircon_runtime/src/core/runtime/tasks/task_graph/scope/tests/cases.rs"


class RuntimeTaskFailureProgressContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.scope = SCOPE.read_text(encoding="utf-8")
        cls.handle = HANDLE.read_text(encoding="utf-8")
        cls.panic_tests = PANIC_TESTS.read_text(encoding="utf-8")
        cls.scope_tests = SCOPE_TESTS.read_text(encoding="utf-8")

    def test_panic_payload_consumes_box_and_preserves_owned_static_and_fallback_values(self) -> None:
        body = self.scope.split("pub(super) fn panic_payload_message(", 1)[1].split(
            "#[cfg(test)]", 1
        )[0]
        for anchor in (
            "Box<dyn std::any::Any + Send>",
            "payload.downcast::<String>()",
            "Err(payload) => payload",
            "payload.downcast::<&str>()",
            "(*message).to_owned()",
            '"non-string panic payload".to_owned()',
        ):
            self.assertIn(anchor, body, anchor)
        self.assertNotIn("&payload", body)

    def test_scoped_and_detached_execution_publish_failure_message_through_one_helper(self) -> None:
        run = self.scope.split("fn run(", 1)[1].split("    fn begin(", 1)[0]
        detached = self.handle.split("fn run_detached(", 1)[1].split("\n    }\n}", 1)[0]
        for source, label in ((run, "scoped"), (detached, "detached")):
            for anchor in (
                "catch_unwind(AssertUnwindSafe",
                "Err(payload)",
                "panic_payload_message(payload)",
            ):
                self.assertIn(anchor, source, f"{label}: {anchor}")
        self.assertIn("JobExecutionOutcome::Panicked(Arc::from(", detached)
        self.assertIn("self.finish(task_id, Some(message))", run)

    def test_failure_retirement_updates_census_and_dependency_failure_does_not_run_user_work(self) -> None:
        retire = self.scope.split("fn retire(", 1)[1].split("    fn retire_prelaunch_failure", 1)[0]
        for anchor in (
            "JobExecutionOutcome::Panicked(_) =>",
            "scope.failed = scope.failed.saturating_add(1);",
            "JobExecutionOutcome::Cancelled =>",
            "scope.cancelled = scope.cancelled.saturating_add(1);",
            "JobExecutionOutcome::Completed =>",
            "scope.completed = scope.completed.saturating_add(1);",
            "self.quiescent.notify_all();",
        ):
            self.assertIn(anchor, retire, anchor)
        for anchor in (
            "direct_submit_after_propagates_dependency_failure_without_running_user_work",
            "canonical_dependency_failure_retires_scope_admission_without_running_user_work",
            "assert!(ran_rx.try_recv().is_err());",
            "assert_eq!(scheduled.status().state, TaskState::Failed);",
        ):
            self.assertIn(anchor, self.scope_tests, anchor)

    def test_panic_regressions_cover_owned_static_non_string_detached_and_dependency_cases(self) -> None:
        for name in (
            "astra_m1_owned_panic_message_moves_its_original_buffer",
            "astra_m1_scoped_and_detached_tasks_preserve_panic_messages",
            "astra_m1_failed_dependency_preserves_original_cause_without_running_dependent",
            "detached_panic_keeps_status_and_dependency_fence_consistent",
        ):
            self.assertIn(f"fn {name}(", self.panic_tests + self.scope_tests, name)
        for anchor in (
            '"static failure"',
            '"owned failure"',
            '"non-string panic payload"',
            '"original dependency failure"',
            "TaskState::Failed",
            "scope.census().failed",
        ):
            self.assertIn(anchor, self.panic_tests + self.scope_tests, anchor)


if __name__ == "__main__":
    unittest.main()
