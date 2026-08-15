from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class EditorMessageBackpressureContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_delivery_clone_shares_one_immutable_payload(self) -> None:
        source = self.read(
            "zircon_editor/src/core/editor_message/message/delivery.rs"
        )
        self.assertIn("Arc<EditorMessageDeliveryPayload>", source)
        self.assertIn("Arc::ptr_eq", source)

    def test_bus_uses_typed_inbox_instead_of_unbounded_vec(self) -> None:
        source = self.read("zircon_editor/src/core/editor_message/bus.rs")
        self.assertIn("EditorMessageInbox", source)
        self.assertIn("checked_add", source)
        self.assertIn("SubscriberIdExhausted", source)
        self.assertIn("DeliverySequenceExhausted", source)
        self.assertNotIn("saturating_add(1)", source)
        self.assertNotIn(
            "BTreeMap<EditorSubscriberId, Vec<EditorMessageDelivery>>", source
        )

    def test_inbox_capacity_checks_use_maintained_lane_depths(self) -> None:
        source = self.read("zircon_editor/src/core/editor_message/inbox.rs")

        for lane in ("lossless_depth", "bounded_depth", "latest_depth"):
            self.assertIn(lane, source)
        self.assertIn("fn can_enqueue_lossless", source)
        self.assertIn(
            "self.lossless_depth < self.limits.lossless_capacity", source
        )
        self.assertIn(
            "self.can_enqueue_lossless(delivery.retained_bytes())", source
        )
        self.assertIn("bounded_depth < self.limits.bounded_capacity", source)
        self.assertIn("latest_depth < self.limits.latest_capacity", source)
        self.assertNotIn("fn latest_count(&self)", source)
        self.assertNotIn("fn count(&self, retention", source)
        self.assertIn("retained_bytes_capacity", source)
        self.assertIn("latest_by_key", source)
        self.assertIn("bounded_order", source)

    def test_delivery_computes_one_shared_retained_byte_cost(self) -> None:
        source = self.read(
            "zircon_editor/src/core/editor_message/message/delivery.rs"
        )

        self.assertIn("retained_bytes", source)
        self.assertIn("estimate_json_bytes", source)
        self.assertIn(".dirty()", source)

    def test_retention_policy_separates_lossless_latest_and_bounded(self) -> None:
        source = self.read("zircon_editor/src/core/editor_message/retention.rs")
        for retention in ("Lossless", "Latest", "Bounded"):
            self.assertIn(retention, source)
        self.assertIn("Transaction", source)
        self.assertIn("DirtyChanged", source)
        self.assertIn("Progress", source)

    def test_shared_bus_exposes_inbox_pressure_metrics(self) -> None:
        source = self.read("zircon_editor/src/core/editor_message/shared.rs")
        self.assertIn("inbox_stats", source)
        self.assertIn("EditorMessageInboxStats", source)
        self.assertIn("#[cfg(test)]\n    pub fn deliveries_for", source)

    def test_behavior_regressions_are_wired(self) -> None:
        source = self.read("zircon_editor/src/tests/editor_message/bus/mod.rs")
        self.assertIn("mod backpressure;", source)

        backpressure = self.read(
            "zircon_editor/src/tests/editor_message/bus/backpressure.rs"
        )
        self.assertIn(
            "managed_fanout_allocation_rss_queue_age_and_publish_p95_report",
            backpressure,
        )
        self.assertIn("[1, 5, 100]", backpressure)
        self.assertIn("large_payload_publish_allocated_bytes", backpressure)
        self.assertIn("publish_p95_ns", backpressure)
        self.assertIn("MIXED_LOSSLESS_BACKLOG", backpressure)
        self.assertIn("MAX_PUBLISH_P95_NS", backpressure)
        self.assertIn("rss_before.is_some()", backpressure)
        for case in (
            "mixed_lane_depths_bytes_and_drain_stay_consistent",
            "zero_capacity_and_byte_budget_reject_without_mutation",
            "latest_replacement_evicts_other_latest_state_atomically_under_byte_pressure",
            "dirty_view_bytes_respect_delivery_budget_without_mutating_dirty_state",
            "identifier_exhaustion_is_typed_and_atomic",
        ):
            self.assertIn(case, backpressure)
        self.assertIn("metadata_operation_budget", backpressure)
        self.assertIn("CoalescedAfterDrop", self.read("zircon_editor/src/core/editor_message/inbox.rs"))


if __name__ == "__main__":
    unittest.main()
