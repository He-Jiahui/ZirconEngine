from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.database import Database
from tools.session_coordinator.integration_candidates import IntegrationCandidateService
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.notifications import WeComNotificationService
from tools.session_coordinator.tests.helpers import init_repo


class IntegrationCandidateTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        root = Path(self.temporary.name)
        self.repo = init_repo(root / "repo")
        self.database = Database(root / "state" / "coordinator.sqlite3")
        migrate(self.database)
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO sessions(
                    session_id, plan_path, status, created_at, updated_at, last_heartbeat_at
                ) VALUES ('primary', 'docs/plans/tooling/01-tooling.md', 'active',
                          '2026-07-31T00:00:00+00:00', '2026-07-31T00:00:00+00:00',
                          '2026-07-31T00:00:00+00:00')
                """
            )
            connection.execute(
                """
                INSERT INTO validation_tickets(
                    ticket_id, session_id, plan_path, status, dedupe_key,
                    source_manifest_hash, source_manifest_json, command_json,
                    toolchain_json, coverage_json, created_at, updated_at
                ) VALUES ('compile-pass', 'primary', 'docs/plans/tooling/01-tooling.md',
                          'passed', 'dedupe', 'a', '{}', '[]', '{}', '{}', 'now', 'now')
                """
            )
        self.leases = LeaseService(
            self.database,
            PathPolicy(self.repo),
            ttl_seconds=900,
            grace_seconds=120,
        )
        self.service = IntegrationCandidateService(self.database, self.repo, self.leases)

    def _enable_notifications(self) -> list[str]:
        messages: list[str] = []

        def notify(command: list[str]) -> subprocess.CompletedProcess[str]:
            messages.append(command[command.index("-Message") + 1])
            return subprocess.CompletedProcess(command, 0, '{"errcode":0}', "")

        self.service.set_notifications(
            WeComNotificationService(
                self.database,
                script_path=self.repo / "send-wecom-message.ps1",
                runner=notify,
            )
        )
        return messages

    def _lease(self, path: str) -> None:
        acquisition = self.leases.acquire("primary", [path])
        self.assertTrue(acquisition.acquired, acquisition.conflicts)

    def test_submit_seals_current_blobs_and_replays_the_same_request(self) -> None:
        source = self.repo / "tools" / "candidate.py"
        source.parent.mkdir(parents=True)
        source.write_text("value = 1\n", encoding="utf-8")
        self._lease("tools/candidate.py")

        first = self.service.submit(
            session_id="primary",
            request_id="candidate-request",
            paths=("tools/candidate.py",),
            compile_ticket_id="compile-pass",
        )
        source.write_text("value = 2\n", encoding="utf-8")
        replay = self.service.submit(
            session_id="primary",
            request_id="candidate-request",
            paths=("tools/candidate.py",),
            compile_ticket_id="compile-pass",
        )

        self.assertEqual("integration_ready", first.status)
        self.assertEqual(first, replay)
        self.assertEqual("tools/candidate.py", first.lease_evidence[0]["candidatePath"])
        blob_content = subprocess.run(
            ["git", "cat-file", "-p", first.paths[0].blob_oid],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout
        self.assertEqual("value = 1\n", blob_content)

    def test_finalize_commits_sealed_blob_without_touching_later_worktree_edits(self) -> None:
        messages = self._enable_notifications()
        source = self.repo / "tools" / "candidate.py"
        source.parent.mkdir(parents=True)
        source.write_text("value = 1\n", encoding="utf-8")
        self._lease("tools/candidate.py")
        candidate = self.service.submit(
            session_id="primary",
            request_id="candidate-finalize",
            paths=("tools/candidate.py",),
            compile_ticket_id="compile-pass",
        )
        source.write_text("value = 2\n", encoding="utf-8")

        integrated = self.service.finalize(
            candidate.candidate_id, message="integration: sealed candidate"
        )

        self.assertEqual("integrated_validation_pending", integrated.status)
        self.assertIsNotNone(integrated.commit_sha)
        committed = subprocess.run(
            ["git", "show", f"{integrated.commit_sha}:tools/candidate.py"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout
        self.assertEqual("value = 1\n", committed)
        self.assertEqual("value = 2\n", source.read_text(encoding="utf-8"))
        self.assertEqual(1, len(messages))
        with self.database.connect() as connection:
            notification = connection.execute(
                "SELECT commit_sha, status FROM notification_attempts"
            ).fetchone()
        self.assertEqual(integrated.commit_sha, notification["commit_sha"])
        self.assertEqual("succeeded", notification["status"])

    def test_finalize_defers_when_main_changed_the_same_candidate_path(self) -> None:
        source = self.repo / "tools" / "candidate.py"
        source.parent.mkdir(parents=True)
        source.write_text("value = 1\n", encoding="utf-8")
        self._lease("tools/candidate.py")
        candidate = self.service.submit(
            session_id="primary",
            request_id="candidate-conflict",
            paths=("tools/candidate.py",),
            compile_ticket_id="compile-pass",
        )
        source.write_text("external = True\n", encoding="utf-8")
        subprocess.run(["git", "add", "tools/candidate.py"], cwd=self.repo, check=True)
        subprocess.run(["git", "commit", "-qm", "test: external update"], cwd=self.repo, check=True)

        delayed = self.service.finalize(
            candidate.candidate_id, message="integration: sealed candidate"
        )

        self.assertEqual("delayed_merge", delayed.status)
        self.assertIsNone(delayed.commit_sha)

    def test_finalize_accepts_when_main_already_contains_the_sealed_blob(self) -> None:
        messages = self._enable_notifications()
        source = self.repo / "tools" / "candidate.py"
        source.parent.mkdir(parents=True)
        source.write_text("value = 1\n", encoding="utf-8")
        self._lease("tools/candidate.py")
        candidate = self.service.submit(
            session_id="primary",
            request_id="candidate-already-integrated",
            paths=("tools/candidate.py",),
            compile_ticket_id="compile-pass",
        )
        subprocess.run(["git", "add", "tools/candidate.py"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "test: integrate sealed blob"],
            cwd=self.repo,
            check=True,
        )
        current_head = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()

        accepted = self.service.finalize(
            candidate.candidate_id, message="integration: sealed candidate"
        )

        self.assertEqual("accepted", accepted.status)
        self.assertEqual(current_head, accepted.commit_sha)
        self.assertEqual([], messages)
        with self.database.connect() as connection:
            notification_count = connection.execute(
                "SELECT COUNT(*) FROM notification_attempts"
            ).fetchone()[0]
        self.assertEqual(0, notification_count)


if __name__ == "__main__":
    unittest.main()
