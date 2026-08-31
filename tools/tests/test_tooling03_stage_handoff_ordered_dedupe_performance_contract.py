from __future__ import annotations

import unittest
from collections.abc import Callable

from tools.zircon_export.stage_handoff import dedupe
from tools.zircon_export.stage_handoff_strategy import _dedupe as dedupe_strategies


class EqualityCountingString(str):
    comparisons = 0

    def __eq__(self, other: object) -> bool:
        type(self).comparisons += 1
        return super().__eq__(other)

    __hash__ = str.__hash__


class Tooling03StageHandoffOrderedDedupePerformanceContractTests(unittest.TestCase):
    def test_handoff_dedupers_use_hash_indexed_membership(self) -> None:
        for dedupe_values in (dedupe, dedupe_strategies):
            with self.subTest(deduper=dedupe_values.__module__):
                self._assert_hash_indexed_dedupe(dedupe_values)

    def _assert_hash_indexed_dedupe(
        self,
        dedupe_values: Callable[[list[str]], list[str]],
    ) -> None:
        unique_count = 128
        first_values = [
            EqualityCountingString(f"strategy-{index:04d}")
            for index in range(unique_count)
        ]
        repeated_values = [EqualityCountingString(value) for value in first_values]
        EqualityCountingString.comparisons = 0

        result = dedupe_values([*first_values, *repeated_values])

        self.assertEqual(result, first_values)
        self.assertLessEqual(
            EqualityCountingString.comparisons,
            unique_count * 2,
            "ordered dedupe membership must stay linear",
        )


if __name__ == "__main__":
    unittest.main()
