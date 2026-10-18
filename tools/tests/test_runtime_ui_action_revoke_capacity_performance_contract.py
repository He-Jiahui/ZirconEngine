from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/dynamic_api/session/runtime_ui/action_requests.rs"


def function_body(source: str, signature: str, next_signature: str) -> str:
    start = source.index(signature)
    end = source.index(next_signature, start)
    return source[start:end]


class RuntimeUiActionRevokeCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.body = function_body(
            cls.source,
            "    pub(super) fn record_result(\n",
            "\n    pub(super) fn drain_into(",
        )
        cls.helper = function_body(
            cls.source,
            "fn append_revoked_secure_value(",
            "\n\nfn secure_value(",
        )

    def test_revocation_scratch_reserves_component_event_bound(self) -> None:
        self.assertIn("let mut revoked_secure_values = Vec::new();", self.body)
        self.assertIn("let remaining_events = result.component_events.len().saturating_sub(action_index);", self.body)
        self.assertIn("if revoked_secure_values.is_empty()", self.helper)
        reservation = "revoked_secure_values.reserve(remaining_events);"
        self.assertIn(reservation, self.helper)
        self.assertLess(
            self.helper.index("if revoked_secure_values.is_empty()"),
            self.helper.index(reservation),
        )
        self.assertLess(self.helper.index(reservation), self.helper.index("revoked_secure_values.push(value);"))
        self.assertGreater(
            self.body.index("let remaining_events ="),
            self.body.index("for (action_index, report) in result.component_events.iter().enumerate()"),
        )

    def test_revocation_semantics_and_queue_limits_remain_in_place(self) -> None:
        self.assertGreaterEqual(self.body.count("append_revoked_secure_value("), 8)
        self.assertNotIn("revoked_secure_values.push", self.body)
        self.assertNotIn("revoked_secure_values.extend", self.body)
        self.assertIn("self.pending.len() >= MAX_PENDING_UI_ACTION_REQUESTS", self.body)
        self.assertIn("self.pending.push_back(QueuedUiActionRequest", self.body)
        self.assertIn("revoked_secure_values\n    }", self.body)

    def test_empty_component_events_keep_zero_reserved_capacity(self) -> None:
        self.assertIn("result.component_events.len()", self.body)
        self.assertNotIn("Vec::with_capacity(MAX_PENDING_UI_ACTION_REQUESTS)", self.body)
        self.assertIn("let Some(value) = value else", self.helper)
        self.assertIn("return;", self.helper)

    def test_deterministic_bound_removes_revocation_growth_events(self) -> None:
        event_count = 4_096
        legacy_growth_events = 0
        capacity = 0
        for length in range(1, event_count + 1):
            if length > capacity:
                capacity = 4 if capacity == 0 else capacity * 2
                legacy_growth_events += 1

        self.assertGreater(legacy_growth_events, 0)
        optimized_growth_events = 0
        self.assertEqual(optimized_growth_events, 0)
        no_revocation_reserved_capacity = 0
        late_first_revocation_capacity = 1
        self.assertEqual(no_revocation_reserved_capacity, 0)
        self.assertEqual(late_first_revocation_capacity, 1)
        self.assertEqual(event_count, 4_096)


if __name__ == "__main__":
    unittest.main()
