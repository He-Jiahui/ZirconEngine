import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
TASKS = ROOT / "zircon_runtime/src/core/runtime/tasks"
JOB = TASKS / "job_handle.rs"
TASK_HANDLE = TASKS / "task_graph/task_handle.rs"
COMBINE_TESTS = TASKS / "job_handle/tests/astra_combine_tests.rs"


class RuntimeCombinedFenceContractTests(unittest.TestCase):
    def test_combine_is_empty_safe_and_counts_every_dependency(self) -> None:
        source = JOB.read_text(encoding="utf-8")

        for anchor in (
            "pub fn combine(handles: &[JobHandle]) -> Self",
            "if handles.is_empty()",
            "return Self::completed();",
            "Self::pending_with_wait_diagnostics(",
            "handles.len(),",
            "for handle in handles",
            "combined.combined_dependency_completed(",
            "combined",
        ):
            self.assertIn(anchor, source)

    def test_combine_reads_terminal_state_under_one_dependency_lock(self) -> None:
        source = JOB.read_text(encoding="utf-8")
        body = source.split("fn combine_with_wait_diagnostics", 1)[1].split("pub fn is_complete", 1)[0]

        for anchor in (
            "let terminal = {",
            "let mut inner = handle.node.lock_inner();",
            "if inner.lifecycle.is_terminal()",
            "inner.panic_message.clone()",
            "inner.lifecycle == TaskState::Cancelled",
            "let handle_for_callback = handle.clone();",
            "let combined_for_callback = combined.clone();",
            "inner.dependents.push(Box::new(move ||",
            "if let Some((panic_message, is_cancelled)) = terminal",
        ):
            self.assertIn(anchor, body)

        self.assertLess(body.index("let mut inner = handle.node.lock_inner()"), body.index("let handle_for_callback"))
        self.assertLess(body.index("inner.dependents.push"), body.index("if let Some((panic_message"))

    def test_task_handle_wait_all_uses_the_canonical_combined_job_fence(self) -> None:
        source = TASK_HANDLE.read_text(encoding="utf-8")

        for anchor in (
            "pub fn wait_all(handles: &[Self])",
            "let completions = handles",
            ".map(|handle| handle.completion.clone())",
            "JobHandle::combine(&completions).wait();",
            "Cancellation remains a normal wait completion",
        ):
            self.assertIn(anchor, source)
        self.assertNotIn("for handle in handles {\n            handle.wait();", source)

    def test_combine_regressions_cover_terminal_failure_duplicates_and_races(self) -> None:
        source = COMBINE_TESTS.read_text(encoding="utf-8")

        for test_name in (
            "astra_m24_terminal_outcomes_match_previous_combine",
            "astra_m24_failure_still_waits_for_duplicate_pending_inputs",
            "astra_m24_completion_racing_registration_never_loses_a_dependency",
            "astra_m24_combined_fence_release_evidence",
        ):
            self.assertIn(f"fn {test_name}(", source)
        for anchor in (
            "1_000, 10_000",
            "pending.clone(), pending.clone()",
            "every input must reach the combined fence",
            "TaskState::Failed",
            "TaskState::Cancelled",
            "JobHandle::combine(&handles)",
        ):
            self.assertIn(anchor, source)


if __name__ == "__main__":
    unittest.main()
