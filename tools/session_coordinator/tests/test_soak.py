from __future__ import annotations

import json
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.soak import (
    ResourceSample,
    _next_sample_deadline,
    _transition_evidence_complete,
    run_fixture_soak,
    summarize_samples,
)
from tools.session_coordinator.server import RunningCoordinator


class SoakTests(unittest.TestCase):
    def test_sample_deadline_avoids_normal_drift_and_skips_missed_slots(self) -> None:
        self.assertEqual(60.0, _next_sample_deadline(0.0, 1.0, 60.0))
        self.assertEqual(120.0, _next_sample_deadline(60.0, 61.0, 60.0))
        self.assertEqual(185.0, _next_sample_deadline(60.0, 125.0, 60.0))

    def test_rollover_evidence_requires_two_samples_from_exactly_two_instances(self) -> None:
        def sample(instance_id: str, elapsed: float) -> ResourceSample:
            return ResourceSample("sample", elapsed, instance_id, 1, 100, 10)

        predecessor = [sample("one", 0), sample("one", 1)]
        successor = [sample("two", 2), sample("two", 3)]
        self.assertFalse(
            _transition_evidence_complete(predecessor, minimum_sample_count=4)
        )
        self.assertFalse(
            _transition_evidence_complete(
                predecessor + successor[:1], minimum_sample_count=3
            )
        )
        self.assertFalse(
            _transition_evidence_complete(
                predecessor + successor + [sample("one", 4)], minimum_sample_count=4
            )
        )
        self.assertTrue(
            _transition_evidence_complete(
                predecessor + successor, minimum_sample_count=4
            )
        )

    def test_summary_rejects_unbounded_resource_growth(self) -> None:
        samples = [
            ResourceSample("start", 0, "one", 10, 100, 10),
            ResourceSample("one-end", 5, "one", 15, 80 * 1024 * 1024, 200),
            ResourceSample("two-start", 6, "two", 16, 100, 10),
            ResourceSample("end", 10, "two", 20, 100, 10),
        ]
        summary = summarize_samples(
            samples,
            started_at="start",
            completed_at="end",
            duration_seconds=10,
            restart_count=1,
            browser_disconnect_count=1,
            maintenance_tick_count=1,
            errors=[],
        )
        self.assertEqual("failed", summary.status)
        self.assertTrue(any("RSS growth" in error for error in summary.errors))
        self.assertTrue(any("handle growth" in error for error in summary.errors))

    def test_summary_rejects_pre_restart_resource_spike_hidden_by_successor(self) -> None:
        samples = [
            ResourceSample("one-start", 0, "one", 10, 100, 10),
            ResourceSample("one-peak", 1, "one", 11, 80 * 1024 * 1024, 200),
            ResourceSample("one-end", 2, "one", 12, 100, 10),
            ResourceSample("two-start", 3, "two", 13, 100, 10),
            ResourceSample("two-end", 4, "two", 14, 100, 10),
        ]

        summary = summarize_samples(
            samples,
            started_at="start",
            completed_at="end",
            duration_seconds=4,
            restart_count=1,
            browser_disconnect_count=1,
            maintenance_tick_count=1,
            errors=[],
        )

        self.assertEqual("failed", summary.status)
        self.assertTrue(any("RSS peak growth" in error for error in summary.errors))
        self.assertTrue(any("handle peak growth" in error for error in summary.errors))

    def test_summary_applies_a_distinct_bounded_rollover_gap(self) -> None:
        samples = [
            ResourceSample("one-start", 0, "one", 10, 100, 10),
            ResourceSample("one-end", 1, "one", 11, 100, 10),
            ResourceSample("two-start", 7, "two", 12, 100, 10),
            ResourceSample("two-end", 8, "two", 13, 100, 10),
        ]

        accepted = summarize_samples(
            samples,
            started_at="start",
            completed_at="end",
            duration_seconds=8,
            minimum_sample_count=4,
            maximum_sample_gap_seconds=5,
            maximum_transition_gap_seconds=10,
            restart_count=1,
            browser_disconnect_count=1,
            maintenance_tick_count=1,
            errors=[],
        )
        rejected = summarize_samples(
            [samples[0], ResourceSample("one-late", 6, "one", 11, 100, 10)],
            started_at="start",
            completed_at="end",
            duration_seconds=6,
            maximum_sample_gap_seconds=5,
            maximum_transition_gap_seconds=10,
            restart_count=1,
            browser_disconnect_count=1,
            maintenance_tick_count=1,
            errors=[],
        )

        self.assertEqual("passed", accepted.status, accepted.errors)
        self.assertTrue(any("sample gap" in error for error in rejected.errors))

    def test_summary_requires_duration_samples_restart_and_periodic_exercises(self) -> None:
        samples = [
            ResourceSample("start", 0, "one", 10, 100, 10),
            ResourceSample("end", 1, "one", 11, 100, 10),
        ]

        summary = summarize_samples(
            samples,
            started_at="start",
            completed_at="end",
            duration_seconds=1,
            expected_duration_seconds=10,
            minimum_sample_count=3,
            minimum_browser_disconnect_count=2,
            minimum_maintenance_tick_count=2,
            maximum_sample_gap_seconds=2,
            restart_count=0,
            browser_disconnect_count=0,
            maintenance_tick_count=0,
            errors=[],
        )

        self.assertEqual("failed", summary.status)
        for fragment in (
            "duration",
            "sample count",
            "exactly one restart",
            "instance transition",
            "browser disconnect count",
            "maintenance tick count",
        ):
            self.assertTrue(any(fragment in error for error in summary.errors), fragment)

    def test_short_fixture_soak_rolls_over_and_preserves_events(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = root / "soak.json"
            workspace = root / "durable-workspace"
            summary = run_fixture_soak(
                duration_seconds=4,
                interval_seconds=0.5,
                output_path=output,
                restart_fraction=0.25,
                work_root=workspace,
            )
            payload = json.loads(output.read_text(encoding="utf-8"))
        self.assertEqual("passed", summary.status, summary.errors)
        self.assertEqual(1, summary.restart_count)
        self.assertGreaterEqual(summary.browser_disconnect_count, 1)
        self.assertGreaterEqual(summary.maintenance_tick_count, 1)
        self.assertGreater(summary.last_event_cursor, summary.first_event_cursor)
        self.assertEqual(summary.sample_count, len(payload["samples"]))
        self.assertFalse(payload["workspaceRetained"])
        self.assertFalse(workspace.exists())

    def test_lost_fixture_state_writes_failure_report_and_retains_durable_workspace(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = root / "soak.json"
            workspace = root / "durable-workspace"
            original_stop = RunningCoordinator.stop

            def stop_then_remove_database(running: RunningCoordinator) -> None:
                original_stop(running)
                deadline = time.monotonic() + 3
                while True:
                    try:
                        running.config.database_path.unlink(missing_ok=True)
                        break
                    except PermissionError:
                        if time.monotonic() >= deadline:
                            raise
                        time.sleep(0.05)

            with mock.patch.object(RunningCoordinator, "stop", stop_then_remove_database):
                summary = run_fixture_soak(
                    duration_seconds=0.1,
                    interval_seconds=0.2,
                    output_path=output,
                    restart_fraction=0.99,
                    work_root=workspace,
                )

            payload = json.loads(output.read_text(encoding="utf-8"))
            self.assertEqual("failed", summary.status)
            self.assertTrue(
                any("fixture state database is missing" in error for error in summary.errors)
            )
            self.assertEqual("failed", payload["summary"]["status"])
            self.assertEqual(str(workspace.resolve()), payload["workspace"])
            self.assertTrue(workspace.is_dir())

    def test_startup_failure_writes_report_and_retains_workspace(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = root / "soak.json"
            workspace = root / "durable-workspace"

            with mock.patch.object(
                RunningCoordinator,
                "start",
                side_effect=RuntimeError("fixture startup failed"),
            ):
                summary = run_fixture_soak(
                    duration_seconds=1,
                    interval_seconds=0.1,
                    output_path=output,
                    work_root=workspace,
                )

            payload = json.loads(output.read_text(encoding="utf-8"))
            self.assertEqual("failed", summary.status)
            self.assertIn("RuntimeError: fixture startup failed", summary.errors)
            self.assertEqual("failed", payload["summary"]["status"])
            self.assertTrue(payload["workspaceRetained"])
            self.assertTrue(workspace.is_dir())

    def test_fixture_repo_initialization_failure_writes_report_and_retains_workspace(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = root / "soak.json"
            workspace = root / "durable-workspace"

            with mock.patch(
                "tools.session_coordinator.soak._initialize_fixture_repo",
                side_effect=RuntimeError("fixture git init failed"),
            ):
                summary = run_fixture_soak(
                    duration_seconds=1,
                    interval_seconds=0.1,
                    output_path=output,
                    work_root=workspace,
                )

            payload = json.loads(output.read_text(encoding="utf-8"))
            self.assertEqual("failed", summary.status)
            self.assertIn("RuntimeError: fixture git init failed", summary.errors)
            self.assertEqual("failed", payload["summary"]["status"])
            self.assertTrue(payload["workspaceRetained"])
            self.assertTrue(workspace.is_dir())


if __name__ == "__main__":
    unittest.main()
