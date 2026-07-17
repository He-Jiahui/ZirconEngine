from __future__ import annotations

import unittest

from tools.session_coordinator.resource_budget import (
    BURST_MIN_FREE_BYTES,
    BURST_MIN_FREE_MEMORY_BYTES,
    BURST_SAMPLE_COUNT,
    ResourceSample,
    SystemTimes,
    WindowsResourceProbe,
    burst_decision,
)


class ResourceBudgetTests(unittest.TestCase):
    def test_three_headroom_samples_allow_one_burst(self) -> None:
        decision = burst_decision(
            (
                ResourceSample(cpu_percent=52.5, free_memory_bytes=BURST_MIN_FREE_MEMORY_BYTES),
                ResourceSample(cpu_percent=79.9, free_memory_bytes=BURST_MIN_FREE_MEMORY_BYTES + 1),
                ResourceSample(cpu_percent=10.0, free_memory_bytes=BURST_MIN_FREE_MEMORY_BYTES + 2),
            ),
            free_bytes=BURST_MIN_FREE_BYTES,
            burst_active=False,
        )

        self.assertTrue(decision.allowed)
        self.assertEqual("allowed", decision.reason)

    def test_burst_decision_reports_the_first_unavailable_resource(self) -> None:
        samples = tuple(
            ResourceSample(cpu_percent=20.0, free_memory_bytes=BURST_MIN_FREE_MEMORY_BYTES)
            for _ in range(BURST_SAMPLE_COUNT)
        )

        self.assertEqual(
            "burst_active",
            burst_decision(samples, free_bytes=BURST_MIN_FREE_BYTES, burst_active=True).reason,
        )
        self.assertEqual(
            "disk_headroom",
            burst_decision(samples, free_bytes=BURST_MIN_FREE_BYTES - 1, burst_active=False).reason,
        )
        self.assertEqual(
            "cpu_headroom",
            burst_decision(
                (*samples[:2], ResourceSample(cpu_percent=80.1, free_memory_bytes=BURST_MIN_FREE_MEMORY_BYTES)),
                free_bytes=BURST_MIN_FREE_BYTES,
                burst_active=False,
            ).reason,
        )
        self.assertEqual(
            "memory_headroom",
            burst_decision(
                (*samples[:2], ResourceSample(cpu_percent=20.0, free_memory_bytes=BURST_MIN_FREE_MEMORY_BYTES - 1)),
                free_bytes=BURST_MIN_FREE_BYTES,
                burst_active=False,
            ).reason,
        )

    def test_burst_decision_rejects_an_incomplete_sample_window(self) -> None:
        decision = burst_decision(
            (ResourceSample(cpu_percent=1.0, free_memory_bytes=BURST_MIN_FREE_MEMORY_BYTES),),
            free_bytes=BURST_MIN_FREE_BYTES,
            burst_active=False,
        )

        self.assertFalse(decision.allowed)
        self.assertEqual("cpu_headroom", decision.reason)

    def test_windows_probe_clamps_invalid_system_time_delta(self) -> None:
        snapshots = iter((SystemTimes(idle=10, kernel=10, user=10), SystemTimes(idle=10, kernel=10, user=10)))
        probe = WindowsResourceProbe(
            read_system_times=lambda: next(snapshots),
            read_free_memory=lambda: BURST_MIN_FREE_MEMORY_BYTES,
            sleep=lambda _seconds: None,
        )

        sample = probe.sample()

        self.assertEqual(100.0, sample.cpu_percent)
        self.assertEqual(BURST_MIN_FREE_MEMORY_BYTES, sample.free_memory_bytes)

    def test_windows_probe_derives_busy_cpu_from_system_time_deltas(self) -> None:
        snapshots = iter((SystemTimes(idle=100, kernel=300, user=200), SystemTimes(idle=130, kernel=350, user=250)))
        probe = WindowsResourceProbe(
            read_system_times=lambda: next(snapshots),
            read_free_memory=lambda: BURST_MIN_FREE_MEMORY_BYTES,
            sleep=lambda _seconds: None,
        )

        sample = probe.sample()

        self.assertEqual(70.0, sample.cpu_percent)


if __name__ == "__main__":
    unittest.main()
