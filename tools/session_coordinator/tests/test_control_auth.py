from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.control_plane.auth import WebControlAuth
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError


class ControlAuthTests(unittest.TestCase):
    def test_ticket_is_single_use_and_cookie_authenticates(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            auth = WebControlAuth(database)
            ticket = auth.issue_bootstrap_ticket("cli", "instance-a")

            raw_session, session = auth.consume_bootstrap_ticket(ticket, "instance-a")
            authenticated = auth.authenticate_cookie(
                f"zircon_control={raw_session}", "instance-a"
            )

            self.assertEqual("observer", session.role)
            self.assertEqual(session.session_id, authenticated.session_id)
            self.assertIn("HttpOnly", auth.cookie_header(raw_session))
            self.assertIn("SameSite=Strict", auth.cookie_header(raw_session))
            self.assertIn("Path=/control", auth.cookie_header(raw_session))
            self.assertIn("Max-Age=28800", auth.cookie_header(raw_session))
            with self.assertRaises(CoordinatorError) as replay:
                auth.consume_bootstrap_ticket(ticket, "instance-a")
            self.assertEqual("bootstrap_ticket_consumed", replay.exception.code)

    def test_wrong_daemon_instance_rejects_ticket_and_cookie(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            auth = WebControlAuth(database)
            ticket = auth.issue_bootstrap_ticket("cli", "instance-a")
            with self.assertRaises(CoordinatorError) as wrong_ticket:
                auth.consume_bootstrap_ticket(ticket, "instance-b")
            self.assertEqual("bootstrap_ticket_instance_mismatch", wrong_ticket.exception.code)

            raw_session, _session = auth.consume_bootstrap_ticket(ticket, "instance-a")
            with self.assertRaises(CoordinatorError) as wrong_cookie:
                auth.authenticate_cookie(
                    f"zircon_control={raw_session}", "instance-b"
                )
            self.assertEqual("web_session_instance_mismatch", wrong_cookie.exception.code)

    def test_expired_ticket_and_forged_cookie_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            auth = WebControlAuth(database)
            expired = auth.issue_bootstrap_ticket(
                "cli", "instance-a", ttl_seconds=-1
            )

            with self.assertRaises(CoordinatorError) as expired_error:
                auth.consume_bootstrap_ticket(expired, "instance-a")
            self.assertEqual("bootstrap_ticket_expired", expired_error.exception.code)
            with self.assertRaises(CoordinatorError) as forged_error:
                auth.authenticate_cookie(
                    "zircon_control=forged-value", "instance-a"
                )
            self.assertEqual("web_session_invalid", forged_error.exception.code)

            with database.connect() as connection:
                stored = connection.execute(
                    "SELECT ticket_hash FROM web_bootstrap_tickets"
                ).fetchone()[0]
            self.assertNotEqual(expired, stored)
            self.assertEqual(64, len(stored))


if __name__ == "__main__":
    unittest.main()
