from __future__ import annotations

import tempfile
import threading
import time
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.control_plane.actions.executor import ActionExecutor
from tools.session_coordinator.control_plane.actions.fingerprint import ActionFingerprinter
from tools.session_coordinator.control_plane.actions.models import ActionContext, ActionKind
from tools.session_coordinator.control_plane.actions.service import ActionService
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError, SupervisionState, WebControlRole
from tools.session_coordinator.supervision.lifecycle import LifecycleService
from tools.session_coordinator.supervision.models import LifecycleKind
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

    def test_confirmed_stop_can_be_cancelled_while_still_draining(self) -> None:
        self.maintenance_active.set()
        stopping = self._confirm(ActionKind.SERVICE_STOP)

        cancelled = self.actions.cancel(
            self.context,
            stopping.action_id,
            reason="operator cancelled before shutdown",
        )
        self.maintenance_active.clear()

        self.assertEqual("cancelled", cancelled.status.value)
        self.assertEqual("healthy", self.supervision.snapshot().state.value)
        self.assertFalse(self.shutdown.wait(0.1))
        with self.database.connect() as connection:
            intent = connection.execute(
                "SELECT status, error_code FROM service_lifecycle_intents WHERE action_id=?",
                (stopping.action_id,),
            ).fetchone()
        self.assertEqual("cancelled", intent["status"])
        self.assertEqual("lifecycle_cancelled", intent["error_code"])

    def test_resume_atomically_cancels_an_active_stop_drain(self) -> None:
        self.maintenance_active.set()
        stopping = self._confirm(ActionKind.SERVICE_STOP)

        resumed = self._confirm(ActionKind.SERVICE_RESUME)
        self.maintenance_active.clear()

        self.assertEqual("succeeded", resumed.status.value)
        self.assertEqual(
            "cancelled", self.actions.get(self.context, stopping.action_id).status.value
        )
        self.assertEqual("healthy", self.supervision.snapshot().state.value)
        self.assertFalse(self.shutdown.wait(0.1))

    def test_second_reversible_lifecycle_is_rejected_before_resume(self) -> None:
        self.maintenance_active.set()
        stopping = self._confirm(ActionKind.SERVICE_STOP)

        with self.assertRaises(CoordinatorError) as rejected:
            self._confirm(ActionKind.SERVICE_RESTART)

        self.assertEqual("lifecycle_already_active", rejected.exception.code)
        resumed = self._confirm(ActionKind.SERVICE_RESUME)
        self.maintenance_active.clear()
        self.assertEqual("succeeded", resumed.status.value)
        self.assertEqual(
            "cancelled", self.actions.get(self.context, stopping.action_id).status.value
        )
        self.assertFalse(self.shutdown.wait(0.1))

    def test_worker_start_failure_atomically_releases_reversible_lifecycle(self) -> None:
        preview = self.actions.preview(
            self.context, ActionKind.SERVICE_STOP.value, {"timeoutSeconds": 5}
        )
        with patch(
            "tools.session_coordinator.supervision.lifecycle.threading.Thread.start",
            side_effect=RuntimeError("thread resources exhausted"),
        ):
            with self.assertRaises(CoordinatorError) as failed:
                self.actions.confirm(
                    self.context,
                    preview.action_id,
                    phrase=preview.confirmation_phrase or "",
                    reason="exercise lifecycle worker compensation",
                )

        self.assertEqual("action_execution_failed", failed.exception.code)
        self.assertEqual("healthy", self.supervision.snapshot().state.value)
        with self.database.connect() as connection:
            intent = connection.execute(
                "SELECT status, error_code, completed_at FROM service_lifecycle_intents "
                "WHERE action_id=?",
                (preview.action_id,),
            ).fetchone()
            action = connection.execute(
                "SELECT status, error_code, completed_at FROM action_requests WHERE action_id=?",
                (preview.action_id,),
            ).fetchone()
        self.assertEqual(("failed", "lifecycle_request_failed", True),
                         (intent["status"], intent["error_code"], intent["completed_at"] is not None))
        self.assertEqual(("failed", "lifecycle_request_failed", True),
                         (action["status"], action["error_code"], action["completed_at"] is not None))

        self.maintenance_active.set()
        replacement = self._confirm(ActionKind.SERVICE_STOP)
        resumed = self._confirm(ActionKind.SERVICE_RESUME)
        self.maintenance_active.clear()
        self.assertEqual("executing", replacement.status.value)
        self.assertEqual("succeeded", resumed.status.value)

    def test_resume_reconciles_active_intent_whose_action_is_already_terminal(self) -> None:
        orphan = self.actions.preview(
            self.context, ActionKind.SERVICE_STOP.value, {"timeoutSeconds": 5}
        )
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE action_requests SET status='failed', error_code='injected', "
                "completed_at='now' WHERE action_id=?",
                (orphan.action_id,),
            )
        self.supervision.create_intent(
            LifecycleKind.STOP,
            action_id=orphan.action_id,
            actor="test",
            deadline_at="later",
        )
        self.supervision.transition(
            SupervisionState.DRAINING,
            reason_code="test.orphan",
            actor="test",
            action_id=orphan.action_id,
        )

        resumed = self._confirm(ActionKind.SERVICE_RESUME)

        self.assertEqual("succeeded", resumed.status.value)
        with self.database.connect() as connection:
            status = connection.execute(
                "SELECT status, error_code FROM service_lifecycle_intents WHERE action_id=?",
                (orphan.action_id,),
            ).fetchone()
        self.assertEqual(("failed", "lifecycle_orphan_reconciled"), tuple(status))

        self.maintenance_active.set()
        replacement = self._confirm(ActionKind.SERVICE_STOP)
        self.actions.cancel(
            self.context, replacement.action_id, reason="cleanup replacement lifecycle"
        )
        self.maintenance_active.clear()

    def test_successor_startup_reconciles_old_active_lifecycle(self) -> None:
        orphan = self.actions.preview(
            self.context, ActionKind.SERVICE_STOP.value, {"timeoutSeconds": 5}
        )
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE action_requests SET status='executing' WHERE action_id=?",
                (orphan.action_id,),
            )
        self.supervision.create_intent(
            LifecycleKind.STOP,
            action_id=orphan.action_id,
            actor="test",
            deadline_at="later",
        )

        successor = SupervisionService(
            self.database,
            repository_key="repo",
            daemon_instance_id="daemon-b",
            process_creation_time="created-b",
        )
        successor.initialize(start_reason="recovery.startup")
        successor.mark_healthy()
        recovered = LifecycleService(successor).recover_restart_intents()

        self.assertEqual(1, recovered)
        with self.database.connect() as connection:
            intent = connection.execute(
                "SELECT status, error_code FROM service_lifecycle_intents WHERE action_id=?",
                (orphan.action_id,),
            ).fetchone()
            action = connection.execute(
                "SELECT status, error_code FROM action_requests WHERE action_id=?",
                (orphan.action_id,),
            ).fetchone()
        self.assertEqual(("failed", "lifecycle_orphan_recovered"), tuple(intent))
        self.assertEqual(("failed", "lifecycle_orphan_recovered"), tuple(action))

        fresh = self.actions.preview(
            self.context, ActionKind.SERVICE_STOP.value, {"timeoutSeconds": 5}
        )
        fresh_intent = successor.create_intent(
            LifecycleKind.STOP,
            action_id=fresh.action_id,
            actor="test",
            deadline_at="later",
        )
        self.assertTrue(fresh_intent)

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

    def test_force_stop_keeps_transport_available_until_terminal_acknowledgement(self) -> None:
        forcing = self._confirm(ActionKind.SERVICE_FORCE_STOP)
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            record = self.actions.get(self.context, forcing.action_id)
            state = self.supervision.snapshot().state.value
            if record.status.value == "succeeded" and state == "offline":
                break
            time.sleep(0.01)
        else:
            self.fail("force-stop action did not reach durable succeeded and offline")

        self.assertEqual("offline", self.supervision.snapshot().state.value)
        self.assertFalse(self.shutdown.wait(0.1))
        acknowledged = self.lifecycle.acknowledge_force_stop(forcing.action_id)

        self.assertTrue(acknowledged["acknowledged"])
        self.assertTrue(self.shutdown.wait(1))

    def test_force_stop_timer_start_failure_keeps_terminal_proof_and_closes(self) -> None:
        with patch(
            "tools.session_coordinator.supervision.lifecycle.threading.Timer.start",
            side_effect=RuntimeError("timer resources exhausted"),
        ):
            forcing = self._confirm(ActionKind.SERVICE_FORCE_STOP)
            self.assertTrue(self.shutdown.wait(2))

        with self.database.connect() as connection:
            intent = connection.execute(
                "SELECT status, error_code FROM service_lifecycle_intents WHERE action_id=?",
                (forcing.action_id,),
            ).fetchone()
            action = connection.execute(
                "SELECT status, error_code FROM action_requests WHERE action_id=?",
                (forcing.action_id,),
            ).fetchone()
        self.assertEqual(("succeeded", None), tuple(intent))
        self.assertEqual(("succeeded", None), tuple(action))
        self.assertEqual("offline", self.supervision.snapshot().state.value)

    def test_force_stop_ack_schedule_failure_preserves_original_fallback(self) -> None:
        forcing = self._confirm(ActionKind.SERVICE_FORCE_STOP)
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            if (
                self.actions.get(self.context, forcing.action_id).status.value
                == "succeeded"
                and self.supervision.snapshot().state.value == "offline"
            ):
                break
            time.sleep(0.01)
        else:
            self.fail("force-stop action did not reach terminal offline proof")

        with patch(
            "tools.session_coordinator.supervision.lifecycle.threading.Timer.start",
            side_effect=RuntimeError("timer resources exhausted"),
        ):
            with self.assertRaises(CoordinatorError) as failed:
                self.lifecycle.acknowledge_force_stop(forcing.action_id)

        self.assertEqual("force_stop_ack_schedule_failed", failed.exception.code)
        fallback = self.lifecycle._force_stop_timers.pop(forcing.action_id)
        fallback.cancel()
        with self.database.connect() as connection:
            intent = connection.execute(
                "SELECT status FROM service_lifecycle_intents WHERE action_id=?",
                (forcing.action_id,),
            ).fetchone()[0]
        self.assertEqual("succeeded", intent)

    def test_force_stop_ack_callback_retries_transient_shutdown_failure(self) -> None:
        calls: list[LifecycleKind] = []
        closed = threading.Event()

        def transient_shutdown(kind: LifecycleKind) -> None:
            calls.append(kind)
            if len(calls) == 1:
                raise RuntimeError("injected ack callback failure")
            closed.set()

        self.lifecycle.set_shutdown(transient_shutdown)
        forcing = self._confirm(ActionKind.SERVICE_FORCE_STOP)
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            if (
                self.actions.get(self.context, forcing.action_id).status.value
                == "succeeded"
                and self.supervision.snapshot().state.value == "offline"
            ):
                break
            time.sleep(0.01)
        else:
            self.fail("force-stop action did not reach terminal offline proof")

        acknowledged = self.lifecycle.acknowledge_force_stop(forcing.action_id)

        self.assertTrue(acknowledged["acknowledged"])
        self.assertTrue(closed.wait(2))
        self.assertEqual([LifecycleKind.FORCE_STOP, LifecycleKind.FORCE_STOP], calls)
        with self.database.connect() as connection:
            intent = connection.execute(
                "SELECT status, error_code FROM service_lifecycle_intents WHERE action_id=?",
                (forcing.action_id,),
            ).fetchone()
        self.assertEqual(("succeeded", None), tuple(intent))

    def test_repeated_force_stop_ack_is_single_flight(self) -> None:
        calls: list[LifecycleKind] = []
        closed = threading.Event()

        def shutdown(kind: LifecycleKind) -> None:
            calls.append(kind)
            closed.set()

        self.lifecycle.set_shutdown(shutdown)
        forcing = self._confirm(ActionKind.SERVICE_FORCE_STOP)
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            if (
                self.actions.get(self.context, forcing.action_id).status.value
                == "succeeded"
                and self.supervision.snapshot().state.value == "offline"
            ):
                break
            time.sleep(0.01)
        else:
            self.fail("force-stop action did not reach terminal offline proof")

        first = self.lifecycle.acknowledge_force_stop(forcing.action_id)
        second = self.lifecycle.acknowledge_force_stop(forcing.action_id)

        self.assertTrue(first["acknowledged"])
        self.assertTrue(second["acknowledged"])
        self.assertTrue(second["alreadyAcknowledged"])
        self.assertTrue(closed.wait(2))
        time.sleep(0.3)
        self.assertEqual([LifecycleKind.FORCE_STOP], calls)

    def test_restart_shutdown_transient_failure_retries_without_rewriting_terminal_state(self) -> None:
        calls: list[LifecycleKind] = []
        closed = threading.Event()

        def transient_shutdown(kind: LifecycleKind) -> None:
            calls.append(kind)
            if len(calls) == 1:
                raise RuntimeError("injected shutdown failure")
            closed.set()

        self.lifecycle.set_shutdown(transient_shutdown)
        restarting = self._confirm(ActionKind.SERVICE_RESTART)

        self.assertTrue(closed.wait(2))
        with self.database.connect() as connection:
            intent = connection.execute(
                "SELECT status, error_code FROM service_lifecycle_intents WHERE action_id=?",
                (restarting.action_id,),
            ).fetchone()
            action = connection.execute(
                "SELECT status, error_code FROM action_requests WHERE action_id=?",
                (restarting.action_id,),
            ).fetchone()
        self.assertEqual([LifecycleKind.RESTART, LifecycleKind.RESTART], calls)
        self.assertEqual(("awaiting_restart", None), tuple(intent))
        self.assertEqual(("executing", None), tuple(action))
        self.assertEqual("offline", self.supervision.snapshot().state.value)


if __name__ == "__main__":
    unittest.main()
