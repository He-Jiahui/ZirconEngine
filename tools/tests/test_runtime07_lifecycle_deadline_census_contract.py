import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
TASK_GRAPH = (
    ROOT / "zircon_runtime/src/core/runtime/tasks/task_graph/engine_task_graph.rs"
)
TASK_GRAPH_TESTS = (
    ROOT / "zircon_runtime/src/core/runtime/tasks/task_graph/engine_task_graph/tests/cases.rs"
)
SESSION_STORE = ROOT / "zircon_runtime/src/dynamic_api/session/registry/session_store.rs"
SESSION_OWNER = (
    ROOT
    / "zircon_runtime/src/dynamic_api/session/registry/session_owner/runtime.rs"
)


class Runtime07LifecycleDeadlineCensusContractTests(unittest.TestCase):
    def test_task_graph_retains_one_closing_scope_set_and_final_census(self) -> None:
        source = TASK_GRAPH.read_text(encoding="utf-8")

        for anchor in (
            "shutdown_scopes: Vec<Arc<TaskGraphScopeInner>>",
            "stopped_scope_census: Vec<TaskGraphScopeCensus>",
            "EngineTaskGraphLifecycle::Closing => return state.shutdown_scopes.clone()",
            "state.shutdown_scopes = state.scopes.values().filter_map(Weak::upgrade).collect();",
            "state.stopped_scope_census = stopped_scope_census;",
            "std::mem::take(&mut state.shutdown_scopes)",
            "state.stopped_scope_census.clone()",
        ):
            self.assertIn(anchor, source)

        begin = source.index("fn begin_shutdown(")
        stop = source.index("fn mark_stopped(")
        report = source.index("fn shutdown_report(")
        self.assertLess(begin, stop)
        self.assertLess(stop, report)

    def test_task_graph_regression_uses_one_absolute_budget_and_retry(self) -> None:
        tests = TASK_GRAPH_TESTS.read_text(encoding="utf-8")
        start = tests.index("fn shutdown_until_does_not_restart_budget_for_later_scope_drains()")
        body = tests[start:]

        for anchor in (
            "let timeout = runtime.shutdown_until(shutdown_started_at + SHARED_SHUTDOWN_BUDGET);",
            "the second scope must not receive a fresh shutdown duration",
            "shutdown_elapsed < MAX_SHARED_SHUTDOWN_ELAPSED",
            "let retry = runtime\n        .shutdown(Duration::from_secs(2))",
            "released scopes should remain retryable after the shared deadline expires",
            "all(TaskGraphWorkerShutdownCensus::all_joined)",
        ):
            self.assertIn(anchor, body)

    def test_session_destroy_reuses_entry_deadline_for_every_teardown_stage(self) -> None:
        source = SESSION_STORE.read_text(encoding="utf-8")
        start = source.index("pub(super) fn destroy_session_slot_with_timeout(")
        body = source[start:]

        for anchor in (
            "let destroy_started_at = Instant::now();",
            "destroy_started_at.checked_add(timeout)",
            "slot.wait_for_actions(destroy_deadline.saturating_duration_since(Instant::now()))",
            "wait_for_wake_callbacks(destroy_deadline.saturating_duration_since(Instant::now()))",
            "slot.shutdown_until(destroy_deadline)",
            "slot.preserve_failed_teardown_for_retry();",
        ):
            self.assertIn(anchor, body)

        self.assertLess(
            body.index("slot.wait_for_actions("),
            body.index("slot.shutdown_until(destroy_deadline)"),
        )
        self.assertLess(
            body.index("slot.shutdown_until(destroy_deadline)"),
            body.index("registry.sessions.remove"),
        )

    def test_owner_retry_waits_for_the_original_operation_without_reinvocation(self) -> None:
        source = SESSION_OWNER.read_text(encoding="utf-8")
        tests = source[source.index("#[cfg(test)]"):]

        shutdown = source[source.index("pub(in crate::dynamic_api::session) fn shutdown_until("):]
        for anchor in (
            "self.wake.disable_new_entries();",
            "wait_for_callbacks(deadline.saturating_duration_since(Instant::now()))",
            "self.owner\n            .shutdown_until(deadline",
            "session.shutdown_before_library_unload_until(deadline)",
        ):
            self.assertIn(anchor, shutdown)

        for anchor in (
            "fn astra_life_a4_runtime_owner_bounds_blocking_event_callback_without_reinvocation()",
            "OwnerShutdownReceipt::Pending",
            "assert_eq!(invocations.load(Ordering::SeqCst), 1);",
            "OwnerShutdownReceipt::Joined",
            "assert!(weak_code_owner.upgrade().is_none());",
        ):
            self.assertIn(anchor, tests)


if __name__ == "__main__":
    unittest.main()
