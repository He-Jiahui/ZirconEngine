from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class EditorRuntimeEventConsumerBoundedPumpContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_pump_contract_exposes_count_time_and_callback_budgets(self) -> None:
        source = self.read(
            "zircon_editor/src/core/runtime_event_consumer/pump.rs"
        )
        self.assertIn("EditorRuntimeEventPumpBudget", source)
        self.assertIn("max_events", source)
        self.assertIn("max_events_per_consumer", source)
        self.assertIn("max_elapsed", source)
        self.assertIn("slow_callback_threshold", source)

    def test_report_exposes_backlog_and_slow_callback_pressure(self) -> None:
        source = self.read(
            "zircon_editor/src/core/runtime_event_consumer/pump.rs"
        )
        for metric in (
            "applied",
            "drained",
            "deferred",
            "dropped",
            "slow_callbacks",
            "queue_depth",
            "pending_sequence_span",
        ):
            self.assertIn(metric, source)

    def test_host_snapshots_active_consumers_before_external_calls(self) -> None:
        source = self.read(
            "zircon_editor/src/core/runtime_event_consumer/host.rs"
        )
        self.assertIn("snapshot_active_consumers", source)
        self.assertIn("pump_with_budget", source)
        self.assertIn("append_drained_deliveries", source)
        self.assertIn("commit_delivery_sequence", source)
        self.assertNotIn("for consumer in active.values_mut()", source)

    def test_regressions_cover_budget_fairness_reentrancy_and_slow_callbacks(self) -> None:
        source = self.read(
            "zircon_editor/src/tests/runtime_event_consumer_bounded_pump.rs"
        )
        for test_name in (
            "bounded_pump_defers_backlog_without_losing_order",
            "round_robin_budget_gives_each_consumer_a_turn",
            "round_robin_start_rotates_under_non_divisible_budgets",
            "gateway_failure_does_not_starve_later_consumers",
            "consumer_callback_can_reenter_host_observation_without_deadlock",
            "concurrent_end_session_is_typed_busy_until_pump_releases_owner",
            "slow_callback_is_visible_in_pump_report",
            "managed_thousand_and_ten_thousand_delivery_budget_report",
            "EDITOR02_RUNTIME_EVENT_PUMP_BENCHMARK",
        ):
            self.assertIn(test_name, source)

    def test_reentrant_lifecycle_mutation_is_typed_busy_and_external_calls_are_lock_free(
        self,
    ) -> None:
        host = self.read("zircon_editor/src/core/runtime_event_consumer/host.rs")
        error = self.read("zircon_editor/src/core/runtime_event_consumer/error.rs")
        regressions = self.read(
            "zircon_editor/src/tests/runtime_event_consumer_bounded_pump.rs"
        )

        self.assertIn("LifecycleMutationBusy", error)
        self.assertIn("LifecycleExecutionGuard", host)
        self.assertIn("EXECUTION_IDLE", host)
        self.assertNotIn("reject_lifecycle_mutation_during_pump", host)
        self.assertIn("remove_active_consumer", host)
        self.assertIn(
            "consumer_callback_reconcile_is_typed_busy_without_deadlock",
            regressions,
        )

    def test_payload_moves_into_callback_and_error_paths_advance_fairness(self) -> None:
        host = self.read("zircon_editor/src/core/runtime_event_consumer/host.rs")
        self.assertIn("delivery.payload", host)
        self.assertNotIn("delivery.payload.clone()", host)
        self.assertIn("first_error", host)
        self.assertIn("advance_round_robin_start", host)
        self.assertNotIn("last_visited", host)


if __name__ == "__main__":
    unittest.main()
