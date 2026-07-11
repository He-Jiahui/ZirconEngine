from __future__ import annotations

import tempfile
import unittest
from contextlib import ExitStack
from pathlib import Path

from tools.session_coordinator.control_plane.events import EventStreamService
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError


class ControlEventTests(unittest.TestCase):
    def test_events_replay_after_cursor_in_order(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            with database.transaction() as connection:
                for number in range(3):
                    connection.execute(
                        "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, 'now')",
                        (f"event.{number}", f'{{"number": {number}}}'),
                    )
            service = EventStreamService(database)

            replay = service.read_after(1)

            self.assertEqual([2, 3], [item.event_id for item in replay.events])
            self.assertFalse(replay.resync_required)
            self.assertIn("id: 2", service.encode(replay.events[0]))

    def test_ninth_client_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            service = EventStreamService(database)
            with ExitStack() as clients:
                for _index in range(8):
                    clients.enter_context(service.client_slot())
                with self.assertRaises(CoordinatorError) as rejected:
                    with service.client_slot():
                        pass
            self.assertEqual("sse_capacity", rejected.exception.code)

    def test_stale_and_future_cursors_require_resync(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            with database.transaction() as connection:
                for number in range(4):
                    connection.execute(
                        "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, '{}', 'now')",
                        (f"event.{number}",),
                    )
            service = EventStreamService(database, replay_capacity=2)

            self.assertTrue(service.read_after(1).resync_required)
            self.assertTrue(service.read_after(99).resync_required)
            self.assertEqual(
                [3, 4], [event.event_id for event in service.read_after(2).events]
            )


if __name__ == "__main__":
    unittest.main()
