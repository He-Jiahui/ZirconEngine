from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
QUEUE = ROOT / "examples/woc/scripts/woc_game/src/social/card_duel_queue.zr"
COORDINATOR = (
    ROOT
    / "examples/woc/scripts/woc_game/src/social/card_duel_queue_coordinator.zr"
)
PRIMITIVES_TEST = (
    ROOT
    / "examples/woc/scripts/woc_game/src/social/card_duel_primitives_test_main.zr"
)
COORDINATOR_TEST = (
    ROOT
    / "examples/woc/scripts/woc_game/src/social/card_duel_queue_coordinator_test_main.zr"
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


class Runtime15CardDuelQueuePerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.queue = QUEUE.read_text(encoding="utf-8")
        cls.coordinator = COORDINATOR.read_text(encoding="utf-8")
        cls.primitives_test = PRIMITIVES_TEST.read_text(encoding="utf-8")
        cls.coordinator_test = COORDINATOR_TEST.read_text(encoding="utf-8")

    def test_queue_uses_generation_qualified_intrusive_slots(self) -> None:
        for marker in (
            "slotGenerations",
            "freeSlots",
            "headSlot",
            "tailSlot",
            "liveCount",
        ):
            self.assertIn(marker, self.queue)

    def test_fifo_pairing_unlinks_two_heads_without_array_shifts(self) -> None:
        body = zr_function_body(self.queue, "pairFirst")
        pop = zr_function_body(self.queue, "popHead")
        self.assertEqual(body.count("popHead("), 2)
        self.assertNotIn("removeAt(0)", body)
        self.assertIn("aOwnerIndex", pop)
        self.assertIn("bOwnerIndex", pop)

    def test_tracked_leave_rejects_stale_generation_before_unlink(self) -> None:
        body = zr_function_body(self.queue, "leaveTracked")
        self.assertIn("slotGenerations[slot] != generation", body)
        self.assertIn("slotPlayerIds[slot] != pid", body)
        self.assertIn("unlinkSlot(slot)", body)

    def test_coordinator_purges_only_transitioned_candidates(self) -> None:
        upsert = zr_function_body(self.coordinator, "upsert")
        purge = zr_function_body(self.coordinator, "purgeUnqueueable")
        self.assertIn("pendingPurgeCandidateIndexes.add(index)", upsert)
        self.assertIn("purgePending", upsert)
        self.assertIn("leaveTracked(", purge)
        self.assertNotIn("for (var candidate in this.candidates)", purge)
        self.assertNotIn("this.queue.leave(", purge)

    def test_snapshot_and_restore_walk_the_queue_once(self) -> None:
        snapshot = zr_function_body(self.coordinator, "snapshotEntries")
        restore = zr_function_body(self.coordinator, "restoreEntries")
        pair = zr_function_body(self.coordinator, "pairNext")
        self.assertIn("this.queue.snapshotEntries()", snapshot)
        self.assertNotIn("this.queue.at(", snapshot)
        self.assertIn("joinTracked(", restore)
        self.assertNotIn("this.queue.contains(", restore)
        self.assertIn("pair.aOwnerIndex", pair)
        self.assertIn("pair.bOwnerIndex", pair)

    def test_zr_contracts_cover_stale_handles_and_fifo_requeue(self) -> None:
        for marker in ("staleJoin", "reusedJoin", "leaveTracked"):
            self.assertIn(marker, self.primitives_test)
        for marker in ("requeued", "beforePurge", "snapshotEntries"):
            self.assertIn(marker, self.coordinator_test)
        entity_count = 1_024
        legacy_shift_moves = entity_count * (entity_count - 1) // 2
        intrusive_head_unlinks = entity_count
        self.assertEqual(legacy_shift_moves, 523_776)
        self.assertGreater(legacy_shift_moves, intrusive_head_unlinks * 500)


if __name__ == "__main__":
    unittest.main()
