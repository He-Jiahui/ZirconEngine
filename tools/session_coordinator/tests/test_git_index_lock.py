from __future__ import annotations

import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.git_finalize import GitFinalizeService
from tools.session_coordinator.git_index_lock import (
    IndexLockRecoveryRefused,
    recover_stale_index_lock,
)
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import SessionStatus
from tools.session_coordinator.processes import file_owner_process_ids
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


_NOW_NS = 2_000_000_000_000
_OLD_NS = _NOW_NS - 120_000_000_000


def _recover(lock_path: Path, *, active_pids: tuple[int, ...] = ()):
    return recover_stale_index_lock(
        lock_path,
        minimum_age_seconds=30.0,
        observation_seconds=0.0,
        now_ns=lambda: _NOW_NS,
        sleep=lambda _: None,
        lock_owner_process_ids=lambda: active_pids,
    )


class GitIndexLockRecoveryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        self.lock_path = self.root / "index.lock"

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def _old_lock(self, content: bytes = b"") -> None:
        self.lock_path.write_bytes(content)
        os.utime(self.lock_path, ns=(_OLD_NS, _OLD_NS))

    def test_stable_old_zero_byte_lock_is_removed(self) -> None:
        self._old_lock()

        recovery = _recover(self.lock_path)

        self.assertIsNotNone(recovery)
        assert recovery is not None
        self.assertEqual(0, recovery.size)
        self.assertEqual(120.0, recovery.age_seconds)
        self.assertFalse(self.lock_path.exists())

    def test_nonzero_lock_is_refused_without_modification(self) -> None:
        self._old_lock(b"owned")

        with self.assertRaises(IndexLockRecoveryRefused) as rejected:
            _recover(self.lock_path)

        self.assertEqual("nonzero", rejected.exception.reason)
        self.assertEqual(b"owned", self.lock_path.read_bytes())

    def test_live_lock_owner_is_refused_without_modification(self) -> None:
        self._old_lock()

        with self.assertRaises(IndexLockRecoveryRefused) as rejected:
            _recover(self.lock_path, active_pids=(421,))

        self.assertEqual("active_lock_owner", rejected.exception.reason)
        self.assertEqual((421,), rejected.exception.active_pids)
        self.assertTrue(self.lock_path.exists())

    def test_identity_change_during_observation_is_refused(self) -> None:
        self._old_lock()

        def replace_lock(_: float) -> None:
            self.lock_path.unlink()
            self._old_lock()

        with self.assertRaises(IndexLockRecoveryRefused) as rejected:
            recover_stale_index_lock(
                self.lock_path,
                minimum_age_seconds=30.0,
                observation_seconds=0.0,
                now_ns=lambda: _NOW_NS,
                sleep=replace_lock,
                lock_owner_process_ids=lambda: (),
            )

        self.assertEqual("identity_changed", rejected.exception.reason)
        self.assertTrue(self.lock_path.exists())

    def test_young_lock_is_refused_without_owner_inspection(self) -> None:
        self.lock_path.write_bytes(b"")
        os.utime(self.lock_path, ns=(_NOW_NS, _NOW_NS))
        inspected = False

        def owners() -> tuple[int, ...]:
            nonlocal inspected
            inspected = True
            return ()

        with self.assertRaises(IndexLockRecoveryRefused) as rejected:
            recover_stale_index_lock(
                self.lock_path,
                now_ns=lambda: _NOW_NS,
                sleep=lambda _: None,
                lock_owner_process_ids=owners,
            )

        self.assertEqual("too_young", rejected.exception.reason)
        self.assertFalse(inspected)
        self.assertTrue(self.lock_path.exists())

    def test_owner_inspection_failure_is_fail_closed(self) -> None:
        self._old_lock()

        def unavailable() -> tuple[int, ...]:
            raise OSError("owner query unavailable")

        with self.assertRaises(IndexLockRecoveryRefused) as rejected:
            recover_stale_index_lock(
                self.lock_path,
                now_ns=lambda: _NOW_NS,
                sleep=lambda _: None,
                lock_owner_process_ids=unavailable,
            )

        self.assertEqual("process_inspection_failed", rejected.exception.reason)
        self.assertTrue(self.lock_path.exists())

    @unittest.skipUnless(os.name == "nt", "Restart Manager is Windows-only")
    def test_restart_manager_reports_the_exact_open_file_owner(self) -> None:
        self._old_lock()

        with self.lock_path.open("rb"):
            owners = file_owner_process_ids(self.lock_path)

        self.assertIn(os.getpid(), owners)


class GitFinalizeIndexLockIntegrationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)
        self.sessions.register(session_id="maintenance")
        self.sessions.set_status("maintenance", SessionStatus.ACTIVE)
        self.baselines = BaselineService(self.database, self.repo)
        self.baselines.initialize()
        self.service = GitFinalizeService(
            self.database,
            self.repo,
            self.baselines,
            self.sessions,
            index_lock_recoverer=_recover,
        )

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def test_maintenance_finalize_recovers_lock_and_persists_audit_event(self) -> None:
        path = "tools/recover_index_lock.py"
        target = self.repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("recovered = True\n", encoding="utf-8")
        lock_path = self.repo / ".git" / "index.lock"
        lock_path.write_bytes(b"")
        os.utime(lock_path, ns=(_OLD_NS, _OLD_NS))

        result = self.service.finalize(
            "maintenance",
            paths=[path],
            message="fix(tooling): recover abandoned git index lock",
            maintenance=True,
        )

        self.assertFalse(lock_path.exists())
        self.assertEqual(
            result.commit_sha,
            subprocess.run(
                ["git", "rev-parse", "HEAD"],
                cwd=self.repo,
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip(),
        )
        with self.database.connect() as connection:
            row = connection.execute(
                """
                SELECT session_id, payload_json FROM events
                WHERE event_type='git.index_lock_recovered'
                ORDER BY event_id DESC LIMIT 1
                """
            ).fetchone()
        self.assertIsNotNone(row)
        assert row is not None
        payload = json.loads(row["payload_json"])
        self.assertEqual("maintenance", row["session_id"])
        self.assertEqual(result.request_id, payload["request_id"])
        self.assertEqual(".git/index.lock", payload["lock_path"])
        self.assertEqual(0, payload["size"])


if __name__ == "__main__":
    unittest.main()
