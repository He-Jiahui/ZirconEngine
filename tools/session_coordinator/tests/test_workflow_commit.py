from __future__ import annotations

import subprocess
import tempfile
import unittest
import json
from unittest import mock
from pathlib import Path

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.git_finalize import GitFinalizeService
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import SessionStatus, WorkflowNodeState
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.notifications import WeComNotificationService
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.workflows.gates import GateEvidenceStore, MilestoneGateEvaluator
from tools.session_coordinator.workflows.milestones import MilestoneWorkflowService
from tools.session_coordinator.workflows.plan_import import TopologyImporter
from tools.session_coordinator.workflows.store import WorkflowStore


class WorkflowCommitTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.repo = init_repo(root / "repo")
        plan = self.repo / "docs/plans/runtime/01-control.md"
        plan.parent.mkdir(parents=True)
        plan.write_text(
            "```zircon-workflow\n"
            '{"schema":1,"workflow_id":"runtime-control","goal":"Runtime",'
            '"milestones":[{"id":"M1","title":"Base","depends_on":[]}]}\n'
            "```\n\n## Milestone M1: Base\n\n"
            "- [ ] **M1.1 Add storage.** details\n",
            encoding="utf-8",
        )
        subprocess.run(["git", "add", "--", "docs/plans/runtime/01-control.md"], cwd=self.repo, check=True)
        subprocess.run(["git", "commit", "-q", "-m", "test: add plan"], cwd=self.repo, check=True)
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)
        self.sessions.register(
            session_id="session-a", plan_path="docs/plans/runtime/01-control.md"
        )
        self.sessions.register(session_id="reviewer-b")
        self.sessions.set_status("session-a", SessionStatus.ACTIVE)
        self.baselines = BaselineService(self.database, self.repo)
        self.baselines.initialize()
        self.leases = LeaseService(
            self.database,
            PathPolicy(self.repo),
            ttl_seconds=config.lease_ttl_seconds,
            grace_seconds=config.lease_grace_seconds,
        )
        self.finalize = GitFinalizeService(
            self.database, self.repo, self.baselines, self.sessions
        )
        imported = TopologyImporter(self.database, self.repo).import_plan(
            "session-a", "docs/plans/runtime/01-control.md"
        )
        self.run_id = imported.run_id
        self.topology_version_id = imported.topology_version_id
        store = WorkflowStore(self.database)
        nodes = {item.node_key: item for item in store.nodes(self.run_id)}
        store.append_attempt(
            nodes["M1.1"].node_id, WorkflowNodeState.SUCCEEDED, {"exit": 0}
        )

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def _service(self, returncode: int = 0) -> MilestoneWorkflowService:
        def runner(command: list[str]) -> subprocess.CompletedProcess[str]:
            return subprocess.CompletedProcess(
                command,
                returncode,
                '{"errcode":0}' if returncode == 0 else "",
                "notification failed" if returncode else "",
            )

        notifications = WeComNotificationService(
            self.database, script_path="send.ps1", runner=runner
        )
        return MilestoneWorkflowService(
            self.database,
            self.repo,
            self.baselines,
            self.finalize,
            notifications,
            sessions=self.sessions,
            leases=self.leases,
        )

    def _prepare_change_and_gates(self, service: MilestoneWorkflowService) -> list[str]:
        paths = ["src/runtime.py", "tests/test_runtime.py"]
        for path in paths:
            target = self.repo / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(f"content for {path}\n", encoding="utf-8")
        record = "docs/plans/runtime/01/2026-07-12-m1-record.md"
        output = self.repo / record
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(
            "# M1 output\n\nPlan: docs/plans/runtime/01-control.md\n"
            "Milestone: M1\nStatus: completed\n"
            f"Files: {json.dumps(paths)}\n\n"
            "## Scope delivered\n\nDone.\n\n## Fresh testing evidence\n\nPassed.\n\n"
            "## Review\n\nAccepted.\n",
            encoding="utf-8",
        )
        paths.append(record)
        self.assertTrue(self.leases.acquire("session-a", paths).acquired)
        self.baselines.attribute("session-a", paths)
        subprocess.run(["git", "add", "--", *paths], cwd=self.repo, check=True)
        service.bind_manifest(
            session_id="session-a", run_id=self.run_id, milestone_key="M1",
            actor="session-a", action_id="bind-a",
        )
        context = service.prepare_context(self.run_id, paths)
        evaluator = MilestoneGateEvaluator(self.database)
        fingerprint = evaluator.input_fingerprint(self.run_id, "M1", context)
        evidence = GateEvidenceStore(self.database)
        with self.database.connect() as connection:
            milestone_node_id = connection.execute(
                "SELECT node_id FROM workflow_nodes WHERE run_id=? AND node_key='M1'",
                (self.run_id,),
            ).fetchone()[0]
        evidence.record_review(
            run_id=self.run_id,
            topology_version_id=self.topology_version_id,
            reviewer="reviewer-b",
            executor="session-a",
            critical_count=0,
            important_count=0,
            summary="clean",
            input_fingerprint=fingerprint,
            node_id=milestone_node_id,
        )
        for kind in evaluator.REQUIRED_GATE_KINDS:
            evidence.record_gate(
                run_id=self.run_id,
                topology_version_id=self.topology_version_id,
                gate_kind=kind,
                decision="accepted",
                decision_code="accepted",
                input_fingerprint=fingerprint,
                actor="reviewer-b",
                node_id=milestone_node_id,
            )
        return paths

    def test_gate_revalidated_commit_contains_only_attributed_manifest(self) -> None:
        service = self._service()
        paths = self._prepare_change_and_gates(service)

        result = service.commit(
            session_id="session-a",
            run_id=self.run_id,
            milestone_key="M1",
            paths=paths,
            message="feat(workflow): complete M1 milestone",
            actor="session-a",
        )

        committed = subprocess.run(
            ["git", "show", "--pretty=", "--name-only", result.finalize.commit_sha],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
        self.assertEqual(sorted(paths), sorted(item for item in committed if item))
        self.assertTrue(result.gate.allowed)
        self.assertEqual("succeeded", result.notification.status)
        self.assertEqual(SessionStatus.ACTIVE, self.sessions.get("session-a").status)

    def test_notification_failure_does_not_rollback_commit(self) -> None:
        service = self._service(returncode=1)
        paths = self._prepare_change_and_gates(service)

        result = service.commit(
            session_id="session-a",
            run_id=self.run_id,
            milestone_key="M1",
            paths=paths,
            message="feat(workflow): complete M1 milestone",
            actor="session-a",
        )

        head = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=self.repo, check=True, capture_output=True, text=True
        ).stdout.strip()
        self.assertEqual(result.finalize.commit_sha, head)
        self.assertEqual("failed", result.notification.status)

    def test_post_cas_baseline_failure_is_reconciled_before_workflow_success(self) -> None:
        service = self._service()
        paths = self._prepare_change_and_gates(service)
        original = self.baselines.accept_commit
        calls = 0

        def fail_once(*args, **kwargs):
            nonlocal calls
            calls += 1
            if calls == 1:
                raise RuntimeError("injected post-CAS baseline failure")
            return original(*args, **kwargs)

        with mock.patch.object(self.baselines, "accept_commit", side_effect=fail_once):
            result = service.commit(
                session_id="session-a",
                run_id=self.run_id,
                milestone_key="M1",
                paths=paths,
                message="feat(workflow): complete M1 milestone",
                actor="session-a",
            )

        self.assertEqual(result.finalize.commit_sha, self.baselines.current().head_commit)
        with self.database.connect() as connection:
            finalize = connection.execute(
                "SELECT status, commit_sha FROM finalize_requests WHERE request_id=?",
                (result.finalize.request_id,),
            ).fetchone()
            intent = connection.execute(
                "SELECT status FROM workflow_commit_intents WHERE intent_id=?",
                (result.finalize.request_id,),
            ).fetchone()
        self.assertEqual(("committed", result.finalize.commit_sha), tuple(finalize))
        self.assertEqual("reconciled", intent["status"])

    def test_foreign_staged_file_is_preserved_but_excluded_from_commit(self) -> None:
        service = self._service()
        paths = self._prepare_change_and_gates(service)
        foreign = self.repo / "foreign.txt"
        foreign.write_text("other session\n", encoding="utf-8")
        subprocess.run(["git", "add", "--", "foreign.txt"], cwd=self.repo, check=True)

        result = service.commit(
            session_id="session-a",
            run_id=self.run_id,
            milestone_key="M1",
            paths=paths,
            message="feat(workflow): complete M1 milestone",
            actor="session-a",
        )

        committed = subprocess.run(
            ["git", "show", "--pretty=", "--name-only", result.finalize.commit_sha],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
        staged = subprocess.run(
            ["git", "diff", "--cached", "--name-only"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
        self.assertNotIn("foreign.txt", committed)
        self.assertEqual(["foreign.txt"], staged)

    def test_gate_change_under_git_mutex_prevents_commit_and_preserves_ref(self) -> None:
        service = self._service()
        paths = self._prepare_change_and_gates(service)
        before = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=self.repo, check=True, capture_output=True, text=True
        ).stdout.strip()
        original = service.prepare_context
        calls = 0

        def changed_context(run_id: str, manifest: list[str] | tuple[str, ...]):
            nonlocal calls
            calls += 1
            value = original(run_id, manifest)
            if calls >= 2:
                return value.__class__(
                    value.topology_version_id,
                    "f" * 40,
                    value.baseline_epoch,
                    value.manifest_hash,
                    value.failure_revision,
                    value.plan_content_hash,
                )
            return value

        with mock.patch.object(service, "prepare_context", side_effect=changed_context):
            with self.assertRaises(CoordinatorError) as rejected:
                service.commit(
                    session_id="session-a",
                    run_id=self.run_id,
                    milestone_key="M1",
                    paths=paths,
                    message="feat(workflow): complete M1 milestone",
                    actor="session-a",
                )
        self.assertEqual("milestone_gate_stale_evidence", rejected.exception.code)
        after = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=self.repo, check=True, capture_output=True, text=True
        ).stdout.strip()
        self.assertEqual(before, after)

    def test_goal_closeout_requires_all_milestones_and_clean_owned_scope(self) -> None:
        service = self._service()
        with self.assertRaises(CoordinatorError) as incomplete:
            service.close_goal("session-a", self.run_id)
        self.assertEqual("workflow_goal_incomplete", incomplete.exception.code)

        paths = self._prepare_change_and_gates(service)
        service.commit(
            session_id="session-a",
            run_id=self.run_id,
            milestone_key="M1",
            paths=paths,
            message="feat(workflow): complete M1 milestone",
            actor="session-a",
        )
        result = service.close_goal("session-a", self.run_id)
        self.assertEqual("completed", result["session"]["status"])
        self.assertEqual(0, len(self.leases.owned_paths("session-a")))

    def test_plan_text_change_invalidates_active_topology_before_commit(self) -> None:
        service = self._service()
        paths = self._prepare_change_and_gates(service)
        plan = self.repo / "docs/plans/runtime/01-control.md"
        plan.write_text(plan.read_text(encoding="utf-8") + "\nnew requirement\n", encoding="utf-8")

        with self.assertRaises(CoordinatorError) as rejected:
            service.commit(
                session_id="session-a",
                run_id=self.run_id,
                milestone_key="M1",
                paths=paths,
                message="feat(workflow): complete M1 milestone",
                actor="session-a",
            )

        self.assertEqual("workflow_topology_plan_changed", rejected.exception.code)

    def test_managed_validation_result_creates_node_scoped_gate_evidence(self) -> None:
        service = self._service()
        paths = ["src/runtime.py"]
        target = self.repo / paths[0]
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("managed validation\n", encoding="utf-8")
        record = "docs/plans/runtime/01/2026-07-12-m1-validation.md"
        output = self.repo / record
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(
            "Plan: docs/plans/runtime/01-control.md\nMilestone: M1\nStatus: completed\n"
            f"Files: {json.dumps(paths)}\n\n## Scope delivered\n\nDone.\n\n"
            "## Fresh testing evidence\n\nPassed.\n\n## Review\n\nAccepted.\n",
            encoding="utf-8",
        )
        paths.append(record)
        self.assertTrue(self.leases.acquire("session-a", paths).acquired)
        self.baselines.attribute("session-a", paths)
        now = "2026-07-12T00:00:00Z"
        copy_source = self.repo.parent / "validation-source"
        for path in paths:
            copied = copy_source / path
            copied.parent.mkdir(parents=True, exist_ok=True)
            copied.write_bytes((self.repo / path).read_bytes())
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO validation_copies(
                       job_id, session_id, job_root, source_root, target_root,
                       head_commit, manifest_json, status, created_at
                   ) VALUES ('job-a', 'session-a', ?, ?, ?, ?, '[]',
                             'materialized', ?)""",
                (
                    str(copy_source.parent),
                    str(copy_source),
                    str(copy_source.parent / "target"),
                    subprocess.run(["git", "rev-parse", "HEAD"], cwd=self.repo, check=True, capture_output=True, text=True).stdout.strip(),
                    now,
                ),
            )
        service.bind_validation(
            session_id="session-a",
            run_id=self.run_id,
            milestone_key="M1",
            validation_run_id="validation-a",
            job_id="job-a",
            template="coordinator-actions",
            source_manifest_hash=service.prepare_context(self.run_id, paths).manifest_hash,
            actor="operator-a",
            action_id="action-a",
        )
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO validation_copy_runs(
                       run_id, job_id, session_id, command_json, exit_code,
                       stdout_text, stderr_text, started_at, completed_at
                   ) VALUES ('validation-a', 'job-a', 'session-a', '[\"test\"]',
                             0, 'ok', '', ?, ?)""",
                (now, now),
            )

        self.assertTrue(service.import_validation_result("validation-a"))
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM workflow_gate_evidence WHERE gate_kind='validation'"
            ).fetchone()
        self.assertEqual("accepted", row["decision"])
        self.assertEqual("action-a", row["action_id"])
        self.assertEqual(f"{self.run_id}:M1", row["node_id"])

    def test_validation_copy_mutation_after_binding_is_rejected(self) -> None:
        service = self._service()
        paths = ["src/runtime.py"]
        target = self.repo / paths[0]
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("bound content\n", encoding="utf-8")
        record = "docs/plans/runtime/01/2026-07-12-m1-mutation.md"
        output = self.repo / record
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(
            "Plan: docs/plans/runtime/01-control.md\nMilestone: M1\nStatus: completed\n"
            f"Files: {json.dumps(paths)}\n\n## Scope delivered\n\nDone.\n\n"
            "## Fresh testing evidence\n\nPassed.\n\n## Review\n\nAccepted.\n",
            encoding="utf-8",
        )
        paths.append(record)
        self.assertTrue(self.leases.acquire("session-a", paths).acquired)
        self.baselines.attribute("session-a", paths)
        source = self.repo.parent / "validation-mutated-source"
        for path in paths:
            copied = source / path
            copied.parent.mkdir(parents=True, exist_ok=True)
            copied.write_bytes((self.repo / path).read_bytes())
        now = "2026-07-12T00:00:00Z"
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO validation_copies(
                       job_id, session_id, job_root, source_root, target_root,
                       head_commit, manifest_json, status, created_at
                   ) VALUES ('job-mutated', 'session-a', ?, ?, ?, ?, '[]',
                             'materialized', ?)""",
                (
                    str(source.parent), str(source), str(source.parent / "target"),
                    subprocess.run(["git", "rev-parse", "HEAD"], cwd=self.repo, check=True, capture_output=True, text=True).stdout.strip(),
                    now,
                ),
            )
        service.bind_validation(
            session_id="session-a", run_id=self.run_id, milestone_key="M1",
            validation_run_id="validation-mutated", job_id="job-mutated",
            template="coordinator-actions",
            source_manifest_hash=service.prepare_context(self.run_id, paths).manifest_hash,
            actor="operator-a", action_id="action-mutated",
        )
        (source / paths[0]).write_text("tampered after binding\n", encoding="utf-8")
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO validation_copy_runs(
                       run_id, job_id, session_id, command_json, exit_code,
                       stdout_text, stderr_text, started_at, completed_at
                   ) VALUES ('validation-mutated', 'job-mutated', 'session-a', '["test"]',
                             0, 'ok', '', ?, ?)""",
                (now, now),
            )

        self.assertTrue(service.import_validation_result("validation-mutated"))
        with self.database.connect() as connection:
            terminal = connection.execute(
                "SELECT terminal_status, terminal_code FROM workflow_validation_bindings WHERE validation_run_id='validation-mutated'"
            ).fetchone()
        self.assertEqual(("rejected", "validation_copy_manifest_changed"), tuple(terminal))

    def test_controlled_review_and_refresh_derive_repository_gates(self) -> None:
        service = self._service()
        paths = ["src/runtime.py"]
        target = self.repo / paths[0]
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("reviewed change\n", encoding="utf-8")
        output = self.repo / "docs/plans/runtime/01/2026-07-12-m1-record.md"
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(
            "# M1 output evidence\n\n"
            "Plan: docs/plans/runtime/01-control.md\n"
            "Milestone: M1\n"
            "Status: completed\n\n"
            f"Files: {json.dumps(paths)}\n\n"
            "## Scope delivered\n\nDone.\n\n"
            "## Fresh testing evidence\n\nPassed.\n\n"
            "## Review\n\nAccepted.\n",
            encoding="utf-8",
        )
        paths.append("docs/plans/runtime/01/2026-07-12-m1-record.md")
        future = "src/future_m2.py"
        (self.repo / future).write_text("future milestone\n", encoding="utf-8")
        all_session_paths = [*paths, future]
        self.assertTrue(self.leases.acquire("session-a", all_session_paths).acquired)
        self.baselines.attribute("session-a", all_session_paths)

        review = service.submit_review(
            session_id="session-a",
            run_id=self.run_id,
            milestone_key="M1",
            reviewer_session_id="reviewer-b",
            reviewer_actor="reviewer-cli",
            critical_count=0,
            important_count=0,
            summary="architecture accepted",
            action_id="review-action",
        )
        refreshed = service.refresh_gates(
            session_id="session-a",
            run_id=self.run_id,
            actor="reviewer-b",
            action_id="refresh-action",
        )

        self.assertEqual("accepted", review["verdict"])
        self.assertTrue(refreshed["refreshed"])
        gates = refreshed["milestones"]["M1"]
        self.assertEqual("accepted", gates["review"])
        self.assertEqual("accepted", gates["plan_output"])
        self.assertEqual("accepted", gates["commit_manifest"])
        self.assertEqual(
            tuple(sorted(paths, key=str.casefold)), service.milestone_paths(self.run_id, "M1")
        )
        self.assertNotIn(future, service.milestone_paths(self.run_id, "M1"))

        output.write_text(
            output.read_text(encoding="utf-8").replace(
                json.dumps(["src/runtime.py"]), json.dumps(["src/runtime.py", future])
            ),
            encoding="utf-8",
        )
        with self.assertRaises(CoordinatorError) as immutable:
            service.bind_manifest(
                session_id="session-a", run_id=self.run_id, milestone_key="M1",
                actor="session-a", action_id="rebind",
            )
        self.assertEqual("milestone_manifest_already_bound", immutable.exception.code)


if __name__ == "__main__":
    unittest.main()
