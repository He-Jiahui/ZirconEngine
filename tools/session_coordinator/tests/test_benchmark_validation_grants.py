from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.benchmark_validation_grants import (
    BenchmarkValidationGrantService,
)
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError, SessionStatus
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class BenchmarkValidationGrantTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.repo = init_repo(root / "repo")
        self.database = Database(root / "state.sqlite3")
        migrate(self.database)
        sessions = SessionService(self.database, self.repo)
        for session_id, plan_path in (
            ("source-a", "docs/plans/plugins/01-plugin.md"),
            ("source-b", "docs/plans/plugins/01-plugin.md"),
            ("target-a", "docs/plans/plugins/01-plugin.md"),
            ("target-b", "docs/plans/plugins/01-plugin.md"),
            ("foreign-source", "docs/plans/runtime/02-runtime.md"),
        ):
            sessions.register(session_id=session_id, plan_path=plan_path)
            sessions.set_status(session_id, SessionStatus.ACTIVE)
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO workflow_runs(
                       run_id, session_id, workflow_key, plan_path, state,
                       created_at, updated_at
                   ) VALUES ('workflow-run', 'target-a', 'plugins-01',
                             'docs/plans/plugins/01-plugin.md', 'active',
                             '2026-08-11T00:00:00+00:00',
                             '2026-08-11T00:00:00+00:00')"""
            )
        self.service = BenchmarkValidationGrantService(self.database)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def _copy(
        self,
        job_id: str,
        session_id: str,
        manifest: str,
        *,
        materialization_kind: str = "cargo",
    ) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO validation_copies(
                       job_id, session_id, job_root, source_root, target_root,
                       head_commit, manifest_json, status, created_at,
                       input_manifest_hash, materialization_kind,
                       materialization_phase
                    ) VALUES (?, ?, ?, ?, ?, 'head', '[]', 'materialized',
                              '2026-08-11T00:00:00+00:00', ?, ?,
                              'materialized')""",
                (
                    job_id,
                    session_id,
                    str(self.repo.parent / job_id),
                    str(self.repo.parent / job_id / "source"),
                    str(self.repo.parent / job_id / "target"),
                    manifest,
                    materialization_kind,
                ),
            )

    @staticmethod
    def _command(name: str) -> tuple[str, ...]:
        return ("cargo", "test", name, "--", "--ignored")

    def test_issue_selects_unique_same_plan_copy_without_caller_job_id(self) -> None:
        self._copy(
            "generic-copy",
            "source-a",
            "f" * 64,
            materialization_kind="validation",
        )
        self._copy("copy-a", "source-a", "a" * 64)
        candidate = self.service.select_candidate(
            source_session_id="source-a", target_session_id="target-a"
        )

        grant = self.service.issue(
            candidate=candidate,
            target_session_id="target-a",
            run_id="workflow-run",
            milestone_id="M1",
            benchmark_name="native_host_context_lookup_1_thread_benchmark",
            cargo_profile="release",
            command=self._command("native_host_context_lookup_1_thread_benchmark"),
            scoped_manifest_hash="b" * 64,
        )

        self.assertEqual("copy-a", grant.job_id)
        self.assertEqual("a" * 64, grant.input_manifest_hash)
        self.assertEqual("b" * 64, grant.scoped_manifest_hash)
        self.assertEqual(1, grant.fifo_sequence)
        payload = grant.to_dict()
        self.assertEqual(grant.grant_id, payload["grantId"])
        self.assertEqual("source-a", payload["sourceSessionId"])
        self.assertNotIn("jobId", payload)
        self.assertNotIn("grant_id", payload)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM benchmark_validation_grants WHERE grant_id=?",
                (grant.grant_id,),
            ).fetchone()
        self.assertEqual("issued", row["status"])
        self.assertEqual("copy-a", row["job_id"])

    def test_restart_denies_unregistered_launching_grant_and_unblocks_fifo(self) -> None:
        self._copy("copy-a", "source-a", "a" * 64)
        grant = self.service.issue(
            candidate=self.service.select_candidate(
                source_session_id="source-a", target_session_id="target-a"
            ),
            target_session_id="target-a",
            run_id="workflow-run",
            milestone_id="M1",
            benchmark_name="native_host_context_lookup_1_thread_benchmark",
            cargo_profile="release",
            command=self._command("native_host_context_lookup_1_thread_benchmark"),
            scoped_manifest_hash="b" * 64,
        )
        acquired = self.service.acquire(
            target_session_id="target-a",
            run_id="workflow-run",
            milestone_id="M1",
            benchmark_name=grant.benchmark_name,
            cargo_profile=grant.cargo_profile,
            command=grant.command,
        )
        self.assertEqual("launching", acquired.status)

        BenchmarkValidationGrantService(self.database)

        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status, error_code FROM benchmark_validation_grants "
                "WHERE grant_id=?",
                (grant.grant_id,),
            ).fetchone()
        self.assertEqual("denied", row["status"])
        self.assertEqual(
            "benchmark_validation_grant_launch_interrupted", row["error_code"]
        )

    def test_restart_rejects_consumed_grant_without_terminal_evidence(self) -> None:
        self._copy("copy-a", "source-a", "a" * 64)
        grant = self.service.issue(
            candidate=self.service.select_candidate(
                source_session_id="source-a", target_session_id="target-a"
            ),
            target_session_id="target-a",
            run_id="workflow-run",
            milestone_id="M1",
            benchmark_name="native_host_context_lookup_1_thread_benchmark",
            cargo_profile="release",
            command=self._command("native_host_context_lookup_1_thread_benchmark"),
            scoped_manifest_hash="b" * 64,
        )
        self.service.acquire(
            target_session_id="target-a",
            run_id="workflow-run",
            milestone_id="M1",
            benchmark_name=grant.benchmark_name,
            cargo_profile=grant.cargo_profile,
            command=grant.command,
        )
        with self.database.transaction() as connection:
            connection.execute(
                """UPDATE benchmark_validation_grants
                   SET status='consumed', validation_run_id='validation-a',
                       root_pid=4242, root_process_creation_time='111222',
                       job_isolated=1,
                       consumed_at='2026-08-11T00:01:00+00:00'
                   WHERE grant_id=?""",
                (grant.grant_id,),
            )
            connection.execute(
                "UPDATE validation_copies SET status='running', run_pid=4242 "
                "WHERE job_id='copy-a'"
            )
        recovery_events: list[str] = []
        reject_validation = mock.Mock(
            side_effect=lambda *_args, **_kwargs: recovery_events.append("reject") or True
        )
        terminate_interrupted = mock.Mock(
            side_effect=lambda **_kwargs: recovery_events.append("terminate")
        )

        recovered = BenchmarkValidationGrantService(
            self.database
        ).reconcile_interrupted_consumed(
            reject_validation,
            terminate_interrupted=terminate_interrupted,
        )

        self.assertEqual(("validation-a",), recovered)
        terminate_interrupted.assert_called_once_with(
            grant_id=grant.grant_id,
            job_id="copy-a",
            root_pid=4242,
            process_creation_time="111222",
            job_isolated=True,
        )
        reject_validation.assert_called_once_with(
            "validation-a",
            error_code="benchmark_validation_collector_interrupted",
        )
        self.assertEqual(["terminate", "reject"], recovery_events)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status, error_code FROM benchmark_validation_grants "
                "WHERE grant_id=?",
                (grant.grant_id,),
            ).fetchone()
        self.assertEqual("consumed", row["status"])
        self.assertEqual(
            "benchmark_validation_collector_interrupted", row["error_code"]
        )

    def test_foreign_plan_and_ambiguous_copy_are_denied_without_copy_mutation(self) -> None:
        self._copy("foreign-copy", "foreign-source", "f" * 64)
        with self.assertRaises(CoordinatorError) as foreign:
            self.service.select_candidate(
                source_session_id="foreign-source", target_session_id="target-a"
            )
        self.assertEqual("benchmark_validation_grant_plan_mismatch", foreign.exception.code)

        self._copy("copy-a", "source-a", "a" * 64)
        self._copy("copy-a-2", "source-a", "b" * 64)
        with self.assertRaises(CoordinatorError) as ambiguous:
            self.service.select_candidate(
                source_session_id="source-a", target_session_id="target-a"
            )
        self.assertEqual("benchmark_validation_grant_copy_ambiguous", ambiguous.exception.code)
        with self.database.connect() as connection:
            statuses = {
                row["job_id"]: row["status"]
                for row in connection.execute(
                    "SELECT job_id, status FROM validation_copies"
                )
            }
        self.assertEqual(
            {
                "foreign-copy": "materialized",
                "copy-a": "materialized",
                "copy-a-2": "materialized",
            },
            statuses,
        )

    def test_acquire_is_fifo_target_bound_and_not_replayable(self) -> None:
        self._copy("copy-a", "source-a", "a" * 64)
        self._copy("copy-b", "source-b", "b" * 64)
        first = self.service.issue(
            candidate=self.service.select_candidate(
                source_session_id="source-a", target_session_id="target-a"
            ),
            target_session_id="target-a",
            run_id="workflow-run",
            milestone_id="M1",
            benchmark_name="native_host_context_lookup_1_thread_benchmark",
            cargo_profile="release",
            command=self._command("native_host_context_lookup_1_thread_benchmark"),
            scoped_manifest_hash="c" * 64,
        )
        second = self.service.issue(
            candidate=self.service.select_candidate(
                source_session_id="source-b", target_session_id="target-a"
            ),
            target_session_id="target-a",
            run_id="workflow-run",
            milestone_id="M1",
            benchmark_name="native_runtime_broadcast_1_plugin_benchmark",
            cargo_profile="profiling",
            command=self._command("native_runtime_broadcast_1_plugin_benchmark"),
            scoped_manifest_hash="d" * 64,
        )

        with self.assertRaises(CoordinatorError) as fifo:
            self.service.acquire(
                target_session_id="target-a",
                run_id="workflow-run",
                milestone_id="M1",
                benchmark_name=second.benchmark_name,
                cargo_profile=second.cargo_profile,
                command=second.command,
            )
        self.assertEqual("benchmark_validation_grant_fifo_wait", fifo.exception.code)
        with self.assertRaises(CoordinatorError) as cross_session:
            self.service.acquire(
                target_session_id="target-b",
                run_id="workflow-run",
                milestone_id="M1",
                benchmark_name=first.benchmark_name,
                cargo_profile=first.cargo_profile,
                command=first.command,
            )
        self.assertEqual(
            "benchmark_validation_grant_required", cross_session.exception.code
        )

        acquired = self.service.acquire(
            target_session_id="target-a",
            run_id="workflow-run",
            milestone_id="M1",
            benchmark_name=first.benchmark_name,
            cargo_profile=first.cargo_profile,
            command=first.command,
        )
        self.assertEqual(first.grant_id, acquired.grant_id)
        with self.assertRaises(CoordinatorError) as replay:
            self.service.acquire(
                target_session_id="target-a",
                run_id="workflow-run",
                milestone_id="M1",
                benchmark_name=first.benchmark_name,
                cargo_profile=first.cargo_profile,
                command=first.command,
            )
        self.assertEqual("benchmark_validation_grant_fifo_wait", replay.exception.code)

        self.service.deny(first.grant_id, error_code="test_denied")
        acquired_second = self.service.acquire(
            target_session_id="target-a",
            run_id="workflow-run",
            milestone_id="M1",
            benchmark_name=second.benchmark_name,
            cargo_profile=second.cargo_profile,
            command=second.command,
        )
        self.assertEqual(second.grant_id, acquired_second.grant_id)

    def test_acquire_revalidation_denies_grant_without_changing_copy(self) -> None:
        self._copy("copy-a", "source-a", "a" * 64)
        grant = self.service.issue(
            candidate=self.service.select_candidate(
                source_session_id="source-a", target_session_id="target-a"
            ),
            target_session_id="target-a",
            run_id="workflow-run",
            milestone_id="M1",
            benchmark_name="native_host_context_lookup_1_thread_benchmark",
            cargo_profile="release",
            command=self._command("native_host_context_lookup_1_thread_benchmark"),
            scoped_manifest_hash="b" * 64,
        )
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET plan_path='docs/plans/plugins/changed.md' "
                "WHERE session_id='source-a'"
            )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.acquire(
                target_session_id="target-a",
                run_id="workflow-run",
                milestone_id="M1",
                benchmark_name=grant.benchmark_name,
                cargo_profile=grant.cargo_profile,
                command=grant.command,
            )

        self.assertEqual(
            "benchmark_validation_grant_plan_mismatch", rejected.exception.code
        )
        with self.database.connect() as connection:
            grant_row = connection.execute(
                "SELECT status, error_code FROM benchmark_validation_grants"
            ).fetchone()
            copy_status = connection.execute(
                "SELECT status FROM validation_copies WHERE job_id='copy-a'"
            ).fetchone()["status"]
        self.assertEqual("denied", grant_row["status"])
        self.assertEqual(rejected.exception.code, grant_row["error_code"])
        self.assertEqual("materialized", copy_status)


if __name__ == "__main__":
    unittest.main()
