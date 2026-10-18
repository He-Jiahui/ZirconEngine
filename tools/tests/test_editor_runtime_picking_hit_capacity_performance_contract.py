from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
ADAPTER = ROOT / "zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs"
LOWER_TESTS = (
    ROOT / "zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter/tests/hit_capacity_tests.rs"
)


class EditorRuntimePickingHitCapacityPerformanceContractTests(unittest.TestCase):
    def test_runtime_hit_projection_reserves_the_known_candidate_upper_bound(self) -> None:
        source = ADAPTER.read_text(encoding="utf-8")
        implementation = source.split("#[cfg(test)]", 1)[0]

        self.assertIn(
            "let hit_capacity = runtime_pointer_hit_capacity(",
            implementation,
        )
        self.assertIn("Vec::with_capacity(hit_capacity)", implementation)
        self.assertIn("runtime_pointer_hit_capacity(stacked.len(), renderer_candidates.len())", implementation)
        self.assertIn("hits.extend(renderer_candidates.iter().filter_map", implementation)
        self.assertIn("if hits.is_empty()", implementation)

    def test_capacity_helper_keeps_empty_fast_path_and_overflow_safety(self) -> None:
        source = ADAPTER.read_text(encoding="utf-8")
        implementation = source.split("#[cfg(test)]", 1)[0]

        self.assertIn("fn runtime_pointer_hit_capacity(", implementation)
        self.assertIn("stacked_count.saturating_add(renderer_count)", implementation)
        self.assertIn("candidate_count == 0", implementation)
        self.assertIn(".saturating_add", implementation)

    def test_lower_regression_covers_capacity_and_benchmark_marker(self) -> None:
        tests = LOWER_TESTS.read_text(encoding="utf-8")

        self.assertIn(
            "optimization_batch_20260915_editor765_runtime_pointer_hit_capacity",
            tests,
        )
        self.assertIn(
            "optimization_batch_20260915_editor765_runtime_pointer_hit_capacity_p95",
            tests,
        )
        self.assertIn("EDITOR765_RUNTIME_POINTER_HIT_CAPACITY_BENCH_V1", tests)
        self.assertIn("legacy_hit_capacity", tests)
        self.assertIn("optimized_hit_capacity", tests)


if __name__ == "__main__":
    unittest.main()
