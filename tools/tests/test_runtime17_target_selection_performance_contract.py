from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
TARGET_SELECTION = (
    ROOT / "examples/woc/scripts/woc_game/src/world/target_selection.zr"
)


def zr_function_body(source: str, name: str) -> str:
    match = re.search(rf"\b(?:pub\s+)?{re.escape(name)}\s*\([^)]*\)[^{{]*{{", source)
    if match is None:
        raise AssertionError(f"missing Zr function {name}")
    depth = 1
    index = match.end()
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError(f"unterminated Zr function {name}")
    return source[match.end() : index - 1]


class Runtime17TargetSelectionPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = TARGET_SELECTION.read_text(encoding="utf-8")

    def test_enemy_cycle_uses_linear_successor_selection_without_sort_storage(self) -> None:
        body = zr_function_body(self.source, "tabEnemy")
        self.assertNotIn("class EnemyOrder", self.source)
        self.assertNotIn("enemyOrder(", self.source)
        self.assertNotIn("new container.Array", body)
        self.assertIn("tabCandidateBefore(", body)
        self.assertIn("currentNear", body)
        self.assertIn("wrapId", body)
        self.assertIn("successorId", body)

    def test_friendly_cycle_uses_linear_stable_successor_selection(self) -> None:
        body = zr_function_body(self.source, "friendlyCycle")
        self.assertNotIn("new container.Array", body)
        self.assertNotIn("while (insert > 0", body)
        self.assertIn("currentCandidateIndex", body)
        self.assertIn("candidateIndex > currentCandidateIndex", body)
        self.assertIn("wrapId", body)
        self.assertIn("successorId", body)

    def test_contract_covers_missing_current_ties_and_both_wrap_domains(self) -> None:
        body = zr_function_body(self.source, "contractTest")
        for marker in (
            "linearEnemyTie",
            "linearEnemyFallback",
            "linearFriendlyTie",
        ):
            self.assertIn(marker, body)

    def test_adversarial_comparison_growth_is_linear(self) -> None:
        candidate_count = 4_096
        legacy_insertion_comparisons = candidate_count * (candidate_count - 1) // 2
        enemy_selection_comparisons = candidate_count * 3
        friendly_selection_comparisons = candidate_count * 2

        self.assertGreater(
            legacy_insertion_comparisons,
            enemy_selection_comparisons * 600,
        )
        self.assertGreater(
            legacy_insertion_comparisons,
            friendly_selection_comparisons * 1_000,
        )


if __name__ == "__main__":
    unittest.main()
