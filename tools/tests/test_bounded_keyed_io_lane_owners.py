"""Guard the bounded I/O lane's state and queue ownership boundaries."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
LANE_ROOT = REPO_ROOT / "zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/lane.rs"
LANE_STATE = REPO_ROOT / "zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/lane/state.rs"
LANE_QUEUE = REPO_ROOT / "zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/lane/queue.rs"


class BoundedKeyedIoLaneOwnerTests(unittest.TestCase):
    def test_lane_separates_public_orchestration_state_and_queue_mechanics(self) -> None:
        lane_source = LANE_ROOT.read_text(encoding="utf-8")

        self.assertLess(len(lane_source.splitlines()), 800)
        self.assertIn("mod queue;", lane_source)
        self.assertIn("mod state;", lane_source)
        self.assertNotIn("struct LaneState", lane_source)
        self.assertNotIn("struct WorkEntry", lane_source)

        state_source = LANE_STATE.read_text(encoding="utf-8")
        for declaration in [
            "pub(crate) struct LaneInner",
            "pub(super) struct LaneState",
            "pub(super) struct WorkEntry",
            "pub(super) struct ActiveEntry",
            "pub(super) struct FencePrerequisite",
            "pub(super) struct TerminalNotification",
        ]:
            self.assertIn(declaration, state_source)

        queue_source = LANE_QUEUE.read_text(encoding="utf-8")
        for function in [
            "pub(super) fn reserve",
            "pub(super) fn release_reservation",
            "pub(super) fn mark_pump_needed",
            "pub(super) fn front_is_runnable",
            "pub(super) fn finish_pre_start_entry",
            "pub(super) fn notify_observers",
        ]:
            self.assertIn(function, queue_source)


if __name__ == "__main__":
    unittest.main()
