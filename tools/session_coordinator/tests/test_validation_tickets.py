from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.validation_tickets import ValidationTicketService


class ValidationTicketTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.database = Database(Path(self.temporary.name) / "coordinator.sqlite3")
        migrate(self.database)
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO sessions(
                    session_id, plan_path, status, created_at, updated_at, last_heartbeat_at
                ) VALUES (
                    'primary', 'docs/plans/tooling/01-tooling.md', 'active',
                    '2026-07-31T00:00:00+00:00', '2026-07-31T00:00:00+00:00',
                    '2026-07-31T00:00:00+00:00'
                )
                """
            )
        self.service = ValidationTicketService(self.database)

    def test_submit_returns_a_durable_receipt_without_blocking_the_owner_session(self) -> None:
        first = self.service.submit(
            session_id="primary",
            request_id="request-a",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused", "scope": ["governance"]},
        )
        replay = self.service.submit(
            session_id="primary",
            request_id="request-a",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused", "scope": ["governance"]},
        )
        merged = self.service.submit(
            session_id="primary",
            request_id="request-b",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused", "scope": ["governance"]},
        )

        self.assertEqual("queued", first.ticket.status)
        self.assertFalse(first.reused)
        self.assertEqual(first, replay)
        self.assertEqual(first.ticket.ticket_id, merged.ticket.ticket_id)
        self.assertTrue(merged.reused)
        with self.database.connect() as connection:
            owner_status = connection.execute(
                "SELECT status FROM sessions WHERE session_id='primary'"
            ).fetchone()["status"]
            request_count = connection.execute(
                "SELECT COUNT(*) FROM validation_ticket_requests"
            ).fetchone()[0]
        self.assertEqual("active", owner_status)
        self.assertEqual(2, request_count)

    def test_different_source_manifest_is_not_merged(self) -> None:
        first = self.service.submit(
            session_id="primary",
            request_id="request-a",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused"},
        )
        second = self.service.submit(
            session_id="primary",
            request_id="request-b",
            source_manifest={"tools/session_coordinator/governance.py": "b" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused"},
        )

        self.assertNotEqual(first.ticket.ticket_id, second.ticket.ticket_id)
        self.assertFalse(second.reused)

    def test_terminal_result_can_be_recorded_without_polling_for_running(self) -> None:
        receipt = self.service.submit(
            session_id="primary",
            request_id="request-a",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused"},
        )

        completed = self.service.record_result(
            receipt.ticket.ticket_id,
            "failed",
            evidence={"exitCode": 1},
        )

        self.assertEqual("failed", completed.status)
        with self.database.connect() as connection:
            owner_status = connection.execute(
                "SELECT status FROM sessions WHERE session_id='primary'"
            ).fetchone()["status"]
        self.assertEqual("active", owner_status)


if __name__ == "__main__":
    unittest.main()
