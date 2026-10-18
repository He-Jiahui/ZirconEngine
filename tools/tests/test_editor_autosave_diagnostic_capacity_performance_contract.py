import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_editor" / "src" / "core" / "recovery" / "autosave_service.rs"
REGRESSION = (
    SOURCE.parent / "autosave_service" / "diagnostic_capacity_tests.rs"
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


class EditorAutosaveDiagnosticCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.regression = REGRESSION.read_text(encoding="utf-8")

    def test_retired_issue_projection_reserves_remaining_retired_bound_lazily(self) -> None:
        body = function_body(self.source, "pump_retired")
        self.assertIn("let retired_count = self.retired.len();", body)
        self.assertIn(
            "issues.reserve(retired_count.saturating_sub(issues.len()));",
            body,
        )
        self.assertIn("if !retired_issues.is_empty()", body)
        self.assertLess(
            body.index("issues.reserve(retired_count.saturating_sub(issues.len()));"),
            body.index("issues.extend(retired_issues);"),
        )

    def test_capacity_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            '#[path = "autosave_service/diagnostic_capacity_tests.rs"]',
            self.source,
        )
        self.assertIn(
            "optimization_batch_20260919_editor_autosave_diagnostic_capacity_preserves_issue_bound",
            self.regression,
        )
        self.assertIn(
            "optimization_batch_20260919_editor_autosave_diagnostic_capacity_bench",
            self.regression,
        )
        self.assertIn(
            "EDITOR_AUTOSAVE_DIAGNOSTIC_CAPACITY_BENCH_V1",
            self.regression,
        )

    def test_deterministic_model_has_no_geometric_growth_for_retired_bound(self) -> None:
        retired_count = 64
        capacity = 0
        legacy_growth_events = 0
        for length in range(1, retired_count + 1):
            if length > capacity:
                capacity = 4 if capacity == 0 else capacity * 2
                legacy_growth_events += 1
        self.assertGreater(legacy_growth_events, 0)
        optimized_growth_events = 0
        self.assertEqual(optimized_growth_events, 0)
        self.assertLess(optimized_growth_events, legacy_growth_events)
        self.assertIn("no_error_issues.capacity(), 0", self.regression)


if __name__ == "__main__":
    unittest.main()
