from __future__ import annotations

import subprocess
import tempfile
import unittest
import json
from types import SimpleNamespace
from unittest import mock
from pathlib import Path

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.benchmark_validation_grants import (
    BenchmarkValidationGrantService,
)
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.failures import FailureGraphService
from tools.session_coordinator.git_finalize import GitFinalizeService
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import SessionStatus, WorkflowNodeState
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.notifications import WeComNotificationService
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.failure_fixture import FailureGraphFixture, FixturePlan
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.workflows.gates import GateEvidenceStore, MilestoneGateEvaluator
from tools.session_coordinator.workflows.milestones import MilestoneWorkflowService
from tools.session_coordinator.workflows.plan_import import TopologyImporter
from tools.session_coordinator.workflows.store import WorkflowStore
from tools.session_coordinator.workflows.topology import TopologyParser


class WorkflowCommitTests(unittest.TestCase):
    def setUp(self) -> None:
        self.notification_messages: list[str] = []
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.repo = init_repo(root / "repo")
        plan = self.repo / "docs/plans/runtime/01-control.md"
        plan.parent.mkdir(parents=True)
        plan.write_text(
            "```zircon-workflow\n"
            '{"schema":1,"workflow_id":"runtime-control","goal":"Runtime",'
            '"milestones":[{"id":"M1","title":"Base","depends_on":[]},'
            '{"id":"M2","title":"Feature","depends_on":["M1"]}]}\n'
            "```\n\n## Milestone M1: Base\n\n"
            "- [ ] **M1.1 Add storage.** details\n\n"
            "## Milestone M2: Feature\n\n"
            "- [ ] **M2.1 Add feature storage.** details\n"
            "- [ ] **M2.2 Add feature projection.** details\n",
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

    def _service(
        self, returncode: int = 0, *, failures=None
    ) -> MilestoneWorkflowService:
        def runner(command: list[str]) -> subprocess.CompletedProcess[str]:
            self.notification_messages.append(command[command.index("-Message") + 1])
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
            failures=failures,
        )

    def _append_future_milestone(self) -> None:
        plan = self.repo / "docs/plans/runtime/01-control.md"
        old_topology_hash = TopologyParser(self.repo).parse(
            "docs/plans/runtime/01-control.md"
        ).topology_hash
        plan_text = plan.read_text(encoding="utf-8")
        prefix, fenced = plan_text.split("```zircon-workflow\n", 1)
        payload_text, suffix = fenced.split("\n```", 1)
        payload = json.loads(payload_text)
        payload["milestones"].append(
            {"id": "M3", "title": "Future", "depends_on": ["M2"]}
        )
        plan.write_text(
            prefix
            + "```zircon-workflow\n"
            + json.dumps(payload, ensure_ascii=False, separators=(",", ":"))
            + "\n```"
            + suffix,
            encoding="utf-8",
        )
        new_topology_hash = TopologyParser(self.repo).parse(
            "docs/plans/runtime/01-control.md"
        ).topology_hash
        self.assertNotEqual(old_topology_hash, new_topology_hash)
        imported = TopologyImporter(self.database, self.repo).import_plan(
            "session-a",
            "docs/plans/runtime/01-control.md",
            activate_candidate=True,
        )
        self.run_id = imported.run_id
        self.topology_version_id = imported.topology_version_id

    def test_failure_node_keys_are_exact_for_slice_and_aggregate_for_parent(self) -> None:
        service = self._service()

        self.assertEqual(("M1.1",), service._failure_node_keys(self.run_id, "M1.1"))
        self.assertEqual(("M1", "M1.1"), service._failure_node_keys(self.run_id, "M1"))
        self.assertEqual(
            ("M2", "M2.1", "M2.2"),
            service._failure_node_keys(self.run_id, "M2"),
        )

    def test_future_fixer_failure_deferral_excludes_source_milestone_but_not_successor(
        self,
    ) -> None:
        self._append_future_milestone()
        fixture = FailureGraphFixture(self.repo)
        origin = fixture.add_plan("docs/plans/performance/01-origin.md")
        fixing = FixturePlan(
            self.repo / "docs/plans/runtime/01-control.md",
            self.repo / "docs/plans/runtime/01",
        )
        fixing.child.mkdir(parents=True, exist_ok=True)
        artifact = fixture.add_handoff(
            origin,
            fixing,
            "future-m3-compile-analysis",
        )
        failures = FailureGraphService(self.database, self.repo)
        audit = failures.import_repository()
        lifecycle_key = next(
            node.lifecycle_key
            for node in audit.nodes
            if node.artifact_path == artifact.relative_to(self.repo).as_posix()
        )
        service = self._service(failures=failures)

        deferral = service.defer_failure(
            session_id="session-a",
            source_milestone_key="M2",
            target_milestone_key="M3",
            failure_lifecycle_key=lifecycle_key,
            actor="session-a",
            action_id="defer-m3-failure",
        )

        self.assertEqual("M2", deferral["sourceMilestoneId"])
        self.assertEqual("M3", deferral["targetMilestoneId"])
        self.assertEqual(
            [],
            service.open_failures_for_milestone(
                run_id=self.run_id,
                milestone_key="M2",
                paths=(),
            ),
        )
        self.assertEqual(
            [artifact.relative_to(self.repo).as_posix()],
            [
                node.artifact_path
                for node in service.open_failures_for_milestone(
                    run_id=self.run_id,
                    milestone_key="M3",
                    paths=(),
                )
            ],
        )

    def test_failure_deferral_rejects_reverse_or_foreign_owner_and_stales_on_topology_change(
        self,
    ) -> None:
        self._append_future_milestone()
        fixture = FailureGraphFixture(self.repo)
        origin = fixture.add_plan("docs/plans/performance/01-origin.md")
        fixing = FixturePlan(
            self.repo / "docs/plans/runtime/01-control.md",
            self.repo / "docs/plans/runtime/01",
        )
        fixing.child.mkdir(parents=True, exist_ok=True)
        artifact = fixture.add_handoff(origin, fixing, "future-topology-bound-failure")
        failures = FailureGraphService(self.database, self.repo)
        audit = failures.import_repository()
        lifecycle_key = next(
            node.lifecycle_key
            for node in audit.nodes
            if node.artifact_path == artifact.relative_to(self.repo).as_posix()
        )
        service = self._service(failures=failures)

        with self.assertRaises(CoordinatorError) as reverse:
            service.defer_failure(
                session_id="session-a",
                source_milestone_key="M3",
                target_milestone_key="M2",
                failure_lifecycle_key=lifecycle_key,
                actor="session-a",
                action_id="reverse",
            )
        self.assertEqual("milestone_failure_deferral_target_invalid", reverse.exception.code)

        with self.assertRaises(CoordinatorError) as foreign:
            service.defer_failure(
                session_id="reviewer-b",
                source_milestone_key="M2",
                target_milestone_key="M3",
                failure_lifecycle_key=lifecycle_key,
                actor="reviewer-b",
                action_id="foreign",
            )
        self.assertEqual("workflow_run_owner_mismatch", foreign.exception.code)

        service.defer_failure(
            session_id="session-a",
            source_milestone_key="M2",
            target_milestone_key="M3",
            failure_lifecycle_key=lifecycle_key,
            actor="session-a",
            action_id="valid",
        )
        plan = self.repo / "docs/plans/runtime/01-control.md"
        old_topology_hash = TopologyParser(self.repo).parse(
            "docs/plans/runtime/01-control.md"
        ).topology_hash
        plan_text = plan.read_text(encoding="utf-8")
        prefix, fenced = plan_text.split("```zircon-workflow\n", 1)
        payload_text, suffix = fenced.split("\n```", 1)
        payload = json.loads(payload_text)
        milestone = next(item for item in payload["milestones"] if item["id"] == "M3")
        milestone["depends_on"] = ["M1"]
        plan.write_text(
            prefix
            + "```zircon-workflow\n"
            + json.dumps(payload, ensure_ascii=False, separators=(",", ":"))
            + "\n```"
            + suffix,
            encoding="utf-8",
        )
        new_topology_hash = TopologyParser(self.repo).parse(
            "docs/plans/runtime/01-control.md"
        ).topology_hash
        self.assertNotEqual(old_topology_hash, new_topology_hash)
        imported = TopologyImporter(self.database, self.repo).import_plan(
            "session-a", "docs/plans/runtime/01-control.md"
        )
        self.run_id = imported.run_id

        self.assertEqual(
            [artifact.relative_to(self.repo).as_posix()],
            [
                node.artifact_path
                for node in service.open_failures_for_milestone(
                    run_id=self.run_id,
                    milestone_key="M2",
                    paths=(),
                )
            ],
        )

    def test_prepare_context_rejects_invalid_failure_scope(self) -> None:
        service = self._service()

        with self.assertRaises(CoordinatorError) as rejected:
            service.prepare_context(
                self.run_id,
                ["src/runtime.py"],
                failure_workflow_node_keys=("M1.0",),
            )

        self.assertEqual(
            "milestone_failure_scope_invalid",
            rejected.exception.code,
        )

    def test_bound_manifest_failure_scope_is_reused_by_context_and_gate_refresh(self) -> None:
        failures = mock.Mock()
        failures.open_for_manifest.return_value = []
        service = self._service(failures=failures)

        paths = self._prepare_change_and_gates(service)

        expected = (
            "docs/plans/runtime/01-control.md",
            ("M1", "M1.1"),
            tuple(sorted(paths, key=str.casefold)),
        )
        self.assertGreaterEqual(failures.open_for_manifest.call_count, 2)
        self.assertIn(mock.call(*expected), failures.open_for_manifest.call_args_list)

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
        context = service.prepare_context(
            self.run_id,
            paths,
            failure_workflow_node_keys=service._failure_node_keys(
                self.run_id, "M1"
            ),
        )
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
            summary="add managed runtime validation coverage",
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
        subject = subprocess.run(
            ["git", "show", "-s", "--format=%s", result.finalize.commit_sha],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        self.assertEqual("feat(runtime): add managed runtime validation coverage", subject)
        self.assertEqual(1, len(self.notification_messages))
        self.assertTrue(
            self.notification_messages[0].startswith("核心内容摘要：【runtime】")
        )
        self.assertIn(
            "M1 · Base：add managed runtime validation coverage",
            self.notification_messages[0],
        )
        self.assertIn(
            f"\n提交的commit内容：{result.finalize.commit_sha} {subject}",
            self.notification_messages[0],
        )
        self.assertTrue(result.gate.allowed)
        self.assertEqual("succeeded", result.notification.status)
        self.assertEqual(SessionStatus.ACTIVE, self.sessions.get("session-a").status)

    def test_commit_reuses_one_immutable_failure_scope(self) -> None:
        service = self._service()
        paths = self._prepare_change_and_gates(service)

        with (
            mock.patch.object(
                service,
                "_failure_node_keys",
                wraps=service._failure_node_keys,
            ) as resolve_scope,
            mock.patch.object(
                self.finalize,
                "commit_milestone",
                wraps=self.finalize.commit_milestone,
            ) as finalize,
        ):
            service.commit(
                session_id="session-a",
                run_id=self.run_id,
                milestone_key="M1",
                paths=paths,
                summary="reuse one failure scope through finalization",
                actor="session-a",
            )

        resolve_scope.assert_called_once_with(self.run_id, "M1")
        self.assertEqual(
            ("M1", "M1.1"),
            finalize.call_args.kwargs["failure_workflow_node_keys"],
        )

    def test_manifest_derivation_uses_current_attributed_record_when_history_exists(self) -> None:
        service = self._service()
        source = "src/runtime.py"
        (self.repo / source).parent.mkdir(parents=True, exist_ok=True)
        (self.repo / source).write_text("current source attestation\n", encoding="utf-8")
        historical = "docs/plans/runtime/01/2026-07-14-m1-historical-acceptance.md"
        current = "docs/plans/runtime/01/2026-07-15-m1-current-source-attestation.md"
        for record, files in ((historical, ["src/historical.py"]), (current, [source])):
            output = self.repo / record
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_text(
                "# M1 evidence\n\n"
                "Plan: docs/plans/runtime/01-control.md\n"
                "Milestone: M1\n"
                "Status: completed\n"
                f"Files: {json.dumps(files)}\n",
                encoding="utf-8",
            )
        current_paths = [source, current]
        self.assertTrue(self.leases.acquire("session-a", current_paths).acquired)
        self.baselines.attribute("session-a", current_paths)

        self.assertEqual(
            tuple(sorted(current_paths, key=str.casefold)),
            service._derive_milestone_paths("session-a", self.run_id, "M1"),
        )

    def test_bind_manifest_allows_directory_leases_to_cover_child_files(self) -> None:
        service = self._service()
        paths = ["src/runtime.py", "tests/test_runtime.py"]
        for path in paths:
            target = self.repo / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(f"content for {path}\n", encoding="utf-8")
        record = "docs/plans/runtime/01/2026-07-17-m1-directory-lease.md"
        output = self.repo / record
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(
            "# M1 directory lease output\n\n"
            "Plan: docs/plans/runtime/01-control.md\n"
            "Milestone: M1\n"
            "Status: completed\n"
            f"Files: {json.dumps(paths)}\n\n"
            "## Scope delivered\n\nDone.\n\n"
            "## Fresh testing evidence\n\nPassed.\n\n"
            "## Review\n\nAccepted.\n",
            encoding="utf-8",
        )
        self.assertTrue(
            self.leases.acquire("session-a", ["src", "tests", record]).acquired
        )
        self.baselines.attribute("session-a", [*paths, record])

        manifest_id = service.bind_manifest(
            session_id="session-a",
            run_id=self.run_id,
            milestone_key="M1",
            actor="session-a",
            action_id="directory-lease-bind",
        )

        self.assertTrue(manifest_id)

    def test_slice_commit_succeeds_without_accepting_parent_milestone(self) -> None:
        service = self._service()
        with self.database.connect() as connection:
            slice_node = connection.execute(
                "SELECT node_id FROM workflow_nodes WHERE run_id=? AND node_key='M2.1'",
                (self.run_id,),
            ).fetchone()[0]
            attempts_before = int(
                connection.execute(
                    "SELECT COUNT(*) FROM workflow_attempts WHERE node_id=?",
                    (slice_node,),
                ).fetchone()[0]
            )

        paths = ["src/storage_slice.py"]
        target = self.repo / paths[0]
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("storage slice\n", encoding="utf-8")
        record = "docs/plans/runtime/01/2026-07-12-m1-1-storage-slice.md"
        output = self.repo / record
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(
            "# M2.1 output\n\nPlan: docs/plans/runtime/01-control.md\n"
            "Milestone: M2.1\nStatus: completed\n"
            f"Files: {json.dumps(paths)}\n\n"
            "## Scope delivered\n\nDone.\n\n"
            "## Fresh testing evidence\n\nPassed.\n\n"
            "## Review\n\nAccepted.\n",
            encoding="utf-8",
        )
        paths.append(record)
        self.assertTrue(self.leases.acquire("session-a", paths).acquired)
        self.baselines.attribute("session-a", paths)
        subprocess.run(["git", "add", "--", *paths], cwd=self.repo, check=True)

        service.bind_manifest(
            session_id="session-a",
            run_id=self.run_id,
            milestone_key="M2.1",
            actor="session-a",
            action_id="bind-slice",
        )
        context = service.prepare_context(
            self.run_id,
            paths,
            failure_workflow_node_keys=service._failure_node_keys(
                self.run_id, "M2.1"
            ),
        )
        evaluator = MilestoneGateEvaluator(self.database)
        fingerprint = evaluator.input_fingerprint(self.run_id, "M2.1", context)
        evidence = GateEvidenceStore(self.database)
        evidence.record_review(
            run_id=self.run_id,
            topology_version_id=self.topology_version_id,
            reviewer="reviewer-b",
            executor="session-a",
            critical_count=0,
            important_count=0,
            summary="slice clean",
            input_fingerprint=fingerprint,
            node_id=slice_node,
        )
        evidence.record_gate(
            run_id=self.run_id,
            topology_version_id=self.topology_version_id,
            gate_kind="validation",
            decision="accepted",
            decision_code="managed_validation_succeeded",
            input_fingerprint=fingerprint,
            actor="reviewer-b",
            node_id=slice_node,
        )
        refreshed = service.refresh_gates(
            session_id="session-a",
            run_id=self.run_id,
            actor="reviewer-b",
            action_id="refresh-slice",
        )
        self.assertEqual(
            {
                "commit_manifest": "accepted",
                "failure_audit": "accepted",
                "plan_output": "accepted",
                "review": "accepted",
            },
            refreshed["milestones"]["M2.1"],
        )

        result = service.commit(
            session_id="session-a",
            run_id=self.run_id,
            milestone_key="M2.1",
            paths=paths,
            summary="add managed storage slice",
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
        with self.database.connect() as connection:
            states = {
                row["node_key"]: row["state"]
                for row in connection.execute(
                    "SELECT node_key, state FROM workflow_nodes WHERE run_id=? AND node_key IN ('M1', 'M2', 'M2.1', 'M2.2')",
                    (self.run_id,),
                )
            }
            attempts_after = int(
                connection.execute(
                    "SELECT COUNT(*) FROM workflow_attempts WHERE node_id=?",
                    (slice_node,),
                ).fetchone()[0]
            )
        self.assertEqual("succeeded", states["M2.1"])
        self.assertEqual("pending", states["M2.2"])
        self.assertEqual("pending", states["M2"])
        parent_gate = service.gates.evaluate(
            self.run_id,
            "M2",
            service.prepare_context(
                self.run_id,
                paths,
                failure_workflow_node_keys=service._failure_node_keys(
                    self.run_id, "M2"
                ),
            ),
        )
        self.assertFalse(parent_gate.allowed)
        self.assertEqual("milestone_gate_nodes_incomplete", parent_gate.code)
        with self.database.connect() as connection:
            blocking_keys = {
                row["node_key"]
                for row in connection.execute(
                    "SELECT node_key FROM workflow_nodes WHERE node_id IN (?, ?)",
                    parent_gate.blocking_node_ids,
                )
            }
        self.assertEqual({"M1", "M2.2"}, blocking_keys)
        self.assertEqual(attempts_before + 1, attempts_after)

    def test_commit_subject_rejects_generic_slice_completion_summary(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            MilestoneWorkflowService._commit_subject(
                "runtime",
                ["src/runtime.py"],
                "complete M2.1 slice",
            )

        self.assertEqual("milestone_commit_summary_invalid", rejected.exception.code)

    def test_commit_subject_preserves_explicit_conventional_subject(self) -> None:
        self.assertEqual(
            "test(shader): verify material redirect persistence contract",
            MilestoneWorkflowService._commit_subject(
                "shader",
                ["zircon_runtime/tests/material_redirect.rs"],
                "test(shader): verify material redirect persistence contract",
            ),
        )

    def test_notification_failure_does_not_rollback_commit(self) -> None:
        self.notification_messages = []
        service = self._service(returncode=1)
        paths = self._prepare_change_and_gates(service)

        result = service.commit(
            session_id="session-a",
            run_id=self.run_id,
            milestone_key="M1",
            paths=paths,
            summary="record notification failure without rolling back commit",
            actor="session-a",
        )

        head = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=self.repo, check=True, capture_output=True, text=True
        ).stdout.strip()
        self.assertEqual(result.finalize.commit_sha, head)
        self.assertEqual("failed", result.notification.status)

    def test_notification_preparation_failure_is_recorded_after_commit(self) -> None:
        service = self._service()
        paths = self._prepare_change_and_gates(service)

        with mock.patch.object(
            service.notifications,
            "format_message",
            side_effect=CoordinatorError(
                "notification_content_invalid", "injected format failure"
            ),
        ):
            result = service.commit(
                session_id="session-a",
                run_id=self.run_id,
                milestone_key="M1",
                paths=paths,
                summary="record post-commit notification preparation failure",
                actor="session-a",
            )

        head = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        self.assertEqual(result.finalize.commit_sha, head)
        self.assertEqual("unknown", result.notification.status)
        self.assertIn("post-commit", result.notification.sanitized_error or "")
        with self.database.connect() as connection:
            attempt = connection.execute(
                "SELECT status, sanitized_error FROM notification_attempts WHERE commit_sha=?",
                (result.finalize.commit_sha,),
            ).fetchone()
        self.assertEqual("unknown", attempt["status"])
        self.assertIn("post-commit", attempt["sanitized_error"])

    def test_commit_rejects_a_generic_milestone_completion_summary(self) -> None:
        service = self._service()
        paths = self._prepare_change_and_gates(service)

        with self.assertRaises(CoordinatorError) as rejected:
            service.commit(
                session_id="session-a",
                run_id=self.run_id,
                milestone_key="M1",
                paths=paths,
                summary="complete M1 milestone",
                actor="session-a",
            )

        self.assertEqual("milestone_commit_summary_invalid", rejected.exception.code)

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
                summary="reconcile the post-CAS baseline update",
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
            summary="preserve foreign staged files during scoped commit",
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

        def changed_context(
            run_id: str,
            manifest: list[str] | tuple[str, ...],
            *,
            failure_workflow_node_keys: tuple[str, ...],
        ):
            nonlocal calls
            calls += 1
            value = original(
                run_id,
                manifest,
                failure_workflow_node_keys=failure_workflow_node_keys,
            )
            if calls >= 2:
                return value.__class__(
                    value.topology_version_id,
                    "f" * 40,
                    value.baseline_epoch,
                    value.manifest_hash,
                    value.failure_revision,
                    value.plan_topology_hash,
                )
            return value

        with mock.patch.object(service, "prepare_context", side_effect=changed_context):
            with self.assertRaises(CoordinatorError) as rejected:
                service.commit(
                    session_id="session-a",
                    run_id=self.run_id,
                    milestone_key="M1",
                    paths=paths,
                    summary="reject stale gate evidence under the Git mutex",
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
            summary="close the accepted runtime workflow goal",
            actor="session-a",
        )
        nodes = {item.node_key: item for item in WorkflowStore(self.database).nodes(self.run_id)}
        WorkflowStore(self.database).append_attempt(
            nodes["M2"].node_id,
            WorkflowNodeState.SUCCEEDED,
            {"exit": 0},
        )
        plan = self.repo / "docs/plans/runtime/01-control.md"
        plan.write_text(
            plan.read_text(encoding="utf-8") + "\ncloseout status update\n",
            encoding="utf-8",
        )
        result = service.close_goal("session-a", self.run_id)
        self.assertEqual("completed", result["session"]["status"])
        self.assertEqual(0, len(self.leases.owned_paths("session-a")))

    def test_goal_closeout_ignores_terminal_failed_commit_intents(self) -> None:
        service = self._service()
        paths = self._prepare_change_and_gates(service)
        service.commit(
            session_id="session-a",
            run_id=self.run_id,
            milestone_key="M1",
            paths=paths,
            summary="preserve failed commit intent audit history",
            actor="session-a",
        )
        nodes = {item.node_key: item for item in WorkflowStore(self.database).nodes(self.run_id)}
        WorkflowStore(self.database).append_attempt(
            nodes["M2"].node_id,
            WorkflowNodeState.SUCCEEDED,
            {"exit": 0},
        )
        with self.database.connect() as connection:
            node_id = connection.execute(
                "SELECT node_id FROM workflow_nodes WHERE run_id=? AND node_key='M1'",
                (self.run_id,),
            ).fetchone()[0]
        now = "2026-07-15T00:00:00Z"
        with self.database.transaction() as connection:
            for intent_id in ("failed-intent-a", "failed-intent-b"):
                connection.execute(
                    """INSERT INTO workflow_commit_intents(
                           intent_id, run_id, topology_version_id, node_id,
                           session_id, action_id, actor, gate_fingerprint,
                           paths_json, message, status, commit_sha, error_text,
                           created_at, updated_at
                       ) VALUES (?, ?, ?, ?, 'session-a', NULL, 'session-a',
                                 'failed-gate', '[]', 'historical failed finalize',
                                 'failed', NULL, 'finalize failed before ref update', ?, ?)""",
                    (intent_id, self.run_id, self.topology_version_id, node_id, now, now),
                )

        result = service.close_goal("session-a", self.run_id)

        self.assertEqual("completed", result["session"]["status"])
        with self.database.connect() as connection:
            failed_count = connection.execute(
                """SELECT COUNT(*) FROM workflow_commit_intents
                   WHERE run_id=? AND status='failed' AND commit_sha IS NULL""",
                (self.run_id,),
            ).fetchone()[0]
        self.assertEqual(2, failed_count)

    def test_goal_closeout_keeps_prepared_commit_intent_pending(self) -> None:
        service = self._service()
        paths = self._prepare_change_and_gates(service)
        service.commit(
            session_id="session-a",
            run_id=self.run_id,
            milestone_key="M1",
            paths=paths,
            summary="retain pending commit reconciliation guard",
            actor="session-a",
        )
        with self.database.connect() as connection:
            node_id = connection.execute(
                "SELECT node_id FROM workflow_nodes WHERE run_id=? AND node_key='M1'",
                (self.run_id,),
            ).fetchone()[0]
        now = "2026-07-15T00:00:00Z"
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO workflow_commit_intents(
                       intent_id, run_id, topology_version_id, node_id,
                       session_id, action_id, actor, gate_fingerprint,
                       paths_json, message, status, created_at, updated_at
                   ) VALUES ('prepared-intent', ?, ?, ?, 'session-a', NULL,
                             'session-a', 'pending-gate', '[]', 'pending finalize',
                             'prepared', ?, ?)""",
                (self.run_id, self.topology_version_id, node_id, now, now),
            )

        with self.assertRaises(CoordinatorError) as pending:
            service.close_goal("session-a", self.run_id)

        self.assertEqual("workflow_goal_commit_reconciliation_pending", pending.exception.code)
        self.assertEqual(1, pending.exception.details["count"])

    def test_plan_text_change_preserves_active_topology_gate_before_commit(self) -> None:
        service = self._service()
        paths = self._prepare_change_and_gates(service)
        plan = self.repo / "docs/plans/runtime/01-control.md"
        plan.write_text(plan.read_text(encoding="utf-8") + "\nnew requirement\n", encoding="utf-8")

        result = service.commit(
            session_id="session-a",
            run_id=self.run_id,
            milestone_key="M1",
            paths=paths,
            summary="accept unchanged semantic topology before commit",
            actor="session-a",
        )

        self.assertTrue(result.finalize.commit_sha)

    def test_plan_topology_change_invalidates_active_topology_before_commit(self) -> None:
        service = self._service()
        paths = self._prepare_change_and_gates(service)
        plan = self.repo / "docs/plans/runtime/01-control.md"
        plan.write_text(
            plan.read_text(encoding="utf-8").replace(
                '"id":"M1","title":"Base"',
                '"id":"M1","title":"Changed Base"',
            ),
            encoding="utf-8",
        )

        with self.assertRaises(CoordinatorError) as rejected:
            service.commit(
                session_id="session-a",
                run_id=self.run_id,
                milestone_key="M1",
                paths=paths,
                summary="reject changed semantic topology before commit",
                actor="session-a",
            )

        self.assertEqual("workflow_topology_plan_changed", rejected.exception.code)

    def test_refreshing_unchanged_topology_retains_active_version_and_gate_identity(self) -> None:
        plan = self.repo / "docs/plans/runtime/01-control.md"
        plan.write_text(plan.read_text(encoding="utf-8") + "\nstatus update\n", encoding="utf-8")

        imported = TopologyImporter(self.database, self.repo).import_plan(
            "session-a",
            "docs/plans/runtime/01-control.md",
            activate_candidate=True,
        )
        current = TopologyParser(self.repo).parse("docs/plans/runtime/01-control.md")
        source = self.repo / "src/runtime.py"
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_text("current source\n", encoding="utf-8")

        context = self._service().prepare_context(
            self.run_id,
            ["src/runtime.py"],
            failure_workflow_node_keys=("M1", "M1.1"),
        )
        with self.database.connect() as connection:
            version_count = connection.execute(
                "SELECT COUNT(*) FROM workflow_topology_versions WHERE run_id=?",
                (self.run_id,),
            ).fetchone()[0]

        self.assertEqual(current.content_hash, imported.content_hash)
        self.assertEqual(self.topology_version_id, imported.topology_version_id)
        self.assertEqual(imported.topology_version_id, context.topology_version_id)
        self.assertEqual(current.topology_hash, context.plan_topology_hash)
        self.assertEqual(1, version_count)

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
            source_manifest_hash=service.prepare_context(
                self.run_id,
                paths,
                failure_workflow_node_keys=service._failure_node_keys(
                    self.run_id, "M1"
                ),
            ).manifest_hash,
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

    def test_benchmark_binding_persists_distinct_scoped_and_full_copy_manifests(self) -> None:
        service = self._service()
        paths = ["src/runtime.py"]
        target = self.repo / paths[0]
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("milestone content\n", encoding="utf-8")
        self.assertTrue(self.leases.acquire("session-a", paths).acquired)
        self.baselines.attribute("session-a", paths)
        source = self.repo.parent / "cargo-closure-source"
        copied = source / paths[0]
        copied.parent.mkdir(parents=True, exist_ok=True)
        copied.write_bytes(target.read_bytes())
        (source / "Cargo.lock").write_text("closure-only\n", encoding="utf-8")
        scoped_hash = service.prepare_context(
            self.run_id,
            paths,
            failure_workflow_node_keys=service._failure_node_keys(self.run_id, "M1"),
        ).manifest_hash
        full_hash = "f" * 64
        self.assertNotEqual(scoped_hash, full_hash)
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO workflow_milestone_manifests(
                       manifest_id, run_id, topology_version_id, node_id,
                       session_id, paths_json, manifest_hash, actor, action_id,
                       created_at
                   ) VALUES ('benchmark-manifest', ?, ?, ?, 'session-a', ?, ?,
                             'operator-a', 'action-a',
                             '2026-08-11T00:00:00+00:00')""",
                (
                    self.run_id,
                    self.topology_version_id,
                    f"{self.run_id}:M1",
                    json.dumps(paths),
                    scoped_hash,
                ),
            )
            connection.execute(
                """INSERT INTO validation_copies(
                       job_id, session_id, job_root, source_root, target_root,
                       head_commit, manifest_json, status, created_at,
                       input_manifest_hash
                   ) VALUES ('benchmark-copy', 'session-a', ?, ?, ?, 'head', ?,
                             'materialized', '2026-08-11T00:00:00+00:00', ?)""",
                (
                    str(source.parent),
                    str(source),
                    str(source.parent / "target"),
                    json.dumps([*paths, "Cargo.lock"]),
                    full_hash,
                ),
            )

        service.bind_validation(
            session_id="session-a",
            run_id=self.run_id,
            milestone_key="M1",
            validation_run_id="benchmark-validation",
            job_id="benchmark-copy",
            template="native-plugin-benchmark",
            source_manifest_hash=scoped_hash,
            copy_input_manifest_hash=full_hash,
            benchmark_name="native_host_context_lookup_1_thread_benchmark",
            cargo_profile="release",
            benchmark_grant_id="grant-a",
            actor="operator-a",
            action_id="action-a",
        )
        service.record_validation_process_identity(
            "benchmark-validation",
            root_pid=4242,
            process_creation_time="111222",
        )

        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM workflow_validation_bindings WHERE validation_run_id=?",
                ("benchmark-validation",),
            ).fetchone()
        self.assertEqual(scoped_hash, row["source_manifest_hash"])
        self.assertEqual(full_hash, row["copy_input_manifest_hash"])
        self.assertEqual(4242, row["root_pid"])
        self.assertEqual("release", row["cargo_profile"])

        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE validation_copies SET input_manifest_hash=? WHERE job_id='benchmark-copy'",
                ("e" * 64,),
            )
            connection.execute(
                """INSERT INTO validation_copy_runs(
                       run_id, job_id, session_id, command_json, exit_code,
                       stdout_text, stderr_text, started_at, completed_at
                   ) VALUES ('benchmark-validation', 'benchmark-copy', 'session-a',
                             '["cargo","test"]', 0, 'ok', '',
                             '2026-08-11T00:00:00+00:00',
                             '2026-08-11T00:01:00+00:00')"""
            )
        self.assertTrue(service.import_validation_result("benchmark-validation"))
        with self.database.connect() as connection:
            rejected = connection.execute(
                """SELECT terminal_status, terminal_code
                   FROM workflow_validation_bindings
                   WHERE validation_run_id='benchmark-validation'"""
            ).fetchone()
        self.assertEqual("rejected", rejected["terminal_status"])
        self.assertEqual(
            "validation_copy_input_manifest_changed", rejected["terminal_code"]
        )

    def test_restart_rejects_consumed_benchmark_without_terminal_evidence(self) -> None:
        service = self._service()
        paths = ["src/runtime.py"]
        target = self.repo / paths[0]
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("milestone content\n", encoding="utf-8")
        self.assertTrue(self.leases.acquire("session-a", paths).acquired)
        self.baselines.attribute("session-a", paths)
        scoped_hash = service.prepare_context(
            self.run_id,
            paths,
            failure_workflow_node_keys=service._failure_node_keys(self.run_id, "M1"),
        ).manifest_hash
        full_hash = "f" * 64
        copy_root = self.repo.parent / "interrupted-benchmark"
        source_root = copy_root / "source"
        copied = source_root / paths[0]
        copied.parent.mkdir(parents=True)
        copied.write_bytes(target.read_bytes())
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO workflow_milestone_manifests(
                       manifest_id, run_id, topology_version_id, node_id,
                       session_id, paths_json, manifest_hash, actor, action_id,
                       created_at
                   ) VALUES ('restart-benchmark-manifest', ?, ?, ?, 'session-a',
                             ?, ?, 'operator-a', 'action-a',
                             '2026-08-11T00:00:00+00:00')""",
                (
                    self.run_id,
                    self.topology_version_id,
                    f"{self.run_id}:M1",
                    json.dumps(paths),
                    scoped_hash,
                ),
            )
            connection.execute(
                """INSERT INTO validation_copies(
                       job_id, session_id, job_root, source_root, target_root,
                       head_commit, manifest_json, status, created_at,
                       input_manifest_hash, run_pid
                   ) VALUES ('restart-benchmark-copy', 'session-a', ?, ?, ?,
                             'head', ?, 'running',
                             '2026-08-11T00:00:00+00:00', ?, 4242)""",
                (
                    str(copy_root),
                    str(source_root),
                    str(copy_root / "target"),
                    json.dumps(paths),
                    full_hash,
                ),
            )

        service.bind_validation(
            session_id="session-a",
            run_id=self.run_id,
            milestone_key="M1",
            validation_run_id="restart-benchmark-validation",
            job_id="restart-benchmark-copy",
            template="native-plugin-benchmark",
            source_manifest_hash=scoped_hash,
            copy_input_manifest_hash=full_hash,
            benchmark_name="native_host_context_lookup_1_thread_benchmark",
            cargo_profile="release",
            benchmark_grant_id="restart-grant",
            actor="operator-a",
            action_id="action-a",
        )
        service.record_validation_process_identity(
            "restart-benchmark-validation",
            root_pid=4242,
            process_creation_time="111222",
        )
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO benchmark_validation_grants(
                       grant_id, job_id, source_session_id, target_session_id,
                       run_id, milestone_id, input_manifest_hash,
                       scoped_manifest_hash, benchmark_name, cargo_profile,
                       command_json, status, issued_at, acquired_at, consumed_at,
                       validation_run_id, root_pid, root_process_creation_time
                   ) VALUES ('restart-grant', 'restart-benchmark-copy',
                             'session-a', 'session-a', ?, 'M1', ?, ?,
                             'native_host_context_lookup_1_thread_benchmark',
                             'release', '["cargo","test"]', 'consumed',
                             '2026-08-11T00:00:00+00:00',
                             '2026-08-11T00:00:01+00:00',
                             '2026-08-11T00:00:02+00:00',
                             'restart-benchmark-validation', 4242, '111222')""",
                (self.run_id, full_hash, scoped_hash),
            )

        recovered = BenchmarkValidationGrantService(
            self.database
        ).reconcile_interrupted_consumed(
            service.reject_validation_launch,
            terminate_interrupted=mock.Mock(),
        )

        self.assertEqual(("restart-benchmark-validation",), recovered)
        with self.database.connect() as connection:
            binding = connection.execute(
                """SELECT terminal_status, terminal_code, imported_at
                   FROM workflow_validation_bindings
                   WHERE validation_run_id='restart-benchmark-validation'"""
            ).fetchone()
            gate = connection.execute(
                """SELECT decision, decision_code
                   FROM workflow_gate_evidence
                   WHERE source_revision='restart-benchmark-validation'
                     AND gate_kind='validation'"""
            ).fetchone()
        self.assertEqual("rejected", binding["terminal_status"])
        self.assertEqual(
            "benchmark_validation_collector_interrupted", binding["terminal_code"]
        )
        self.assertIsNotNone(binding["imported_at"])
        self.assertEqual(
            ("rejected", "benchmark_validation_collector_interrupted"), tuple(gate)
        )

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
            source_manifest_hash=service.prepare_context(
                self.run_id,
                paths,
                failure_workflow_node_keys=service._failure_node_keys(
                    self.run_id, "M1"
                ),
            ).manifest_hash,
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

    def test_reconcile_accepted_milestone_copies_immutable_evidence_between_equal_topologies(self) -> None:
        service = self._service()
        service.failures = SimpleNamespace(
            import_repository=mock.Mock(),
            open_related_to_plan=mock.Mock(
                return_value=[SimpleNamespace(artifact_path="docs/plans/runtime/01/failure-m3.md")]
            ),
        )
        plan_path = "docs/plans/runtime/01-control.md"
        self.sessions.set_status("session-a", SessionStatus.COMPLETED)
        self.sessions.register(session_id="session-b", plan_path=plan_path)
        self.sessions.set_status("session-b", SessionStatus.ACTIVE)
        target = TopologyImporter(self.database, self.repo).import_plan("session-b", plan_path)
        WorkflowStore(self.database).synchronize_session(self.sessions.get("session-b"))

        historical_path = "src/historical.py"
        (self.repo / historical_path).parent.mkdir(parents=True, exist_ok=True)
        (self.repo / historical_path).write_text("accepted historical content\n", encoding="utf-8")
        subprocess.run(["git", "add", "--", historical_path], cwd=self.repo, check=True)
        subprocess.run(["git", "commit", "-q", "-m", "test: add historical evidence"], cwd=self.repo, check=True)
        commit_sha = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=self.repo, check=True, capture_output=True, text=True
        ).stdout.strip()
        paths = (historical_path,)
        manifest_hash = service._manifest_hash_from_commit(commit_sha, paths)
        source_node_id = f"{self.run_id}:M1"
        now = "2026-07-15T00:00:00+00:00"
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO workflow_milestone_manifests(
                       manifest_id, run_id, topology_version_id, node_id, session_id,
                       paths_json, manifest_hash, actor, action_id, created_at
                   ) VALUES ('source-manifest', ?, ?, ?, 'session-a', ?, ?, 'source', 'bind', ?)""",
                (self.run_id, self.topology_version_id, source_node_id, json.dumps(paths), manifest_hash, now),
            )
            connection.execute(
                """INSERT INTO workflow_commit_intents(
                       intent_id, run_id, topology_version_id, node_id, session_id,
                       action_id, actor, gate_fingerprint, paths_json, message, status,
                       commit_sha, created_at, updated_at
                   ) VALUES ('source-intent', ?, ?, ?, 'session-a', 'source-action',
                             'source', 'gate', ?, 'test(runtime): accept historical evidence',
                             'reconciled', ?, ?, ?)""",
                (self.run_id, self.topology_version_id, source_node_id, json.dumps(paths), commit_sha, now, now),
            )
            connection.execute(
                """INSERT INTO workflow_attempts(
                       attempt_id, run_id, node_id, attempt_number, state, accepted,
                       evidence_json, started_at, completed_at
                   ) VALUES ('source-attempt', ?, ?, 1, 'succeeded', 1, ?, ?, ?)""",
                (
                    self.run_id,
                    source_node_id,
                    json.dumps({"commitSha": commit_sha, "intentId": "source-intent"}),
                    now,
                    now,
                ),
            )
            connection.execute(
                "UPDATE workflow_nodes SET state='succeeded', attempt_count=1 WHERE node_id=?",
                (source_node_id,),
            )
            source_version = connection.execute(
                "SELECT * FROM workflow_topology_versions WHERE topology_version_id=?",
                (self.topology_version_id,),
            ).fetchone()
            refreshed_topology = json.loads(source_version["topology_json"])
            refreshed_topology["content_hash"] = "refreshed-plan-content"
            connection.execute(
                """INSERT INTO workflow_topology_versions(
                       topology_version_id, run_id, version_number, plan_path, plan_id,
                       schema_version, source_kind, content_hash, topology_hash, topology_json,
                       supersedes_id, created_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    "source-refreshed-topology",
                    self.run_id,
                    int(source_version["version_number"]) + 1,
                    source_version["plan_path"],
                    source_version["plan_id"],
                    source_version["schema_version"],
                    source_version["source_kind"],
                    "refreshed-plan-content",
                    source_version["topology_hash"],
                    json.dumps(refreshed_topology, sort_keys=True, separators=(",", ":")),
                    self.topology_version_id,
                    now,
                ),
            )
            connection.execute(
                "UPDATE workflow_runs SET current_topology_version_id=? WHERE run_id=?",
                ("source-refreshed-topology", self.run_id),
            )
            connection.execute(
                "UPDATE workflow_runs SET state='stale' WHERE run_id=?",
                (target.run_id,),
            )

        result = service.reconcile_accepted_milestones(
            source_run_id=self.run_id,
            target_run_id=target.run_id,
            milestone_keys=("M1",),
            actor="maintainer",
            action_id="reconcile-action",
        )

        self.assertEqual(["M1"], [item["milestoneId"] for item in result["nodes"]])
        self.assertEqual(
            ["docs/plans/runtime/01/failure-m3.md"], result["openFailurePaths"]
        )
        with self.database.connect() as connection:
            target_node = connection.execute(
                "SELECT state, attempt_count FROM workflow_nodes WHERE node_id=?",
                (f"{target.run_id}:M1",),
            ).fetchone()
            target_manifest = connection.execute(
                """SELECT manifest_hash, paths_json, action_id
                   FROM workflow_milestone_manifests
                   WHERE run_id=? AND node_id=?""",
                (target.run_id, f"{target.run_id}:M1"),
            ).fetchone()
            target_intent = connection.execute(
                """SELECT intent_id, status, commit_sha, action_id
                   FROM workflow_commit_intents WHERE run_id=? AND node_id=?""",
                (target.run_id, f"{target.run_id}:M1"),
            ).fetchone()
            target_attempt = connection.execute(
                "SELECT accepted, evidence_json FROM workflow_attempts WHERE run_id=? AND node_id=?",
                (target.run_id, f"{target.run_id}:M1"),
            ).fetchone()

        self.assertEqual(("succeeded", 1), tuple(target_node))
        self.assertEqual((manifest_hash, json.dumps(paths), "reconcile-action"), tuple(target_manifest))
        self.assertEqual(("reconciled", commit_sha, "reconcile-action"), tuple(target_intent[1:]))
        self.assertEqual(1, target_attempt["accepted"])
        evidence = json.loads(target_attempt["evidence_json"])
        self.assertEqual(self.run_id, evidence["reconciledFromRunId"])
        self.assertEqual(target_intent["intent_id"], evidence["intentId"])
        self.assertEqual("source-intent", evidence["sourceIntentId"])

    def test_reconcile_accepted_milestone_recovers_legacy_second_hop_intent_reference(self) -> None:
        service = self._service()
        plan_path = "docs/plans/runtime/01-control.md"
        self.sessions.set_status("session-a", SessionStatus.COMPLETED)
        self.sessions.register(session_id="session-b", plan_path=plan_path)
        self.sessions.set_status("session-b", SessionStatus.ACTIVE)
        target = TopologyImporter(self.database, self.repo).import_plan("session-b", plan_path)
        WorkflowStore(self.database).synchronize_session(self.sessions.get("session-b"))

        historical_path = "src/legacy_second_hop.py"
        (self.repo / historical_path).parent.mkdir(parents=True, exist_ok=True)
        (self.repo / historical_path).write_text("accepted historical content\n", encoding="utf-8")
        subprocess.run(["git", "add", "--", historical_path], cwd=self.repo, check=True)
        subprocess.run(["git", "commit", "-q", "-m", "test: add legacy reconciliation evidence"], cwd=self.repo, check=True)
        commit_sha = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=self.repo, check=True, capture_output=True, text=True
        ).stdout.strip()
        paths = (historical_path,)
        manifest_hash = service._manifest_hash_from_commit(commit_sha, paths)
        source_node_id = f"{self.run_id}:M1"
        now = "2026-07-15T00:00:00+00:00"
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO workflow_milestone_manifests(
                       manifest_id, run_id, topology_version_id, node_id, session_id,
                       paths_json, manifest_hash, actor, action_id, created_at
                   ) VALUES ('legacy-source-manifest', ?, ?, ?, 'session-a', ?, ?, 'source', 'bind', ?)""",
                (self.run_id, self.topology_version_id, source_node_id, json.dumps(paths), manifest_hash, now),
            )
            connection.execute(
                """INSERT INTO workflow_commit_intents(
                       intent_id, run_id, topology_version_id, node_id, session_id,
                       action_id, actor, gate_fingerprint, paths_json, message, status,
                       commit_sha, created_at, updated_at
                   ) VALUES ('legacy-local-reconciled-intent', ?, ?, ?, 'session-a', 'source-action',
                             'source', 'gate', ?, 'test(runtime): accept legacy evidence',
                             'reconciled', ?, ?, ?)""",
                (self.run_id, self.topology_version_id, source_node_id, json.dumps(paths), commit_sha, now, now),
            )
            connection.execute(
                """INSERT INTO workflow_attempts(
                       attempt_id, run_id, node_id, attempt_number, state, accepted,
                       evidence_json, started_at, completed_at
                   ) VALUES ('legacy-source-attempt', ?, ?, 1, 'succeeded', 1, ?, ?, ?)""",
                (
                    self.run_id,
                    source_node_id,
                    json.dumps(
                        {
                            "commitSha": commit_sha,
                            "intentId": "legacy-source-intent",
                            "sourceIntentId": "legacy-source-intent",
                            "reconciledFromRunId": "older-run",
                        }
                    ),
                    now,
                    now,
                ),
            )
            connection.execute(
                "UPDATE workflow_nodes SET state='succeeded', attempt_count=1 WHERE node_id=?",
                (source_node_id,),
            )

        result = service.reconcile_accepted_milestones(
            source_run_id=self.run_id,
            target_run_id=target.run_id,
            milestone_keys=("M1",),
            actor="maintainer",
            action_id="second-hop",
        )

        self.assertEqual(["M1"], [item["milestoneId"] for item in result["nodes"]])
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT evidence_json FROM workflow_attempts WHERE run_id=? AND node_id=?",
                (target.run_id, f"{target.run_id}:M1"),
            ).fetchone()
        evidence = json.loads(row["evidence_json"])
        self.assertEqual(result["nodes"][0]["intentId"], evidence["intentId"])
        self.assertEqual("legacy-local-reconciled-intent", evidence["sourceIntentId"])
        self.assertEqual("legacy-source-intent", evidence["legacyEvidenceIntentId"])

    def test_reconcile_rejects_terminal_target_run(self) -> None:
        service = self._service()
        plan_path = "docs/plans/runtime/01-control.md"
        self.sessions.set_status("session-a", SessionStatus.COMPLETED)
        self.sessions.register(session_id="session-terminal", plan_path=plan_path)
        self.sessions.set_status("session-terminal", SessionStatus.ACTIVE)
        target = TopologyImporter(self.database, self.repo).import_plan(
            "session-terminal", plan_path
        )
        WorkflowStore(self.database).synchronize_session(
            self.sessions.get("session-terminal")
        )
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE workflow_runs SET state='succeeded' WHERE run_id=?",
                (target.run_id,),
            )

        with self.assertRaises(CoordinatorError) as rejected:
            service.reconcile_accepted_milestones(
                source_run_id=self.run_id,
                target_run_id=target.run_id,
                milestone_keys=("M1",),
                actor="maintainer",
                action_id="terminal-target",
            )

        self.assertEqual("workflow_reconcile_target_terminal", rejected.exception.code)

    def test_reconciliation_does_not_skip_unaccepted_dependencies(self) -> None:
        records = [
            {
                "milestone_key": "M2",
                "dependencies": ("M1",),
            }
        ]

        unresolved = MilestoneWorkflowService._reconciliation_unaccepted_dependencies(
            records, set()
        )

        self.assertEqual({"M2": ["M1"]}, unresolved)
        self.assertEqual(
            {},
            MilestoneWorkflowService._reconciliation_unaccepted_dependencies(
                records, {"M1"}
            ),
        )


if __name__ == "__main__":
    unittest.main()
