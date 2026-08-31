from __future__ import annotations

import unittest
from pathlib import Path


SCRIPT = (
    Path(__file__).resolve().parents[1]
    / "validate_performance_comparison_receipt.py"
)


class BootstrapStatisticPerformanceContractTests(unittest.TestCase):
    def test_resample_computes_only_the_requested_statistic(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        function = source[source.index("def _resampled_statistic(") : source.index("def _percentile(")]

        self.assertIn("sample_counts = [0] * len(ordered_values)", function)
        self.assertIn("zip(ordered_values, sample_counts, strict=True)", function)
        self.assertIn('if statistic == "median":', function)
        self.assertNotIn("_statistics(resampled)", function)
        self.assertNotIn("deviations", function)

    def test_p95_uses_the_existing_nearest_rank_contract(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        function = source[source.index("def _resampled_statistic(") : source.index("def _percentile(")]

        self.assertIn("ranks = (math.ceil(count * 0.95) - 1,)", function)

    def test_groups_fixed_sample_values_once_before_resampling(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        interval = source[source.index("def _bootstrap_ratio_interval(") : source.index("def _resampled_statistic(")]

        self.assertIn("baseline_groups, baseline_values = _ordered_sample_groups", interval)
        self.assertIn("candidate_groups, candidate_values = _ordered_sample_groups", interval)

    def test_reuses_statistics_verified_against_each_report(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        validation = source[
            source.index("def _validate_samples_against_report(") :
            source.index("def _statistics(")
        ]
        comparison = source[
            source.index("def _validate_comparison(") :
            source.index("def _validate_samples(")
        ]

        self.assertIn(") -> dict[str, float]:", validation)
        self.assertIn("return actual_statistics", validation)
        self.assertIn(
            "baseline_statistics = _validate_samples_against_report(", comparison
        )
        self.assertNotIn("baseline_statistics = _statistics(", comparison)


if __name__ == "__main__":
    unittest.main()
