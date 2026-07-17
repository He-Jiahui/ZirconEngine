from __future__ import annotations

import unittest
from pathlib import Path

from tools.session_coordinator.cpu_burst import CpuBurstRequest, select_cpu_burst
from tools.session_coordinator.resource_budget import BurstDecision


class CpuBurstTests(unittest.TestCase):
    def test_eligible_cpu_check_uses_its_own_ephemeral_target(self) -> None:
        selection = select_cpu_burst(
            CpuBurstRequest(
                reservation_id="reservation-a",
                lane_scope="cpu",
                burst_eligible=True,
                command=("cargo", "check", "-p", "zircon_runtime"),
                target_dir=None,
            ),
            BurstDecision(True, "allowed"),
        )

        self.assertEqual("burst", selection.mode)
        self.assertEqual("allowed", selection.reason)
        self.assertEqual(
            Path("E:/cargo-targets/zircon-engine/burst/reservation-a"),
            selection.target_dir,
        )

    def test_test_and_explicit_target_stay_in_the_warm_lane(self) -> None:
        test_selection = select_cpu_burst(
            CpuBurstRequest(
                reservation_id="reservation-test",
                lane_scope="cpu",
                burst_eligible=True,
                command=("cargo", "test", "-p", "zircon_runtime"),
                target_dir=None,
            ),
            BurstDecision(True, "allowed"),
        )
        target_selection = select_cpu_burst(
            CpuBurstRequest(
                reservation_id="reservation-target",
                lane_scope="cpu",
                burst_eligible=True,
                command=("cargo", "check", "-p", "zircon_runtime"),
                target_dir="D:/cargo-targets/managed",
            ),
            BurstDecision(True, "allowed"),
        )

        self.assertEqual(("warm", None, "not_eligible"), test_selection.as_tuple())
        self.assertEqual(("warm", None, "not_eligible"), target_selection.as_tuple())

    def test_resource_denial_is_preserved_without_consuming_warm_fifo(self) -> None:
        selection = select_cpu_burst(
            CpuBurstRequest(
                reservation_id="reservation-b",
                lane_scope="cpu",
                burst_eligible=True,
                command=("cargo", "check"),
                target_dir=None,
            ),
            BurstDecision(False, "cpu_headroom"),
        )

        self.assertEqual(("warm", None, "cpu_headroom"), selection.as_tuple())


if __name__ == "__main__":
    unittest.main()
