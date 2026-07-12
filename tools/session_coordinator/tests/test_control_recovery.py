from __future__ import annotations

import json
import sqlite3
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.session_coordinator.client import CoordinatorClient, CoordinatorClientError
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.control_plane.events import EventStreamService
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import LATEST_SCHEMA_VERSION, MIGRATIONS, migrate
from tools.session_coordinator.models import utc_text
from tools.session_coordinator.server import RunningCoordinator
from tools.session_coordinator.tests.helpers import init_repo


class ControlRecoveryTests(unittest.TestCase):
    def test_v13_upgrade_preserves_domain_rows(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            self._migrate_through(database, 13)
            self._insert_v13_domain_rows(database)

            self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))

            with database.connect() as connection:
                version = connection.execute(
                    "SELECT MAX(version) FROM schema_version"
                ).fetchone()[0]
                session = connection.execute(
                    "SELECT status, status_reason FROM sessions WHERE session_id='upgrade-session'"
                ).fetchone()
                event = connection.execute(
                    "SELECT event_type, payload_json FROM events WHERE session_id='upgrade-session'"
                ).fetchone()
            self.assertEqual(LATEST_SCHEMA_VERSION, version)
            self.assertEqual(("active", "preserve me"), tuple(session))
            self.assertEqual("upgrade.fixture", event["event_type"])
            self.assertEqual({"preserved": True}, json.loads(event["payload_json"]))

    def test_injected_migration_failure_rolls_back_without_repair_sql(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            self._migrate_through(database, 13)
            self._insert_v13_domain_rows(database)

            def fail_after_write(connection) -> None:
                connection.execute("CREATE TABLE injected_partial(value TEXT)")
                raise sqlite3.OperationalError("injected migration failure")

            with patch.dict(MIGRATIONS, {14: fail_after_write}):
                with self.assertRaisesRegex(sqlite3.OperationalError, "injected"):
                    migrate(database)

            with database.connect() as connection:
                versions = [
                    int(row[0])
                    for row in connection.execute(
                        "SELECT version FROM schema_version ORDER BY version"
                    )
                ]
                partial = connection.execute(
                    "SELECT COUNT(*) FROM sqlite_master WHERE name='injected_partial'"
                ).fetchone()[0]
                preserved = connection.execute(
                    "SELECT status_reason FROM sessions WHERE session_id='upgrade-session'"
                ).fetchone()[0]
            self.assertEqual(list(range(1, 14)), versions)
            self.assertEqual(0, partial)
            self.assertEqual("preserve me", preserved)

    def test_startup_migration_failure_emits_sanitized_fatal_diagnostic(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            database = Database(config.database_path)
            self._migrate_through(database, 13)

            def fail_migration(_connection) -> None:
                raise sqlite3.OperationalError("secret migration detail")

            with patch.dict(MIGRATIONS, {14: fail_migration}):
                with self.assertRaises(sqlite3.OperationalError):
                    RunningCoordinator.start(config)

            diagnostic = json.loads(
                (config.state_root / "startup-failure.json").read_text(encoding="utf-8")
            )
            self.assertEqual("migration_or_integrity_failure", diagnostic["kind"])
            self.assertEqual("OperationalError", diagnostic["errorType"])
            self.assertNotIn("secret migration detail", json.dumps(diagnostic))
            self.assertFalse(config.runtime_path.exists())
            self.assertFalse(config.lock_path.exists())

    def test_restart_preserves_event_continuity_and_invalidates_old_runtime(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(
                repo,
                state_root=root / "state",
                port=0,
                watch_interval_seconds=60,
                maintenance_interval_seconds=60,
            )
            with RunningCoordinator.start(config) as first:
                first_client = CoordinatorClient.from_runtime(config)
                first_client.command("session.register", {"session_id": "restart-session"})
                cursor = int(first_client.control_snapshot()["eventCursor"])
                first_instance = first.instance_id

            with self.assertRaises(CoordinatorClientError):
                first_client.health()

            with RunningCoordinator.start(config) as second:
                second_client = CoordinatorClient.from_runtime(config)
                second_client.command(
                    "session.set_status",
                    {
                        "session_id": "restart-session",
                        "status": "active",
                        "reason": "successor recovered",
                    },
                )
                self.assertNotEqual(first_instance, second.instance_id)
                replay = EventStreamService(Database(config.database_path)).read_after(cursor)

            self.assertFalse(replay.resync_required)
            self.assertTrue(replay.events)
            self.assertGreater(replay.events[0].event_id, cursor)

    def test_force_stop_terminal_and_offline_proof_remain_readable_until_ack(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(
                repo,
                state_root=root / "state",
                port=0,
                watch_interval_seconds=60,
                maintenance_interval_seconds=60,
            )
            running = RunningCoordinator.start(config)
            try:
                client = CoordinatorClient.from_runtime(config)
                preview = client.control_request(
                    "POST",
                    "/control/v1/actions/preview",
                    {
                        "kind": "service.force_stop",
                        "parameters": {"timeoutSeconds": 5},
                    },
                )["action"]
                confirmed = client.control_request(
                    "POST",
                    f"/control/v1/actions/{preview['actionId']}/confirm",
                    {
                        "phrase": preview["confirmationPhrase"],
                        "reason": "integration force-stop handshake",
                    },
                )["action"]
                deadline = time.monotonic() + 2
                terminal = confirmed
                state = client.health()["supervision"]["state"]
                while (
                    terminal["status"] != "succeeded" or state != "offline"
                ) and time.monotonic() < deadline:
                    terminal = client.control_request(
                        "GET", f"/control/v1/actions/{preview['actionId']}"
                    )["action"]
                    state = client.health()["supervision"]["state"]
                    time.sleep(0.01)

                self.assertEqual("succeeded", terminal["status"])
                self.assertEqual("offline", state)
                self.assertTrue(running.thread.is_alive())
                ack = client.command(
                    "supervision.force_stop_ack", {"actionId": preview["actionId"]}
                )
                self.assertTrue(ack["acknowledged"])
                running.thread.join(timeout=2)
                self.assertFalse(running.thread.is_alive())
            finally:
                running.stop()

    @staticmethod
    def _migrate_through(database: Database, version: int) -> None:
        with database.transaction() as connection:
            connection.execute(
                "CREATE TABLE schema_version (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)"
            )
            for current in range(1, version + 1):
                MIGRATIONS[current](connection)
                connection.execute(
                    "INSERT INTO schema_version(version, applied_at) VALUES (?, ?)",
                    (current, utc_text()),
                )

    @staticmethod
    def _insert_v13_domain_rows(database: Database) -> None:
        now = utc_text()
        with database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO sessions(
                    session_id, display_name, plan_path, status, status_reason,
                    base_head, write_scope_json, created_at, updated_at,
                    last_heartbeat_at
                ) VALUES ('upgrade-session', 'Upgrade Session', NULL, 'active',
                          'preserve me', '', '[]', ?, ?, ?)
                """,
                (now, now, now),
            )
            connection.execute(
                """
                INSERT INTO events(session_id, event_type, payload_json, created_at)
                VALUES ('upgrade-session', 'upgrade.fixture', '{"preserved": true}', ?)
                """,
                (now,),
            )


if __name__ == "__main__":
    unittest.main()
