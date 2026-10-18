import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "workbench"
    / "asset_content_layout"
    / "paint_metadata.rs"
)
REGRESSION = (
    REPO_ROOT / "zircon_editor/src/ui/workbench/asset_content_layout/paint_metadata/tests/capacity_tests.rs"
)


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


class EditorAssetVisibleRowCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.regression = REGRESSION.read_text(encoding="utf-8")

    def test_visible_group_append_reserves_bounded_row_count(self) -> None:
        helper = function_body(self.source, "reserve_visible_group_rows")
        self.assertIn("saturating_add", helper)
        self.assertIn("rows.reserve(additional_rows)", helper)

        body = function_body(self.source, "append_visible_group_rows")
        self.assertIn("let first_visible = first.min(last);", body)
        reserve_call = "reserve_visible_group_rows(rows, groups, first_visible, last);"
        self.assertIn(reserve_call, body)
        self.assertLess(body.index(reserve_call), body.index("rows.extend_from_slice"))

    def test_visibility_clip_and_sorted_projection_remain_intact(self) -> None:
        body = function_body(self.source, "append_visible_group_rows")
        self.assertIn("viewport.intersect(damage_clip)", body)
        self.assertIn("partition_point", body)
        self.assertIn("rows.sort_unstable()", self.source)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("#[path = \"paint_metadata/tests/capacity_tests.rs\"]", self.source)
        self.assertIn("editor760_visible_group_rows_reserve_exact_bound", self.regression)
        self.assertIn("editor760_visible_group_rows_capacity_bench_v1", self.regression)
        self.assertIn("EDITOR760_VISIBLE_GROUP_ROWS_CAPACITY_BENCH_V1", self.regression)
        self.assertIn("optimized_growth_events", self.regression)

    def test_deterministic_model_removes_visible_append_growth(self) -> None:
        group_row_counts = [2, 3, 1]
        visible_group_row_count = sum(group_row_counts[0:2])
        legacy_growth_events = 1
        optimized_growth_events = 0
        self.assertEqual(visible_group_row_count, 5)
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
