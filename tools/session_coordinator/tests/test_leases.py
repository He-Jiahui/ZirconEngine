from __future__ import annotations

import tempfile
import unittest
from datetime import UTC, datetime, timedelta
from pathlib import Path

from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import SessionStatus
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class LeaseTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        (self.repo / "a.txt").write_text("a\n", encoding="utf-8")
        (self.repo / "b.txt").write_text("b\n", encoding="utf-8")
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)
        self.sessions.register(session_id="session-a")
        self.sessions.register(session_id="session-b")
        self.service = LeaseService(self.database, PathPolicy(self.repo), ttl_seconds=5, grace_seconds=2)

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def test_multi_path_claim_is_atomic_and_case_insensitive(self) -> None:
        first = self.service.acquire("session-a", ["a.txt"])
        second = self.service.acquire("session-b", ["A.TXT", "b.txt"])

        self.assertTrue(first.acquired)
        self.assertFalse(second.acquired)
        self.assertEqual(("a.txt",), second.conflicts)
        self.assertEqual([], self.service.owned_paths("session-b"))

    def test_expired_lease_can_be_reclaimed_after_grace(self) -> None:
        now = datetime(2026, 7, 11, tzinfo=UTC)
        self.assertTrue(self.service.acquire("session-a", ["a.txt"], now=now).acquired)

        reclaimed = self.service.acquire(
            "session-b", ["a.txt"], now=now + timedelta(seconds=8)
        )

        self.assertTrue(reclaimed.acquired)
        self.assertEqual(["a.txt"], self.service.owned_paths("session-b"))

    def test_archived_session_cannot_reacquire_a_write_lease(self) -> None:
        self.sessions.set_status("session-a", SessionStatus.ACTIVE)
        self.sessions.set_status("session-a", SessionStatus.STALE)
        self.sessions.set_status("session-a", SessionStatus.ARCHIVED)

        with self.assertRaises(Exception):
            self.service.acquire("session-a", ["a.txt"])


if __name__ == "__main__":
    unittest.main()
