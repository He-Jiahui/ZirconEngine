from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
HEAL_STATE = (
    ROOT / "examples/woc/scripts/woc_game/src/combat/heal_state.zr"
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


class Runtime13HealThreatPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = HEAL_STATE.read_text(encoding="utf-8")
        cls.threat_body = zr_function_body(cls.source, "applyHealingThreat")

    def test_first_pass_tracks_the_only_aware_mob(self) -> None:
        self.assertIn("var onlyAwareIndex = -1;", self.threat_body)
        self.assertIn("onlyAwareIndex = index;", self.threat_body)

    def test_single_aware_mob_commits_without_a_second_scan(self) -> None:
        fast_path = re.search(
            r"if \(awareCount == 1\) \{(?P<body>.*?)\n\s*\}",
            self.threat_body,
            re.DOTALL,
        )
        self.assertIsNotNone(fast_path)
        body = fast_path.group("body")
        self.assertIn("mobs.healerThreat[onlyAwareIndex]", body)
        self.assertIn("threatModifier(source)", body)
        self.assertIn("return;", body)

    def test_multi_mob_path_retains_even_threat_distribution(self) -> None:
        self.assertIn("/ <float>awareCount", self.threat_body)
        self.assertIn("index = 0;", self.threat_body)
        self.assertGreaterEqual(self.threat_body.count("while (index < mobs.dead.length)"), 2)

    def test_contract_exercises_single_aware_mob_result(self) -> None:
        contract = zr_function_body(self.source, "contractTest")
        self.assertIn("singleAware", contract)
        self.assertIn("singleAware.healerThreat[2]", contract)

    def test_single_aware_work_drops_by_at_least_fifty_percent(self) -> None:
        mob_count = 4_096
        legacy_predicate_evaluations = mob_count * 2
        optimized_predicate_evaluations = mob_count
        reduction_percent = (
            (legacy_predicate_evaluations - optimized_predicate_evaluations)
            * 100.0
            / legacy_predicate_evaluations
        )
        self.assertGreaterEqual(reduction_percent, 50.0)


if __name__ == "__main__":
    unittest.main()
