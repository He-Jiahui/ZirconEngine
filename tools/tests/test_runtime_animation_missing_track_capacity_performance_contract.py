from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
COMPILED = ROOT / "zircon_runtime/src/animation/sequence/compiled.rs"
LOWER_TEST = ROOT / "zircon_runtime/src/animation/sequence/tests/compiled_missing_track_capacity_tests.rs"


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated Rust function: {signature}")


class RuntimeAnimationMissingTrackCapacityPerformanceContractTests(unittest.TestCase):
    def test_missing_track_diagnostics_reserve_only_after_first_miss(self) -> None:
        source = COMPILED.read_text(encoding="utf-8")
        body = function_body(source, "pub fn compile_sequence_for_world(")

        self.assertIn("let mut missing_tracks = Vec::new();", body)
        self.assertGreaterEqual(body.count("missing_tracks.reserve(source_track_capacity)"), 2)
        self.assertNotIn("let mut missing_tracks = Vec::with_capacity", body)

    def test_lower_regression_and_deterministic_capacity_model_are_wired(self) -> None:
        source = COMPILED.read_text(encoding="utf-8")
        self.assertIn(
            "#[path = \"tests/compiled_missing_track_capacity_tests.rs\"]",
            source,
        )
        self.assertIn("mod compiled_missing_track_capacity_tests;", source)

        lower = LOWER_TEST.read_text(encoding="utf-8")
        self.assertIn(
            "optimization_batch_20260919_runtime839_animation_missing_track_capacity_preserves_lazy_success_path",
            lower,
        )
        self.assertIn("RUNTIME839_ANIMATION_MISSING_TRACK_CAPACITY_BENCH_V1", lower)
        self.assertEqual(growth_events(4096), 11)
        self.assertEqual(growth_events(4096, 4096), 0)


def growth_events(item_count: int, initial_capacity: int = 0) -> int:
    capacity = initial_capacity
    events = 0
    for length in range(1, item_count + 1):
        if length > capacity:
            capacity = 4 if capacity == 0 else capacity * 2
            events += 1
    return events


if __name__ == "__main__":
    unittest.main()
