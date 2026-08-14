from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.control_plane.actions.catalog import action_spec
from tools.session_coordinator.control_plane.actions.fingerprint import ActionFingerprinter
from tools.session_coordinator.control_plane.actions.models import ActionKind
from tools.session_coordinator.database import Database
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import SessionStatus
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.workflows.plan_import import TopologyImporter


class ActionFingerprintTests(unittest.TestCase):
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
        target = self.repo / "src" / "feature.py"
        target.parent.mkdir(parents=True)
        target.write_text("one\n", encoding="utf-8")
        self.baselines.attribute("session-a", ["src/feature.py"])
        self.fingerprinter = ActionFingerprinter(
            self.database, self.repo, daemon_instance_id="instance-a"
        )
        self.spec = action_spec(ActionKind.LEASE_CLAIM.value)
        self.parameters = self.spec.parse_parameters({"sessionId": "session-a"})

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def _digest(self) -> str:
        return self.fingerprinter.capture(
            self.spec, self.parameters, bound_session_id="session-a"
        ).digest

    def test_head_index_target_lease_and_session_changes_alter_digest(self) -> None:
        initial = self._digest()
        subprocess.run(["git", "add", "README.md"], cwd=self.repo, check=True)
        (self.repo / "README.md").write_text("index change\n", encoding="utf-8")
        subprocess.run(["git", "add", "README.md"], cwd=self.repo, check=True)
        after_index = self._digest()
        self.assertNotEqual(initial, after_index)

        (self.repo / "src" / "feature.py").write_text("two\n", encoding="utf-8")
        after_target = self._digest()
        self.assertNotEqual(after_index, after_target)

        self.assertTrue(self.leases.acquire("session-a", ["src/feature.py"]).acquired)
        after_lease = self._digest()
        self.assertNotEqual(after_target, after_lease)

        self.sessions.set_status("session-a", SessionStatus.WAITING_VALIDATION)
        after_session = self._digest()
        self.assertNotEqual(after_lease, after_session)

    def test_unstaged_unowned_change_does_not_alter_index_identity(self) -> None:
        initial = self._digest()
        (self.repo / "README.md").write_text("unowned worktree change\n", encoding="utf-8")
        self.assertEqual(initial, self._digest())

    def test_head_change_alters_digest_without_owned_file_change(self) -> None:
        initial = self._digest()
        subprocess.run(
            ["git", "commit", "--allow-empty", "-m", "test: advance head"],
            cwd=self.repo,
            check=True,
            capture_output=True,
        )
        self.assertNotEqual(initial, self._digest())

    def test_patch_validation_and_failure_markdown_changes_alter_digest(self) -> None:
        patch_spec = action_spec(ActionKind.PATCH_PROCESS.value)
        patch_parameters = patch_spec.parse_parameters({"sessionId": "session-a"})
        initial = self.fingerprinter.capture(
            patch_spec, patch_parameters, bound_session_id="session-a"
        ).digest
        with self.database.transaction() as connection:
            connection.execute(
                "INSERT INTO objects VALUES ('object-a', 1, 1, 'now')"
            )
            connection.execute(
                """INSERT INTO patches(
                       session_id, patch_object_hash, targets_json,
                       base_hashes_json, base_objects_json, status,
                       created_at, updated_at
                   ) VALUES ('session-a', 'object-a', '["src/feature.py"]',
                             '{}', '{}', 'queued', 'now', 'now')"""
            )
        after_patch = self.fingerprinter.capture(
            patch_spec, patch_parameters, bound_session_id="session-a"
        ).digest
        self.assertNotEqual(initial, after_patch)

        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO validation_copies(
                       job_id, session_id, job_root, source_root, target_root,
                       head_commit, manifest_json, status, created_at
                   ) VALUES ('joba', 'session-a', 'job', 'source', 'target',
                             'head', '[]', 'materialized', 'now')"""
            )
        validation_spec = action_spec(ActionKind.VALIDATION_CANCEL.value)
        validation_parameters = validation_spec.parse_parameters(
            {"sessionId": "session-a", "jobId": "joba"}
        )
        validation_before = self.fingerprinter.capture(
            validation_spec, validation_parameters, bound_session_id="session-a"
        ).digest
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE validation_copies SET status = 'running' WHERE job_id = 'joba'"
            )
        validation_after = self.fingerprinter.capture(
            validation_spec, validation_parameters, bound_session_id="session-a"
        ).digest
        self.assertNotEqual(validation_before, validation_after)

        handoff = self.repo / "docs/plans/runtime/01/failure-2026-07-11-test.md"
        handoff.parent.mkdir(parents=True, exist_ok=True)
        failure_spec = action_spec(ActionKind.FAILURE_REFRESH.value)
        failure_parameters = failure_spec.parse_parameters({"sessionId": "session-a"})
        failure_before = self.fingerprinter.capture(
            failure_spec, failure_parameters, bound_session_id="session-a"
        ).digest
        handoff.write_text("failure snapshot one\n", encoding="utf-8")
        after_failure = self.fingerprinter.capture(
            failure_spec, failure_parameters, bound_session_id="session-a"
        ).digest
        self.assertNotEqual(failure_before, after_failure)
        handoff.write_text("failure snapshot two\n", encoding="utf-8")
        self.assertNotEqual(
            after_failure,
            self.fingerprinter.capture(
                failure_spec, failure_parameters, bound_session_id="session-a"
            ).digest,
        )

    def test_unrelated_session_resources_do_not_invalidate_heartbeat(self) -> None:
        self.sessions.register(session_id="session-b")
        spec = action_spec(ActionKind.SESSION_HEARTBEAT.value)
        parameters = spec.parse_parameters({"sessionId": "session-a"})
        before = self.fingerprinter.capture(
            spec, parameters, bound_session_id="session-a"
        ).digest
        with self.database.transaction() as connection:
            connection.execute("INSERT INTO objects VALUES ('foreign', 1, 1, 'now')")
            connection.execute(
                """INSERT INTO patches(
                       session_id, patch_object_hash, targets_json,
                       base_hashes_json, base_objects_json, status,
                       created_at, updated_at
                   ) VALUES ('session-b', 'foreign', '[]', '{}', '{}',
                             'queued', 'now', 'now')"""
            )
            connection.execute(
                """INSERT INTO leases(path_key, display_path, session_id, acquired_at,
                       last_heartbeat_at, expires_at)
                   VALUES ('foreign.txt', 'foreign.txt', 'session-b', 'now', 'now', 'later')"""
            )
        after = self.fingerprinter.capture(
            spec, parameters, bound_session_id="session-a"
        ).digest
        self.assertEqual(before, after)

    def test_benchmark_grant_fingerprint_tracks_source_copy_state(self) -> None:
        self.sessions.register(
            session_id="source-session",
            plan_path="docs/plans/runtime/01-feature.md",
        )
        self.sessions.set_status("source-session", SessionStatus.ACTIVE)
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO workflow_runs(
                       run_id, session_id, workflow_key, plan_path, state,
                       created_at, updated_at
                   ) VALUES ('benchmark-run', 'session-a', 'benchmark',
                             'docs/plans/runtime/01-feature.md', 'active',
                             'now', 'now')"""
            )
            connection.execute(
                """INSERT INTO validation_copies(
                       job_id, session_id, job_root, source_root, target_root,
                       head_commit, manifest_json, status, created_at,
                       input_manifest_hash
                   ) VALUES ('benchmark-copy', 'source-session', 'job', 'source',
                             'target', 'head', '[]', 'materialized', 'now', ?)""",
                ("a" * 64,),
            )
        spec = action_spec(ActionKind.BENCHMARK_GRANT_ISSUE.value)
        parameters = spec.parse_parameters(
            {
                "sessionId": "session-a",
                "sourceSessionId": "source-session",
                "runId": "benchmark-run",
                "milestoneId": "M1",
                "benchmarkName": "native_host_context_lookup_1_thread_benchmark",
                "cargoProfile": "release",
            }
        )

        before = self.fingerprinter.capture(
            spec, parameters, bound_session_id="session-a"
        ).digest
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE validation_copies SET status='running' WHERE job_id='benchmark-copy'"
            )
        after = self.fingerprinter.capture(
            spec, parameters, bound_session_id="session-a"
        ).digest

        self.assertNotEqual(before, after)

    def test_milestone_reconciliation_fingerprint_tracks_both_run_evidence(self) -> None:
        plan_path = "docs/plans/runtime/01-feature.md"
        plan = self.repo / plan_path
        plan.parent.mkdir(parents=True, exist_ok=True)
        plan.write_text(
            "```zircon-workflow\n"
            '{"schema":1,"workflow_id":"fingerprint","goal":"test",'
            '"milestones":[{"id":"M1","title":"one","depends_on":[]}]}\n'
            "```\n",
            encoding="utf-8",
        )
        self.sessions.register(session_id="session-c", plan_path=plan_path)
        self.sessions.set_status("session-c", SessionStatus.ACTIVE)
        importer = TopologyImporter(self.database, self.repo)
        source = importer.import_plan("session-a", plan_path)
        target = importer.import_plan("session-c", plan_path)
        spec = action_spec(ActionKind.MILESTONE_RECONCILE.value)
        parameters = spec.parse_parameters(
            {
                "sourceRunId": source.run_id,
                "targetRunId": target.run_id,
                "milestoneIds": ["M1"],
            }
        )

        before = self.fingerprinter.capture(spec, parameters, bound_session_id=None).digest
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE workflow_runs SET updated_at='changed' WHERE run_id=?",
                (target.run_id,),
            )
        after = self.fingerprinter.capture(spec, parameters, bound_session_id=None).digest

        self.assertNotEqual(before, after)


if __name__ == "__main__":
    unittest.main()
