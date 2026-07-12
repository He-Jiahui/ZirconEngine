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
from tools.session_coordinator.failures import FailureGraphService
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError, SessionStatus, WebControlRole
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class ActionConcurrencyTests(unittest.TestCase):
    def test_state_change_invalidates_confirm_without_side_effect_repeatedly(self) -> None:
        for attempt in range(20):
            with self.subTest(attempt=attempt), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                repo = init_repo(root / "repo")
                database = Database(root / "state.sqlite3")
                migrate(database)
                sessions = SessionService(database, repo)
                sessions.register(
                    session_id="session-a",
                    plan_path="docs/plans/runtime/01-feature.md",
                    write_scope=["src/feature.py"],
                )
                sessions.set_status("session-a", SessionStatus.ACTIVE)
                baselines = BaselineService(database, repo)
                baselines.initialize()
                leases = LeaseService(
                    database, PathPolicy(repo), ttl_seconds=300, grace_seconds=30
                )
                service = ActionService(
                    database,
                    ActionFingerprinter(database, repo, daemon_instance_id="instance-a"),
                    ActionExecutor(
                        sessions=sessions,
                        leases=leases,
                        patches=None,
                        failures=FailureGraphService(database, repo),
                        workspace_copy=None,
                        workflows=None,
                    ),
                    daemon_instance_id="instance-a",
                )
                context = ActionContext(
                    actor="cli",
                    role=WebControlRole.OPERATOR,
                    web_session_id="web-a",
                    bound_session_id="session-a",
                    daemon_instance_id="instance-a",
                )
                preview = service.preview(
                    context, ActionKind.LEASE_CLAIM.value, {"sessionId": "session-a"}
                )
                sessions.set_status("session-a", SessionStatus.WAITING_VALIDATION)

                with self.assertRaises(CoordinatorError) as changed:
                    service.confirm(
                        context,
                        preview.action_id,
                        phrase=preview.confirmation_phrase,
                        reason="should be stale",
                    )

                self.assertEqual("action_state_changed", changed.exception.code)
                self.assertEqual([], leases.owned_paths("session-a"))

    def test_confirm_holds_the_shared_mutation_gate_through_the_side_effect(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            sessions = SessionService(database, repo)
            sessions.register(
                session_id="session-a",
                plan_path="docs/plans/runtime/01-feature.md",
                write_scope=["src/feature.py"],
            )
            sessions.set_status("session-a", SessionStatus.ACTIVE)
            BaselineService(database, repo).initialize()
            gate = threading.RLock()
            executor = _BlockingExecutor(sessions)
            service = ActionService(
                database,
                ActionFingerprinter(database, repo, daemon_instance_id="instance-a"),
                executor,
                daemon_instance_id="instance-a",
                mutation_lock=gate,
            )
            context = ActionContext(
                actor="cli",
                role=WebControlRole.OPERATOR,
                web_session_id="web-a",
                bound_session_id="session-a",
                daemon_instance_id="instance-a",
            )
            preview = service.preview(
                context, ActionKind.SESSION_HEARTBEAT.value, {"sessionId": "session-a"}
            )
            confirm_error: list[BaseException] = []

            def confirm() -> None:
                try:
                    service.confirm(
                        context,
                        preview.action_id,
                        phrase=preview.confirmation_phrase,
                        reason="atomic execution",
                    )
                except BaseException as error:  # pragma: no cover - asserted below
                    confirm_error.append(error)

            mutated = threading.Event()

            def mutate() -> None:
                with gate:
                    sessions.set_status("session-a", SessionStatus.WAITING_VALIDATION)
                    mutated.set()

            confirm_thread = threading.Thread(target=confirm)
            confirm_thread.start()
            self.assertTrue(executor.entered.wait(timeout=2))
            mutation_thread = threading.Thread(target=mutate)
            mutation_thread.start()
            time.sleep(0.1)
            self.assertFalse(mutated.is_set())
            executor.release.set()
            confirm_thread.join(timeout=2)
            mutation_thread.join(timeout=2)

            self.assertEqual([], confirm_error)
            self.assertTrue(mutated.is_set())
            self.assertEqual(SessionStatus.ACTIVE, executor.status_during_execute)

    def test_cancel_cannot_cross_confirm_between_preview_and_execution(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            sessions = SessionService(database, repo)
            sessions.register(session_id="session-a")
            sessions.set_status("session-a", SessionStatus.ACTIVE)
            BaselineService(database, repo).initialize()
            executor = _BlockingExecutor(sessions)
            service = ActionService(
                database,
                ActionFingerprinter(database, repo, daemon_instance_id="instance-a"),
                executor,
                daemon_instance_id="instance-a",
            )
            context = ActionContext(
                actor="cli",
                role=WebControlRole.OPERATOR,
                web_session_id="web-a",
                bound_session_id="session-a",
                daemon_instance_id="instance-a",
            )
            preview = service.preview(
                context,
                ActionKind.SESSION_HEARTBEAT.value,
                {"sessionId": "session-a"},
            )
            confirm_errors: list[BaseException] = []
            cancel_errors: list[BaseException] = []
            cancel_finished = threading.Event()

            def confirm() -> None:
                try:
                    service.confirm(
                        context,
                        preview.action_id,
                        phrase=preview.confirmation_phrase,
                        reason="confirm wins while holding the gate",
                    )
                except BaseException as error:  # pragma: no cover - asserted below
                    confirm_errors.append(error)

            def cancel() -> None:
                try:
                    service.cancel(context, preview.action_id, reason="racing cancel")
                except BaseException as error:  # pragma: no cover - asserted below
                    cancel_errors.append(error)
                finally:
                    cancel_finished.set()

            confirm_thread = threading.Thread(target=confirm)
            confirm_thread.start()
            self.assertTrue(executor.entered.wait(timeout=2))
            cancel_thread = threading.Thread(target=cancel)
            cancel_thread.start()
            self.assertFalse(cancel_finished.wait(timeout=0.1))
            executor.release.set()
            confirm_thread.join(timeout=2)
            cancel_thread.join(timeout=2)

            self.assertEqual([], confirm_errors)
            self.assertEqual(1, len(cancel_errors))
            self.assertIsInstance(cancel_errors[0], CoordinatorError)
            self.assertEqual("action_not_cancellable", cancel_errors[0].code)
            self.assertEqual(
                "succeeded", service.get(context, preview.action_id).status.value
            )


class _BlockingExecutor:
    def __init__(self, sessions: SessionService) -> None:
        self.sessions = sessions
        self.entered = threading.Event()
        self.release = threading.Event()
        self.status_during_execute: SessionStatus | None = None

    def execute(
        self, _spec, _parameters, *, resource_snapshot, action_id=None, actor=None
    ):
        self.entered.set()
        if not self.release.wait(timeout=2):
            raise AssertionError("test executor was not released")
        self.status_during_execute = self.sessions.get("session-a").status
        return {"resourceSnapshot": resource_snapshot}


if __name__ == "__main__":
    unittest.main()
