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
        host = self.read(
            "zircon_editor/src/core/runtime_event_consumer/host.rs"
        )
        pump = self.read(
            "zircon_editor/src/core/runtime_event_consumer/host/pump_execution.rs"
        )
        pending = self.read(
            "zircon_editor/src/core/runtime_event_consumer/host/pending.rs"
        )
        self.assertIn("mod pump_execution;", host)
        self.assertIn("mod pending;", host)
        self.assertIn("snapshot_active_consumers", host)
        self.assertIn("self.snapshot_active_consumers()", pump)
        self.assertIn("pump_with_budget", pump)
        self.assertIn("append_drained_deliveries", pump)
        self.assertIn("take_pending_batch", pump)
        self.assertIn("restore_pending_batch", pending)
        self.assertIn("PendingDeliveryBatchRestoreGuard", pump)
        source = "\n".join((host, pump, pending))
        self.assertNotIn("commit_delivery_sequence", source)
        self.assertNotIn("for consumer in active.values_mut()", source)

    def test_execution_support_preserves_host_state_ownership(self) -> None:
        host = self.read("zircon_editor/src/core/runtime_event_consumer/host.rs")
        support = self.read(
            "zircon_editor/src/core/runtime_event_consumer/host/execution_support.rs"
        )
        pending = self.read(
            "zircon_editor/src/core/runtime_event_consumer/host/pending.rs"
        )

        self.assertIn("mod execution_support;", host)
        for atomic_owner in ("AtomicU8", "AtomicU64", "Ordering"):
            self.assertIn(atomic_owner, host)
        for state_owner in (
            "active: Mutex<BTreeMap<String, ActiveConsumer>>",
            "pending: VecDeque<PendingDelivery>",
        ):
            self.assertIn(state_owner, host)
        for pending_owner in (
            "pub(super) struct PendingDelivery",
            "delivery: ZrRuntimePluginEventDeliveryV1",
            "fn take_pending_batch(",
            "fn restore_pending_batch(",
        ):
            self.assertIn(pending_owner, pending)
        for support_owner in (
            "pub(super) struct PumpExecutionGuard",
            "pub(super) struct LifecycleExecutionGuard",
            "pub(super) fn validate_delivery(",
            "pub(super) fn unsubscribe_consumer(",
        ):
            self.assertIn(support_owner, support)

    def test_regressions_cover_budget_fairness_reentrancy_and_slow_callbacks(self) -> None:
        root = ROOT / "zircon_editor/src/tests/runtime_event_consumer_bounded_pump"
        source = "\n".join(
            path.read_text(encoding="utf-8")
            for path in sorted(root.glob("*.rs"))
        )
        round_robin = self.read(
            "zircon_editor/src/tests/runtime_event_consumer_bounded_pump/round_robin.rs"
        )
        for test_name in (
            "bounded_pump_defers_backlog_without_losing_order",
            "round_robin_budget_gives_each_consumer_a_turn",
            "round_robin_start_rotates_under_non_divisible_budgets",
            "gateway_failure_does_not_starve_later_consumers",
            "consume_panic_is_typed_and_does_not_starve_other_consumers",
            "consumer_callback_can_reenter_host_observation_without_deadlock",
            "concurrent_end_session_is_typed_busy_until_pump_releases_owner",
            "slow_callback_is_visible_in_pump_report",
            "managed_thousand_and_ten_thousand_delivery_budget_report",
            "PLUGINS01_RUNTIME_EVENT_ABI_PUMP_BENCHMARK",
        ):
            self.assertIn(test_name, source)
        self.assertIn(
            "global_budget_resumes_at_the_first_unvisited_consumer", round_robin
        )
        self.assertIn("budget(0, 1)", round_robin)
        self.assertIn(
            "global_budget_covers_sixty_four_consumers_without_revisiting_the_prefix",
            round_robin,
        )
        self.assertIn("CONSUMER_COUNT: usize = 64", round_robin)

    def test_reentrant_lifecycle_mutation_is_typed_busy_and_external_calls_are_lock_free(
        self,
    ) -> None:
        host = self.read("zircon_editor/src/core/runtime_event_consumer/host.rs")
        error = self.read("zircon_editor/src/core/runtime_event_consumer/error.rs")
        lifecycle = self.read(
            "zircon_editor/src/core/runtime_event_consumer/host/lifecycle.rs"
        )
        regressions = self.read(
            "zircon_editor/src/tests/runtime_event_consumer_bounded_pump/lifecycle.rs"
        )

        self.assertIn("LifecycleMutationBusy", error)
        self.assertIn("LifecycleExecutionGuard", host)
        self.assertIn("EXECUTION_IDLE", host)
        self.assertNotIn("reject_lifecycle_mutation_during_pump", host)
        self.assertIn("mod lifecycle;", host)
        self.assertIn("remove_active_consumer", lifecycle)
        self.assertIn(
            "consumer_callback_reconcile_is_typed_busy_without_deadlock",
            regressions,
        )

    def test_payload_is_borrowed_for_decode_and_error_paths_advance_fairness(self) -> None:
        pump = self.read(
            "zircon_editor/src/core/runtime_event_consumer/host/pump_execution.rs"
        )
        registration = self.read(
            "zircon_editor/src/core/runtime_event_consumer/registration.rs"
        )
        self.assertIn("delivery.delivery().payload.as_ref()", pump)
        self.assertNotIn("delivery.payload.clone()", pump)
        self.assertNotIn("delivery.delivery().payload.clone()", pump)
        self.assertIn("serde_json::from_str::<S::Payload>(payload.get())", registration)
        self.assertIn(".consume(play_session_id, sequence, payload)", registration)
        self.assertIn("first_error", pump)
        self.assertIn("advance_round_robin_start", pump)
        self.assertNotIn("last_visited", pump)


if __name__ == "__main__":
    unittest.main()
