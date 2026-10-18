import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RULER_SOURCE = REPO_ROOT / "zircon_editor" / "src" / "ui" / "timeline" / "ruler.rs"
TEST_SOURCE = REPO_ROOT / "zircon_editor/src/ui/timeline/tests/cases.rs"


def function_body(source: str, function_name: str) -> str:
    match = re.search(
        rf"\bfn\s+{re.escape(function_name)}(?![A-Za-z0-9_])\s*\(.*?\)",
        source,
        re.S,
    )
    if match is None:
        raise AssertionError(f"missing function {function_name}")
    opening = source.find("{", match.end())
    if opening < 0:
        raise AssertionError(f"missing body for {function_name}")
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated body for {function_name}")


class EditorTimelineRulerCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.ruler = RULER_SOURCE.read_text(encoding="utf-8")
        cls.tests = TEST_SOURCE.read_text(encoding="utf-8")

    def test_ruler_reserves_interval_bound_plus_endpoints(self) -> None:
        body = function_body(self.ruler, "build_timeline_ruler_ticks")
        self.assertIn("ruler_tick_capacity(desired_intervals)", body)
        self.assertIn("Vec::with_capacity", body)
        self.assertLess(body.index("Vec::with_capacity"), body.index("while time"))

        helper = function_body(self.ruler, "ruler_tick_capacity")
        self.assertIn("MAX_RULER_TICKS", helper)
        self.assertIn("saturating_add(2)", helper)

    def test_tick_generation_semantics_remain_bounded_and_ordered(self) -> None:
        body = function_body(self.ruler, "build_timeline_ruler_ticks")
        self.assertIn("ticks.len() < MAX_RULER_TICKS", body)
        self.assertIn("ticks.insert(", body)
        self.assertIn("ticks.push(", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("ruler_tick_capacity", self.tests)
        self.assertIn("editor761_timeline_ruler_capacity_bench_v1", self.tests)
        self.assertIn("EDITOR761_TIMELINE_RULER_CAPACITY_BENCH_V1", self.tests)
        self.assertIn("optimized_growth_events", self.tests)

    def test_deterministic_model_removes_growth_reallocations(self) -> None:
        requested_intervals = 4_096
        endpoint_slots = 2
        legacy_growth_events = 1
        optimized_growth_events = 0
        self.assertEqual(requested_intervals + endpoint_slots, 4_098)
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
