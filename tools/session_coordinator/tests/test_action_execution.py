from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.control_plane.actions.catalog import action_spec
from tools.session_coordinator.control_plane.actions.executor import ActionExecutor
from tools.session_coordinator.control_plane.actions.fingerprint import ActionFingerprinter
from tools.session_coordinator.control_plane.actions.models import ActionContext, ActionKind
from tools.session_coordinator.control_plane.actions.service import ActionService
from tools.session_coordinator.database import Database
from tools.session_coordinator.failures import FailureGraphService
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError, SessionStatus, WebControlRole
from tools.session_coordinator.patches import PatchService
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.snapshots import ObjectStore, SnapshotService
from tools.session_coordinator.tests.helpers import init_repo


class ActionExecutionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        self.database = Database(root / "state.sqlite3")
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)
        self.sessions.register(
            session_id="session-a",
            plan_path="docs/plans/runtime/01-feature.md",
            write_scope=["src/feature.py"],
        )
        self.sessions.set_status("session-a", SessionStatus.ACTIVE)
        self.baselines = BaselineService(self.database, self.repo)
        self.baselines.initialize()
        self.leases = LeaseService(
            self.database, PathPolicy(self.repo), ttl_seconds=300, grace_seconds=30
        )
        objects = ObjectStore(self.database, root / "objects")
        snapshots = SnapshotService(self.database, self.repo, objects)
        patches = PatchService(
            self.database, self.repo, objects, snapshots, self.leases, self.sessions
        )
        failures = FailureGraphService(self.database, self.repo)
        executor = ActionExecutor(
            sessions=self.sessions,
            leases=self.leases,
            patches=patches,
            failures=failures,
            workspace_copy=None,
            workflows=None,
        )
        self.service = ActionService(
            self.database,
            ActionFingerprinter(self.database, self.repo, daemon_instance_id="instance-a"),
            executor,
            daemon_instance_id="instance-a",
        )
        self.context = ActionContext(
            actor="cli",
            role=WebControlRole.OPERATOR,
            web_session_id="web-a",
            bound_session_id="session-a",
            daemon_instance_id="instance-a",
        )

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def test_preview_confirm_executes_typed_lease_and_writes_immutable_audit(self) -> None:
        preview = self.service.preview(
            self.context, ActionKind.LEASE_CLAIM.value, {"sessionId": "session-a"}
        )
        result = self.service.confirm(
            self.context,
            preview.action_id,
            phrase=preview.confirmation_phrase,
            reason="edit owned feature",
        )

        self.assertEqual("succeeded", result.status.value)
        self.assertEqual(["src/feature.py"], self.leases.owned_paths("session-a"))
        with self.database.connect() as connection:
            approvals = connection.execute("SELECT COUNT(*) FROM action_approvals").fetchone()[0]
            with self.assertRaises(Exception):
                connection.execute("UPDATE action_approvals SET reason = 'changed'")
        self.assertEqual(1, approvals)

    def test_runtime_confirm_reuses_the_session_binding_from_preview(self) -> None:
        runtime_preview = ActionContext(
            actor="local-runtime",
            role=WebControlRole.MAINTAINER,
            web_session_id=None,
            bound_session_id="session-a",
            daemon_instance_id="instance-a",
        )
        preview = self.service.preview(
            runtime_preview, ActionKind.LEASE_CLAIM.value, {"sessionId": "session-a"}
        )
        runtime_confirm = ActionContext(
            actor="local-runtime",
            role=WebControlRole.MAINTAINER,
            web_session_id=None,
            bound_session_id=None,
            daemon_instance_id="instance-a",
        )

        result = self.service.confirm(
            runtime_confirm,
            preview.action_id,
            phrase=preview.confirmation_phrase,
            reason="runtime protocol confirmation",
        )

        self.assertEqual("succeeded", result.status.value)
        self.assertEqual(["src/feature.py"], self.leases.owned_paths("session-a"))

    def test_denial_cancel_and_execution_failure_are_audited(self) -> None:
        observer = ActionContext(
            actor="cli",
            role=WebControlRole.OBSERVER,
            web_session_id="web-observer",
            bound_session_id="session-a",
            daemon_instance_id="instance-a",
        )
        with self.assertRaises(CoordinatorError) as denied:
            self.service.preview(
                observer, ActionKind.LEASE_CLAIM.value, {"sessionId": "session-a"}
            )
        self.assertEqual("action_permission_denied", denied.exception.code)

        preview = self.service.preview(
            self.context, ActionKind.SESSION_HEARTBEAT.value, {"sessionId": "session-a"}
        )
        cancelled = self.service.cancel(self.context, preview.action_id, reason="no longer needed")
        self.assertEqual("cancelled", cancelled.status.value)

        disabled = action_spec(ActionKind.SERVICE_RESTART.value)
        self.assertFalse(disabled.enabled)
        with self.assertRaises(CoordinatorError) as red:
            self.service.preview(
                self.context, disabled.kind.value, {"sessionId": "session-a"}
            )
        self.assertEqual("action_disabled", red.exception.code)

    def test_preview_only_action_never_issues_an_executable_confirmation(self) -> None:
        preview = self.service.preview(
            self.context, ActionKind.DRAIN_PREVIEW.value, {"sessionId": "session-a"}
        )

        self.assertIsNone(preview.confirmation_phrase)
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.confirm(
                self.context,
                preview.action_id,
                phrase="",
                reason="inspect service drain impact",
            )
        self.assertEqual("action_preview_only", rejected.exception.code)


if __name__ == "__main__":
    unittest.main()
