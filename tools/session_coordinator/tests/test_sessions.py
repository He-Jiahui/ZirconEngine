from __future__ import annotations

import tempfile
import unittest
from datetime import timedelta
from pathlib import Path

from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import (
    InvalidStatusTransition,
    SessionStatus,
    utc_now,
    utc_text,
)
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class SessionServiceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        self.service = SessionService(self.database, self.repo)

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def test_register_uses_enum_status_and_current_head(self) -> None:
        session = self.service.register(
            session_id="session-a",
            display_name="M1 test",
            plan_path="docs/superpowers/plans/coordinator.md",
            write_scope=["tools/session_coordinator"],
        )

        self.assertEqual(SessionStatus.REGISTERED, session.status)
        self.assertEqual(40, len(session.base_head))
        self.assertEqual(("tools/session_coordinator",), session.write_scope)

    def test_legal_transitions_and_heartbeat_are_persisted(self) -> None:
        self.service.register(session_id="session-a")
        active = self.service.set_status("session-a", SessionStatus.ACTIVE)
        before = active.last_heartbeat_at
        heartbeat = self.service.heartbeat("session-a")
        completed = self.service.set_status("session-a", SessionStatus.COMPLETED)

        self.assertGreaterEqual(heartbeat.last_heartbeat_at, before)
        self.assertEqual(SessionStatus.COMPLETED, completed.status)

    def test_invalid_transition_is_rejected_without_mutation(self) -> None:
        self.service.register(session_id="session-a")

        with self.assertRaises(InvalidStatusTransition):
            self.service.set_status("session-a", SessionStatus.ARCHIVED)

        self.assertEqual(SessionStatus.REGISTERED, self.service.get("session-a").status)

    def test_free_form_status_is_not_accepted(self) -> None:
        self.service.register(session_id="session-a")

        with self.assertRaises(ValueError):
            self.service.set_status("session-a", "almost done")  # type: ignore[arg-type]

    def test_mark_stale_is_atomic_and_preserves_last_heartbeat(self) -> None:
        self.service.register(session_id="expired")
        self.service.register(session_id="live")
        self.service.set_status("expired", SessionStatus.ACTIVE)
        self.service.set_status("live", SessionStatus.ACTIVE)
        expired_heartbeat = utc_text(utc_now() - timedelta(hours=2))
        live_heartbeat = utc_text(utc_now())
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET last_heartbeat_at = ? WHERE session_id = 'expired'",
                (expired_heartbeat,),
            )
            connection.execute(
                "UPDATE sessions SET last_heartbeat_at = ? WHERE session_id = 'live'",
                (live_heartbeat,),
            )

        marked = self.service.mark_stale(older_than_seconds=60)

        self.assertEqual(["expired"], marked)
        expired = self.service.get("expired")
        live = self.service.get("live")
        self.assertEqual(SessionStatus.STALE, expired.status)
        self.assertEqual(expired_heartbeat, utc_text(expired.last_heartbeat_at))
        self.assertEqual(SessionStatus.ACTIVE, live.status)
        self.assertEqual(live_heartbeat, utc_text(live.last_heartbeat_at))


if __name__ == "__main__":
    unittest.main()
