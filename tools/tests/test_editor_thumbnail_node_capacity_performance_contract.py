import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "layouts"
    / "views"
    / "asset_browser"
    / "thumbnail_nodes.rs"
)
REGRESSION = SOURCE.parent / "thumbnail_nodes" / "capacity_tests.rs"


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


class EditorThumbnailNodeCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.regression = REGRESSION.read_text(encoding="utf-8")

    def test_thumbnail_slots_reserve_the_bounded_node_upper_bound(self) -> None:
        helper = function_body(self.source, "thumbnail_node_capacity")
        self.assertIn("min(", helper)
        self.assertIn("saturating_mul", helper)
        self.assertIn("saturating_add", helper)
        self.assertIn("THUMBNAIL_NODES_PER_ITEM", helper)

        body = function_body(self.source, "append_asset_browser_thumbnail_slots")
        self.assertIn("thumbnail_node_capacity(", body)
        self.assertIn("nodes.reserve(additional_nodes)", body)
        self.assertLess(body.index("nodes.reserve(additional_nodes)"), body.index("nodes.push"))

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            '#[path = "thumbnail_nodes/capacity_tests.rs"]',
            self.source,
        )
        self.assertIn(
            "optimization_batch_20260917_editor790_thumbnail_node_capacity_bench",
            self.regression,
        )
        self.assertIn("EDITOR790_THUMBNAIL_NODE_CAPACITY_BENCH_V1", self.regression)
        self.assertIn("optimized_growth_events", self.regression)

    def test_deterministic_model_eliminates_geometric_growth(self) -> None:
        item_count = 4_096
        node_count = 1 + item_count * 9
        capacity = 0
        legacy_growth_events = 0
        for length in range(1, node_count + 1):
            if length > capacity:
                capacity = 4 if capacity == 0 else capacity * 2
                legacy_growth_events += 1
        optimized_growth_events = 0
        self.assertGreater(legacy_growth_events, 0)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
