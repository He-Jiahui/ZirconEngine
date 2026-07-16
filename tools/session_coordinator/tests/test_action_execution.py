from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.control_plane.actions.catalog import action_spec
from tools.session_coordinator.control_plane.actions.executor import ActionExecutor
from tools.session_coordinator.control_plane.actions.fingerprint import ActionFingerprinter
from tools.session_coordinator.control_plane.actions.models import (
    ActionContext,
    ActionKind,
    SessionParameters,
    ValidationTemplate,
)
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
        self.sessions.register(
            session_id="session-b",
            plan_path="docs/plans/runtime/02-feature.md",
            write_scope=["src/other.py"],
        )
        self.sessions.set_status("session-b", SessionStatus.ACTIVE)
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
        self.executor = ActionExecutor(
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
            self.executor,
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

        maintenance = action_spec(ActionKind.MAINTENANCE_CLEANUP.value)
        self.assertTrue(maintenance.enabled)
        with self.assertRaises(CoordinatorError) as red:
            self.service.preview(
                self.context, maintenance.kind.value, {"sessionId": "session-a"}
            )
        self.assertEqual("action_permission_denied", red.exception.code)

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

    def test_maintainer_reconciles_accepted_milestone_evidence_through_audited_action(self) -> None:
        reconciliation = {
            "auditId": "placeholder",
            "sourceRunId": "source-run",
            "targetRunId": "target-run",
            "nodes": [{"milestoneId": "M3", "state": "succeeded"}],
        }
        milestones = SimpleNamespace(
            reconcile_accepted_milestones=mock.Mock(return_value=reconciliation)
        )
        self.executor.milestones = milestones
        with self.database.transaction() as connection:
            for run_id, session_id, version_id in (
                ("source-run", "session-a", "source-version"),
                ("target-run", "session-b", "target-version"),
            ):
                connection.execute(
                    """INSERT INTO workflow_runs(
                           run_id, session_id, workflow_key, plan_path, topology_hash,
                           state, created_at, updated_at, current_topology_version_id
                       ) VALUES (?, ?, 'workflow', 'docs/plans/runtime/01-feature.md',
                                 'topology', 'active', 'now', 'now', NULL)""",
                    (run_id, session_id),
                )
                connection.execute(
                    """INSERT INTO workflow_topology_versions(
                           topology_version_id, run_id, version_number, plan_path, plan_id,
                           schema_version, source_kind, content_hash, topology_hash,
                           topology_json, created_at
                       ) VALUES (?, ?, 1, 'docs/plans/runtime/01-feature.md', '01',
                                 1, 'zircon-workflow', 'content', 'topology', '{}', 'now')""",
                    (version_id, run_id),
                )
                connection.execute(
                    "UPDATE workflow_runs SET current_topology_version_id=? WHERE run_id=?",
                    (version_id, run_id),
                )
        maintainer = ActionContext(
            actor="maintainer-cli",
            role=WebControlRole.MAINTAINER,
            web_session_id="web-maintainer",
            bound_session_id=None,
            daemon_instance_id="instance-a",
        )
        parameters = {
            "sourceRunId": "source-run",
            "targetRunId": "target-run",
            "milestoneIds": ["M3"],
        }
        preview = self.service.preview(
            maintainer, ActionKind.MILESTONE_RECONCILE.value, parameters
        )
        result = self.service.confirm(
            maintainer,
            preview.action_id,
            phrase=preview.confirmation_phrase,
            reason="reconcile immutable historical evidence",
        )

        self.assertEqual("succeeded", result.status.value)
        milestones.reconcile_accepted_milestones.assert_called_once_with(
            source_run_id="source-run",
            target_run_id="target-run",
            milestone_keys=("M3",),
            actor="maintainer-cli",
            action_id=preview.action_id,
        )

    def test_action_activity_is_latest_first_bounded_and_identity_scoped(self) -> None:
        first = self.service.preview(
            self.context, ActionKind.SESSION_HEARTBEAT.value, {"sessionId": "session-a"}
        )
        second = self.service.preview(
            self.context, ActionKind.SESSION_HEARTBEAT.value, {"sessionId": "session-a"}
        )
        self.service.preview(
            ActionContext(
                actor="other-actor",
                role=WebControlRole.OPERATOR,
                web_session_id="web-a",
                bound_session_id="session-a",
                daemon_instance_id="instance-a",
            ),
            ActionKind.SESSION_HEARTBEAT.value,
            {"sessionId": "session-a"},
        )
        self.service.preview(
            ActionContext(
                actor="cli",
                role=WebControlRole.OPERATOR,
                web_session_id="other-web",
                bound_session_id="session-a",
                daemon_instance_id="instance-a",
            ),
            ActionKind.SESSION_HEARTBEAT.value,
            {"sessionId": "session-a"},
        )
        self.service.preview(
            ActionContext(
                actor="cli",
                role=WebControlRole.OPERATOR,
                web_session_id="web-a",
                bound_session_id="session-b",
                daemon_instance_id="instance-a",
            ),
            ActionKind.SESSION_HEARTBEAT.value,
            {"sessionId": "session-b"},
        )

        bounded, truncated = self.service.list_activity(self.context, limit=1)
        visible, visible_truncated = self.service.list_activity(self.context, limit=10)

        self.assertEqual([second.action_id], [action.action_id for action in bounded])
        self.assertTrue(truncated)
        self.assertEqual(
            [second.action_id, first.action_id],
            [action.action_id for action in visible],
        )
        self.assertFalse(visible_truncated)
        self.assertTrue(all(action.confirmation_phrase is None for action in visible))

    def test_action_activity_rejects_stale_daemon_identity(self) -> None:
        stale = ActionContext(
            actor=self.context.actor,
            role=self.context.role,
            web_session_id=self.context.web_session_id,
            bound_session_id=self.context.bound_session_id,
            daemon_instance_id="instance-b",
        )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.list_activity(stale, limit=10)

        self.assertEqual("action_instance_mismatch", rejected.exception.code)

    def test_codex_reconcile_action_only_enqueues_worker(self) -> None:
        wake_reasons: list[str] = []
        self.service.executor.codex_wake = lambda reason: wake_reasons.append(reason) or True
        maintainer = ActionContext(
            actor="maintainer",
            role=WebControlRole.MAINTAINER,
            web_session_id="web-maintainer",
            bound_session_id=None,
            daemon_instance_id="instance-a",
        )

        with self.assertRaises(CoordinatorError) as denied:
            self.service.preview(self.context, ActionKind.CODEX_RECONCILE.value, {})
        self.assertEqual("action_permission_denied", denied.exception.code)
        preview = self.service.preview(
            maintainer, ActionKind.CODEX_RECONCILE.value, {}
        )
        result = self.service.confirm(
            maintainer,
            preview.action_id,
            phrase=preview.confirmation_phrase or "",
            reason="reconcile Codex source projection",
        )

        self.assertEqual("succeeded", result.status.value)
        self.assertEqual(["controlled"], wake_reasons)
        self.assertEqual({"queued": True, "trigger": "controlled"}, result.result)

    def test_validation_templates_declare_small_readonly_dependency_roots(self) -> None:
        self.assertEqual(
            ("tools/session_coordinator",),
            ActionExecutor._validation_dependency_roots(ValidationTemplate.COORDINATOR_ACTIONS),
        )
        self.assertEqual(
            ("tools/session_coordinator/web",),
            ActionExecutor._validation_dependency_roots(ValidationTemplate.WEB_CHECK),
        )

    def test_second_validation_for_the_same_milestone_is_rejected_while_first_runs(self) -> None:
        parameters = {
            "sessionId": "session-a",
            "template": ValidationTemplate.COORDINATOR_ACTIONS.value,
            "runId": "run-1",
            "milestoneId": "M1",
        }
        fingerprint = SimpleNamespace(digest="stable", payload={"actionResources": {}})
        with (
            mock.patch.object(self.service.fingerprinter, "capture", return_value=fingerprint),
            mock.patch.object(self.service.fingerprinter, "impact", return_value={}),
        ):
            first = self.service.preview(self.context, ActionKind.VALIDATION_START.value, parameters)
            second = self.service.preview(self.context, ActionKind.VALIDATION_START.value, parameters)
            with self.database.transaction() as connection:
                connection.execute(
                    "UPDATE action_requests SET status='executing' WHERE action_id=?",
                    (first.action_id,),
                )

            with self.assertRaises(CoordinatorError) as rejected:
                self.service.confirm(
                    self.context,
                    second.action_id,
                    phrase=second.confirmation_phrase or "",
                    reason="start the same validation again",
                )

        self.assertEqual("validation_already_running", rejected.exception.code)
        self.assertEqual("previewed", self.service.get(self.context, second.action_id).status.value)

    def test_restart_recovery_finishes_only_actions_from_a_previous_daemon(self) -> None:
        stale = self.service.preview(
            self.context, ActionKind.SESSION_HEARTBEAT.value, {"sessionId": "session-a"}
        )
        current = self.service.preview(
            self.context, ActionKind.SESSION_HEARTBEAT.value, {"sessionId": "session-a"}
        )
        with self.database.transaction() as connection:
            connection.execute(
                """UPDATE action_requests
                   SET status='executing', daemon_instance_id='instance-old'
                   WHERE action_id=?""",
                (stale.action_id,),
            )
            connection.execute(
                "UPDATE action_requests SET status='executing' WHERE action_id=?",
                (current.action_id,),
            )

        recovered = self.service.recover_interrupted_actions()

        self.assertEqual(1, recovered)
        with self.database.connect() as connection:
            stale_row = connection.execute(
                "SELECT status, error_code, completed_at FROM action_requests WHERE action_id=?",
                (stale.action_id,),
            ).fetchone()
            current_row = connection.execute(
                "SELECT status FROM action_requests WHERE action_id=?", (current.action_id,)
            ).fetchone()
        self.assertEqual(("failed", "action_interrupted_by_restart"), tuple(stale_row)[:2])
        self.assertIsNotNone(stale_row["completed_at"])
        self.assertEqual("executing", current_row["status"])

    def test_deferred_completion_closes_an_executing_action(self) -> None:
        preview = self.service.preview(
            self.context, ActionKind.SESSION_HEARTBEAT.value, {"sessionId": "session-a"}
        )
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE action_requests SET status='executing' WHERE action_id=?",
                (preview.action_id,),
            )

        self.service.complete_deferred(
            preview.action_id,
            SessionParameters("session-a"),
            result={"queued": True},
        )

        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status, result_json, completed_at FROM action_requests WHERE action_id=?",
                (preview.action_id,),
            ).fetchone()
        self.assertEqual("succeeded", row["status"])
        self.assertEqual({"queued": True}, json.loads(row["result_json"]))
        self.assertIsNotNone(row["completed_at"])


if __name__ == "__main__":
    unittest.main()
