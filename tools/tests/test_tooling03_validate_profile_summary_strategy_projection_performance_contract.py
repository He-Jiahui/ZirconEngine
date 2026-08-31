"""Performance contract for validate profile-summary strategies."""

from __future__ import annotations

import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report_validate_profile_summary_schema import (
    validate_profile_summary_strategy_projection,
)


REPO_ROOT = Path(__file__).resolve().parents[2]
OWNER = REPO_ROOT / "tools/zircon_export/pipeline_report_validate_profile_summary_schema.py"


class ValidateProfileSummaryStrategyProjectionPerformanceContractTests(unittest.TestCase):
    def test_projection_preserves_ordered_groups(self) -> None:
        result = validate_profile_summary_strategy_projection(
            ["", " bad ", "unknown", "library_embed", "library_embed"]
        )
        self.assertEqual(result[0], [])
        self.assertEqual(result[1], [])
        self.assertEqual(
            result[2],
            [
                "validate report profile_summary.strategies[0] must be a non-empty trimmed export strategy",
                "validate report profile_summary.strategies[1] must be a non-empty trimmed export strategy",
                "unsupported export strategy unknown",
            ],
        )
        self.assertEqual(
            result[3],
            ["validate report profile_summary.strategies[4] duplicates entry 3"],
        )

    def test_non_string_entry_suppresses_strategy_groups(self) -> None:
        result = validate_profile_summary_strategy_projection(["library_embed", 7])
        self.assertEqual(
            result,
            (["validate report profile_summary.strategies[1] must be a string"], [], [], []),
        )

    def test_projection_has_one_strategy_loop_for_normalization(self) -> None:
        source = OWNER.read_text(encoding="utf-8")
        helper = source[source.index("def validate_profile_summary_strategy_projection("):source.index("def validate_profile_summary_schema_diagnostics(")]
        self.assertEqual(helper.count("normalize_export_strategy(strategy)"), 1)


if __name__ == "__main__":
    unittest.main()
