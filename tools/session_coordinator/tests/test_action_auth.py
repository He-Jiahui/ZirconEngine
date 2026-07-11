from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.control_plane.auth import WebControlAuth
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError, WebControlRole


class ActionAuthTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.database = Database(Path(self.temporary_directory.name) / "coordinator.sqlite3")
        migrate(self.database)
        self.auth = WebControlAuth(self.database)
        ticket = self.auth.issue_bootstrap_ticket("cli", "instance-a")
        self.cookie, _ = self.auth.consume_bootstrap_ticket(ticket, "instance-a")

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def test_one_use_elevation_rotates_csrf_and_binds_session(self) -> None:
        grant = self.auth.issue_elevation_grant(
            "cli", WebControlRole.OPERATOR, "instance-a", bound_session_id="session-a"
        )
        csrf, elevated = self.auth.consume_elevation_grant(
            grant, f"zircon_control={self.cookie}", "instance-a"
        )

        self.assertEqual("operator", elevated.role)
        self.assertEqual("session-a", elevated.bound_session_id)
        self.auth.validate_csrf(
            f"zircon_control={self.cookie}", csrf, "instance-a"
        )
        with self.assertRaises(CoordinatorError) as replay:
            self.auth.consume_elevation_grant(
                grant, f"zircon_control={self.cookie}", "instance-a"
            )
        self.assertEqual("elevation_grant_consumed", replay.exception.code)

    def test_downgrade_cross_session_and_csrf_abuse_are_rejected(self) -> None:
        with self.assertRaises(CoordinatorError) as downgrade:
            self.auth.issue_elevation_grant(
                "cli", WebControlRole.OBSERVER, "instance-a"
            )
        self.assertEqual("elevation_role_invalid", downgrade.exception.code)

        grant = self.auth.issue_elevation_grant(
            "cli", WebControlRole.COMMITTER, "instance-a", bound_session_id="session-a"
        )
        csrf, elevated = self.auth.consume_elevation_grant(
            grant, f"zircon_control={self.cookie}", "instance-a"
        )
        with self.assertRaises(CoordinatorError) as cross_session:
            self.auth.require_bound_session(elevated, "session-b")
        self.assertEqual("web_session_scope_mismatch", cross_session.exception.code)
        for supplied in ("", "wrong"):
            with self.assertRaises(CoordinatorError) as csrf_error:
                self.auth.validate_csrf(
                    f"zircon_control={self.cookie}", supplied, "instance-a"
                )
            self.assertEqual("csrf_invalid", csrf_error.exception.code)
        self.auth.validate_csrf(f"zircon_control={self.cookie}", csrf, "instance-a")

    def test_expiry_actor_and_restart_invalidation_are_enforced(self) -> None:
        expired = self.auth.issue_elevation_grant(
            "cli", WebControlRole.OPERATOR, "instance-a", ttl_seconds=-1
        )
        with self.assertRaises(CoordinatorError) as expiry:
            self.auth.consume_elevation_grant(
                expired, f"zircon_control={self.cookie}", "instance-a"
            )
        self.assertEqual("elevation_grant_expired", expiry.exception.code)

        wrong_actor = self.auth.issue_elevation_grant(
            "tray", WebControlRole.OPERATOR, "instance-a"
        )
        with self.assertRaises(CoordinatorError) as actor:
            self.auth.consume_elevation_grant(
                wrong_actor, f"zircon_control={self.cookie}", "instance-a"
            )
        self.assertEqual("elevation_actor_mismatch", actor.exception.code)

        restart = self.auth.issue_elevation_grant(
            "cli", WebControlRole.OPERATOR, "instance-a"
        )
        with self.assertRaises(CoordinatorError) as instance:
            self.auth.consume_elevation_grant(
                restart, f"zircon_control={self.cookie}", "instance-b"
            )
        self.assertEqual("web_session_instance_mismatch", instance.exception.code)


if __name__ == "__main__":
    unittest.main()
