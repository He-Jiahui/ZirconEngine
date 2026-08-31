from __future__ import annotations

import unittest

from tools.zircon_export.native_build_command import (
    normalized_native_dynamic_build_features,
)
from tools.zircon_export.native_build_workspace import dedupe as dedupe_workspace_crates
from tools.zircon_export.plugin_build_command import plugin_build_features


class EqualityCountingString(str):
    comparisons = 0

    def __eq__(self, other: object) -> bool:
        type(self).comparisons += 1
        return super().__eq__(other)

    __hash__ = str.__hash__


class Tooling03BuildFeatureOrderedDedupePerformanceContractTests(unittest.TestCase):
    def test_build_feature_and_crate_dedupers_use_hash_indexed_membership(
        self,
    ) -> None:
        unique_count = 128
        first_values = [
            EqualityCountingString(f"feature-{index:04d}")
            for index in range(unique_count)
        ]
        repeated_values = [EqualityCountingString(value) for value in first_values]
        values = [*first_values, *repeated_values]

        cases = (
            ("native build features", self._native_build_features, first_values),
            ("workspace crates", dedupe_workspace_crates, first_values),
            (
                "plugin build features",
                self._plugin_build_features,
                ["dist", *first_values],
            ),
        )
        for label, dedupe_values, expected in cases:
            with self.subTest(deduper=label):
                EqualityCountingString.comparisons = 0

                result = dedupe_values(values)

                self.assertEqual(result, expected)
                self.assertLessEqual(
                    EqualityCountingString.comparisons,
                    unique_count * 8,
                    "ordered dedupe membership must stay linear",
                )

    @staticmethod
    def _native_build_features(values: list[str]) -> list[str]:
        diagnostics: list[str] = []
        result = normalized_native_dynamic_build_features(values, diagnostics)
        if diagnostics:
            raise AssertionError(diagnostics)
        return result

    @staticmethod
    def _plugin_build_features(values: list[str]) -> list[str]:
        diagnostics: list[str] = []
        result = plugin_build_features(values, diagnostics)
        if diagnostics:
            raise AssertionError(diagnostics)
        return result


if __name__ == "__main__":
    unittest.main()
