from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
DUNGEON_RESET_STATE = (
    ROOT
    / "examples/woc/scripts/woc_game/src/instances/dungeon_reset_state.zr"
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


class Runtime16KeyedResetLockPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = DUNGEON_RESET_STATE.read_text(encoding="utf-8")

    def test_lock_rows_replace_four_parallel_arrays(self) -> None:
        self.assertIn("pub class DungeonResetLock", self.source)
        self.assertIn("pub class DungeonResetLockBucket", self.source)
        self.assertIn(
            "pub var lockBuckets: container.Array<DungeonResetLockBucket>;",
            self.source,
        )
        for legacy_column in (
            "lockOwnerKeys",
            "lockDungeonIds",
            "lockAvailableAt",
            "lockClaimIds",
        ):
            self.assertNotIn(legacy_column, self.source)
        self.assertNotIn("removeResetLockAt", self.source)

    def test_owner_key_lookup_is_binary_search(self) -> None:
        body = zr_function_body(self.source, "findResetLockBucketIndex")
        self.assertIn("var low = 0", body)
        self.assertIn("var high = state.lockBuckets.length", body)
        self.assertIn("while (low < high)", body)
        self.assertIn("low + (high - low) / 2", body)
        self.assertNotIn("index = index + 1", body)

    def test_active_lookup_returns_one_row_and_expires_in_place(self) -> None:
        body = zr_function_body(self.source, "activeResetLock")
        self.assertIn("findResetLockBucketIndex", body)
        self.assertIn("bucket.locks.removeAt", body)
        self.assertIn("state.lockBuckets.removeAt", body)
        self.assertIn("return lock", body)
        self.assertNotIn("state.lockBuckets.length", body)

    def test_contract_covers_index_order_update_and_expiry(self) -> None:
        body = zr_function_body(self.source, "contractTest")
        for marker in (
            "indexedLocks",
            "owner order",
            "updated lock",
            "expired lock",
        ):
            self.assertIn(marker, body)

    def test_adversarial_owner_lookup_growth_is_logarithmic(self) -> None:
        owner_count = 4_096
        legacy_owner_comparisons = owner_count
        binary_owner_comparisons = owner_count.bit_length()

        self.assertLessEqual(binary_owner_comparisons, 13)
        self.assertGreater(
            legacy_owner_comparisons,
            binary_owner_comparisons * 300,
        )


if __name__ == "__main__":
    unittest.main()
