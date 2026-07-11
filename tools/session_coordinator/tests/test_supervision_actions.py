from __future__ import annotations

import tempfile
import threading
import time
import unittest
from pathlib import Path

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.control_plane.actions.executor import ActionExecutor
from tools.session_coordinator.control_plane.actions.fingerprint import ActionFingerprinter
from tools.session_coordinator.control_plane.actions.models import ActionContext, ActionKind
from tools.session_coordinator.control_plane.actions.service import ActionService
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError, WebControlRole
from tools.session_coordinator.supervision.lifecycle import LifecycleService
from tools.session_coordinator.supervision.service import SupervisionService
from tools.session_coordinator.tests.helpers import init_repo


class SupervisionActionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        root = Path(self.temporary.name)
        self.repo = init_repo(root / "repo")
        self.database = Database(root / "coordinator.sqlite3")
        migrate(self.database)
        BaselineService(self.database, self.repo).initialize()
        self.maintenance_active = threading.Event()
        self.supervision = SupervisionService(
            self.database,
            repository_key="repo",
            daemon_instance_id="daemon-a",
            process_creation_time="created-a",
            maintenance_active=self.maintenance_active.is_set,
        )
        self.supervision.initialize()
        self.supervision.mark_healthy()
        self.shutdown = threading.Event()
        self.lifecycle = LifecycleService(
            self.supervision,
            shutdown=lambda _kind: self.shutdown.set(),
            poll_seconds=0.01,
        )
        executor = ActionExecutor(
            sessions=None,
            leases=None,
            patches=None,
            failures=None,
            workspace_copy=None,
            workflows=None,
            lifecycle=self.lifecycle,
        )
        self.actions = ActionService(
            self.database,
            ActionFingerprinter(
                self.database,
                self.repo,
                daemon_instance_id="daemon-a",
                supervision=self.supervision,
            ),
            executor,
            daemon_instance_id="daemon-a",
            mutation_gate=self.supervision.require_mutation_allowed,
        )
        self.context = ActionContext(
            actor="tray",
            role=WebControlRole.MAINTAINER,
            web_session_id=None,
            bound_session_id=None,
            daemon_instance_id="daemon-a",
        )

    def _confirm(self, kind: ActionKind, timeout: int = 5):
        preview = self.actions.preview(
            self.context, kind.value, {"timeoutSeconds": timeout}
        )
        return self.actions.confirm(
            self.context,
            preview.action_id,
            phrase=preview.confirmation_phrase or "",
            reason=f"test {kind.value}",
        )

    def test_drain_and_resume_share_the_controlled_action_protocol(self) -> None:
        drained = self._confirm(ActionKind.SERVICE_DRAIN)

        self.assertEqual("succeeded", drained.status.value)
        self.assertEqual("draining", self.supervision.snapshot().state.value)
        with self.assertRaises(CoordinatorError):
            self.supervision.require_mutation_allowed("lease.claim")

        resumed = self._confirm(ActionKind.SERVICE_RESUME)

        self.assertEqual("succeeded", resumed.status.value)
        self.assertEqual("healthy", self.supervision.snapshot().state.value)

    def test_stop_remains_executing_until_critical_sections_drain(self) -> None:
        self.maintenance_active.set()
        stopping = self._confirm(ActionKind.SERVICE_STOP)

        self.assertEqual("executing", stopping.status.value)
        self.assertFalse(self.shutdown.wait(0.05))
        self.maintenance_active.clear()
        self.assertTrue(self.shutdown.wait(2))
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            record = self.actions.get(self.context, stopping.action_id)
            if record.status.value == "succeeded":
                break
            time.sleep(0.01)
        else:
            self.fail("stop action did not reach succeeded")

        snapshot = self.supervision.snapshot()
        self.assertEqual("offline", snapshot.state.value)
        self.assertTrue(snapshot.explicit_stop)

    def test_restart_is_completed_only_by_successor_daemon(self) -> None:
        restarting = self._confirm(ActionKind.SERVICE_RESTART)
        self.assertTrue(self.shutdown.wait(2))
        self.assertEqual("executing", self.actions.get(self.context, restarting.action_id).status.value)

        successor = SupervisionService(
            self.database,
            repository_key="repo",
            daemon_instance_id="daemon-b",
            process_creation_time="created-b",
        )
        successor.initialize(start_reason="recovery.restart")
        successor.mark_healthy()
        recovered = LifecycleService(successor).recover_restart_intents()

        self.assertEqual(1, recovered)
        with self.database.connect() as connection:
            action_status = connection.execute(
                "SELECT status FROM action_requests WHERE action_id=?",
                (restarting.action_id,),
            ).fetchone()[0]
            successor_id = connection.execute(
                "SELECT successor_daemon_instance_id FROM service_lifecycle_intents "
                "WHERE action_id=?",
                (restarting.action_id,),
            ).fetchone()[0]
        self.assertEqual("succeeded", action_status)
        self.assertEqual("daemon-b", successor_id)


if __name__ == "__main__":
    unittest.main()
