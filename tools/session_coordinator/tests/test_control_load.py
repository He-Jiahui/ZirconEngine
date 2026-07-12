from __future__ import annotations

import statistics
import tempfile
import time
import unittest
from contextlib import ExitStack
from pathlib import Path

from tools.session_coordinator.control_plane.artifact_downloads import ArtifactDownloadService
from tools.session_coordinator.control_plane.actions.executor import ActionExecutor
from tools.session_coordinator.control_plane.actions.fingerprint import ActionFingerprinter
from tools.session_coordinator.control_plane.actions.models import ActionContext, ActionKind
from tools.session_coordinator.control_plane.actions.service import ActionService
from tools.session_coordinator.control_plane.events import EventStreamService
from tools.session_coordinator.control_plane.snapshot import ControlSnapshotService
from tools.session_coordinator.client import CoordinatorClient
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import WebControlRole
from tools.session_coordinator.server import RunningCoordinator
from tools.session_coordinator.tests.load_fixture import ControlLoadFixture, ControlLoadShape
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.workflows.projections import WorkflowProjectionService


def percentile95(samples: list[float]) -> float:
    return statistics.quantiles(samples, n=100, method="inclusive")[94]


class ControlLoadTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.stack = ExitStack()
        cls.root = Path(cls.stack.enter_context(tempfile.TemporaryDirectory()))
        cls.repo = init_repo(cls.root / "repo")
        cls.database = Database(cls.root / "state.sqlite3")
        migrate(cls.database)
        cls.shape = ControlLoadShape()
        ControlLoadFixture(cls.database, cls.root / "artifacts", cls.shape).seed()
        cls.projections = WorkflowProjectionService()
        cls.snapshot = ControlSnapshotService(
            cls.database,
            cls.projections,
            lambda connection: {
                "status": "ok",
                "eventCount": connection.execute("SELECT COUNT(*) FROM events").fetchone()[0],
            },
        )

    @classmethod
    def tearDownClass(cls) -> None:
        cls.stack.close()

    def _measure(self, operation, repetitions: int = 20) -> list[float]:
        operation()
        samples = []
        for _ in range(repetitions):
            started = time.perf_counter()
            operation()
            samples.append((time.perf_counter() - started) * 1_000)
        return samples

    def test_exact_release_scale_is_seeded(self) -> None:
        with self.database.connect() as connection:
            counts = {
                table: connection.execute(f"SELECT COUNT(*) FROM {table}").fetchone()[0]
                for table in (
                    "sessions",
                    "workflow_runs",
                    "workflow_nodes",
                    "events",
                    "workflow_artifacts",
                )
            }
        self.assertEqual(self.shape.sessions, counts["sessions"])
        self.assertEqual(self.shape.workflows, counts["workflow_runs"])
        self.assertEqual(self.shape.nodes, counts["workflow_nodes"])
        self.assertEqual(self.shape.events, counts["events"])
        self.assertEqual(self.shape.artifacts, counts["workflow_artifacts"])

    def test_snapshot_and_list_p95_targets(self) -> None:
        snapshot_p95 = percentile95(self._measure(self.snapshot.build))

        def list_workflows() -> None:
            with self.database.connect() as connection:
                self.projections.workflow_summaries(connection)

        list_p95 = percentile95(self._measure(list_workflows))
        print(
            f"M6_METRIC snapshot_p95_ms={snapshot_p95:.3f} "
            f"workflow_list_p95_ms={list_p95:.3f}"
        )
        self.assertLess(snapshot_p95, 800, f"snapshot P95 was {snapshot_p95:.1f} ms")
        self.assertLess(list_p95, 300, f"workflow list P95 was {list_p95:.1f} ms")

    def test_event_replay_and_eight_client_capacity(self) -> None:
        events = EventStreamService(self.database)
        latest = self.shape.events
        replay_p95 = percentile95(self._measure(lambda: events.read_after(latest - 256)))
        print(f"M6_METRIC event_replay_p95_ms={replay_p95:.3f}")
        self.assertLess(replay_p95, 500, f"event replay P95 was {replay_p95:.1f} ms")
        with ExitStack() as stack:
            for _ in range(8):
                stack.enter_context(events.client_slot())
            with self.assertRaisesRegex(Exception, "capacity"):
                with events.client_slot():
                    pass

    def test_500mb_log_supports_bounded_range_reads(self) -> None:
        downloads = ArtifactDownloadService(self.database, self.root / "artifacts")
        response = downloads.download("artifact-000-000", "bytes=524287744-524287999")
        self.assertEqual(206, response.status)
        self.assertEqual(256, len(response.body))
        self.assertEqual(
            "bytes 524287744-524287999/524288000",
            response.headers["Content-Range"],
        )

    def test_z_health_and_action_preview_p95_targets(self) -> None:
        config = CoordinatorConfig.for_repo(
            self.repo,
            state_root=self.root / "health-state",
            port=0,
        )
        with RunningCoordinator.start(config):
            client = CoordinatorClient.from_runtime(config)
            health_p95 = percentile95(self._measure(client.health, repetitions=30))

        action_service = ActionService(
            self.database,
            ActionFingerprinter(
                self.database,
                self.repo,
                daemon_instance_id="load-instance",
            ),
            ActionExecutor(
                sessions=None,
                leases=None,
                patches=None,
                failures=None,
                workspace_copy=None,
                workflows=None,
            ),
            daemon_instance_id="load-instance",
        )
        context = ActionContext(
            actor="load-fixture",
            role=WebControlRole.OPERATOR,
            web_session_id="load-web-session",
            bound_session_id="load-session-000",
            daemon_instance_id="load-instance",
        )
        action_p95 = percentile95(
            self._measure(
                lambda: action_service.preview(
                    context,
                    ActionKind.SESSION_HEARTBEAT.value,
                    {"sessionId": "load-session-000"},
                )
            )
        )
        print(
            f"M6_METRIC health_p95_ms={health_p95:.3f} "
            f"action_preview_p95_ms={action_p95:.3f}"
        )
        self.assertLess(health_p95, 100, f"health P95 was {health_p95:.1f} ms")
        self.assertLess(action_p95, 1_000, f"action preview P95 was {action_p95:.1f} ms")


if __name__ == "__main__":
    unittest.main()
