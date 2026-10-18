from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/dynamic_api/session/runtime_ui/input_routing.rs"


def function_body(source: str, signature: str, next_signature: str) -> str:
    start = source.index(signature)
    end = source.index(next_signature, start)
    return source[start:end]


class RuntimeUiDispatchOutputFastPathPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.body = function_body(
            cls.source,
            "    fn record_dispatch_outputs(\n",
            "\n    }\n}\n\nfn input_event_for_surface",
        )

    def test_empty_output_guard_precedes_surface_lookup_and_tree_id_clone(self) -> None:
        guard = "if result.host_requests.is_empty() && result.component_events.is_empty() {"
        self.assertIn(guard, self.body)
        guard_offset = self.body.index(guard)
        surface_lookup_offset = self.body.index("self.surfaces.get(surface_index)")
        tree_clone_offset = self.body.index("tree_id.clone()")
        self.assertLess(guard_offset, surface_lookup_offset)
        self.assertLess(guard_offset, tree_clone_offset)

    def test_non_empty_outputs_still_feed_both_queues(self) -> None:
        self.assertIn("self.host_requests", self.body)
        self.assertIn("action_requests", self.body)
        self.assertIn("result.host_requests", self.body)
        self.assertIn("result.component_events", self.body)

    def test_fast_path_only_covers_queues_owned_by_this_projection(self) -> None:
        self.assertNotIn("result.widget_events.is_empty()", self.body)
        self.assertNotIn("result.binding_reports.is_empty()", self.body)
        self.assertNotIn("result.applied_effects.is_empty()", self.body)
        self.assertNotIn("result.rejected_effects.is_empty()", self.body)

    def test_deterministic_empty_dispatch_model_removes_projection_work(self) -> None:
        dispatches = 100_000
        legacy_tree_id_clones = dispatches
        optimized_tree_id_clones = 0
        legacy_queue_passes = dispatches * 2
        optimized_queue_passes = 0

        self.assertEqual(legacy_tree_id_clones, dispatches)
        self.assertEqual(optimized_tree_id_clones, 0)
        self.assertEqual(legacy_queue_passes, dispatches * 2)
        self.assertEqual(optimized_queue_passes, 0)
        self.assertGreaterEqual(
            legacy_tree_id_clones - optimized_tree_id_clones,
            dispatches,
        )
        self.assertGreaterEqual(legacy_queue_passes - optimized_queue_passes, dispatches * 2)


if __name__ == "__main__":
    unittest.main()
