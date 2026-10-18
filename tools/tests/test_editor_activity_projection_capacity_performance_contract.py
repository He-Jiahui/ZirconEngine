import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
VIEW_SOURCE = REPO_ROOT / "zircon_editor" / "src" / "ui" / "activity" / "view.rs"
DECISION_SOURCE = (
    REPO_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "activity"
    / "decision"
    / "view.rs"
)
CAPACITY_SOURCE = (
    REPO_ROOT / "zircon_editor/src/ui/activity/tests/capacity_tests.rs"
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


class EditorActivityProjectionCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.view = VIEW_SOURCE.read_text(encoding="utf-8")
        cls.decision = DECISION_SOURCE.read_text(encoding="utf-8")
        cls.capacity = CAPACITY_SOURCE.read_text(encoding="utf-8")

    def test_activity_snapshot_projections_reserve_their_exact_input_bound(self) -> None:
        for function_name in (
            "activity_toast_views",
            "activity_progress_views",
            "activity_log_views",
        ):
            body = function_body(self.view, function_name)
            self.assertIn("Vec::with_capacity", body)
            expected = "records.len()" if function_name == "activity_log_views" else "snapshots.len()"
            self.assertIn(expected, body)

    def test_decision_projection_reserves_the_visible_option_count(self) -> None:
        body = function_body(self.decision, "activity_decision_options")
        self.assertIn("Vec::with_capacity", body)
        self.assertIn("decision.options().len()", body)

    def test_capacity_is_applied_before_iterator_materialization(self) -> None:
        for source, function_name in (
            (self.view, "activity_toast_views"),
            (self.view, "activity_progress_views"),
            (self.view, "activity_log_views"),
            (self.decision, "activity_decision_options"),
        ):
            body = function_body(source, function_name)
            self.assertLess(body.index("Vec::with_capacity"), body.index(".map("))
            self.assertIn(".extend(", body)
            self.assertNotIn(".collect()", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        activity_module = (
            REPO_ROOT / "zircon_editor" / "src" / "ui" / "activity" / "mod.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("#[path = \"tests/capacity_tests.rs\"]", activity_module)
        self.assertIn("editor745_activity_projection_capacity_preserves_order", self.capacity)
        self.assertIn("EDITOR745_ACTIVITY_PROJECTION_CAPACITY_BENCH_V1", self.capacity)
        self.assertIn("assert_eq!(optimized_growth_events, 0)", self.capacity)


if __name__ == "__main__":
    unittest.main()
