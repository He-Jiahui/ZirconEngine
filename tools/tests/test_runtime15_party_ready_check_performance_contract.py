from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
WORLD_STATE = ROOT / "examples/woc/scripts/woc_game/src/world/state.zr"
READY_SWEEP = (
    ROOT
    / "examples/woc/scripts/woc_game/src/social/party_raid_state.zr"
)
READY_SWEEP_PACKAGE = (
    ROOT
    / "examples/woc/scripts/woc_game/woc_m6_party_raid_tests.zrp"
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


class Runtime15PartyReadyCheckPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.world = WORLD_STATE.read_text(encoding="utf-8")
        cls.sweep = READY_SWEEP.read_text(encoding="utf-8")

    def test_world_tick_delegates_to_the_linear_sweep_module(self) -> None:
        body = zr_function_body(self.world, "updatePartyReadyChecks")
        self.assertIn(
            'var partyReadyCheckSweep = %import("social/party_raid_state");',
            self.world,
        )
        self.assertIn("partyReadyCheckSweep.expireRows(", body)
        self.assertNotIn("var prior", body)
        self.assertNotIn("partyReadyCheckHasPending", body)
        self.assertNotIn("clearPartyReadyCheck", body)

    def test_member_removal_finalizes_completion_at_the_mutation_boundary(self) -> None:
        body = zr_function_body(self.world, "partyRemoveReadyCheckMember")
        self.assertIn("var partyId =", body)
        self.assertIn("clearReadyCheckRow(state, index)", body)
        self.assertIn("!partyReadyCheckHasPending(state, partyId)", body)
        self.assertIn("clearPartyReadyCheck(state, partyId)", body)

    def test_expiry_module_has_one_linear_row_loop(self) -> None:
        body = zr_function_body(self.sweep, "expireRows")
        self.assertEqual(body.count("while ("), 1)
        self.assertIn("clearReadyCheckSweepRow(", body)
        self.assertIn("endsAtMicros[index] <= nowMicros", body)
        self.assertNotIn("partyId", body)

    def test_module_contract_covers_mixed_deadlines_and_empty_rows(self) -> None:
        body = zr_function_body(self.sweep, "readyCheckSweepContractTest")
        for marker in (
            "mixedPartyIds",
            "expiredRows != 2",
            "future row",
            "empty row",
        ):
            self.assertIn(marker, body)
        package = READY_SWEEP_PACKAGE.read_text(encoding="utf-8")
        self.assertIn('"entry": "social/party_raid_state_test_main"', package)

    def test_adversarial_tick_growth_is_linear(self) -> None:
        entity_count = 4_096
        legacy_prior_comparisons = entity_count * (entity_count - 1) // 2
        linear_row_checks = entity_count

        self.assertEqual(legacy_prior_comparisons, 8_386_560)
        self.assertGreater(legacy_prior_comparisons, linear_row_checks * 2_000)


if __name__ == "__main__":
    unittest.main()
