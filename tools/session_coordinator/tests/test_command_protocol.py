from __future__ import annotations

import json
import tempfile
import threading
import unittest
import urllib.error
import urllib.request
from datetime import UTC, datetime, timedelta
from pathlib import Path
from unittest import mock

from tools.session_coordinator.command_requests import (
    MAX_COMMAND_RESPONSE_BYTES,
    CommandRequestJournal,
)
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import LATEST_SCHEMA_VERSION, migrate
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.server import CoordinatorApplication, RunningCoordinator
from tools.session_coordinator.tests.helpers import init_repo


class CommandProtocolTests(unittest.TestCase):
    @staticmethod
    def _request(
        base_url: str, token: str, method: str, path: str, payload=None
    ) -> dict[str, object]:
        body = json.dumps(payload).encode("utf-8") if payload is not None else None
        request = urllib.request.Request(
            f"{base_url}{path}",
            data=body,
            headers={
                "Authorization": f"Bearer {token}",
                "Content-Type": "application/json",
            },
            method=method,
        )
        try:
            with urllib.request.urlopen(request, timeout=3) as response:
                status = response.status
                result = json.loads(response.read().decode("utf-8"))
        except urllib.error.HTTPError as error:
            try:
                status = error.code
                result = json.loads(error.read().decode("utf-8"))
            finally:
                error.close()
        if not isinstance(result, dict):
            raise AssertionError("coordinator returned a non-object response")
        result["_httpStatus"] = status
        return result

    def test_handler_durably_accepts_and_queries_request_id(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = CoordinatorConfig.for_repo(
                init_repo(root / "repo"), state_root=root / "state", port=0
            )
            request_id = "a" * 32
            with RunningCoordinator.start(config) as running:
                first = self._request(
                    running.base_url,
                    running.token,
                    "POST",
                    "/command",
                    {
                        "request_id": request_id,
                        "command": "session.register",
                        "arguments": {"session_id": "session-a"},
                    },
                )
                query = self._request(
                    running.base_url,
                    running.token,
                    "GET",
                    f"/command/requests/{request_id}",
                )

        self.assertEqual(200, first["_httpStatus"])
        self.assertEqual(200, query["_httpStatus"])
        self.assertEqual(request_id, first["requestId"])
        self.assertEqual("completed", query["request"]["status"])
        self.assertEqual("session-a", query["result"]["session"]["session_id"])

    def test_transactional_admission_commits_start_ack_before_scheduling(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            journal = CommandRequestJournal(database)
            request_id = "9" * 32
            scheduled: list[str] = []

            def admit(connection):
                accepted = connection.execute(
                    "SELECT status FROM command_requests WHERE request_id=?", (request_id,)
                ).fetchone()
                self.assertEqual("accepted", accepted["status"])
                connection.execute(
                    """
                    INSERT INTO cargo_start_requests(
                        request_id, reservation_id, job_id, session_id, command_json,
                        status, acknowledged_at, deadline_at
                    ) VALUES (?, 'reservation-a', 'job-a', 'session-a', '["cargo"]',
                              'start_pending', '2026-07-23T00:00:00+00:00',
                              '2026-07-23T00:15:00+00:00')
                    """,
                    (request_id,),
                )
                return {"start": {"status": "start_pending"}}, lambda: scheduled.append(
                    request_id
                )

            first = journal.execute_transactional(
                request_id, "cargo.run_reserved", {"job_id": "job-a"}, admit
            )
            duplicate = journal.execute_transactional(
                request_id,
                "cargo.run_reserved",
                {"job_id": "job-a"},
                lambda _connection: self.fail("duplicate request must not be admitted again"),
            )

            with database.connect() as connection:
                statuses = connection.execute(
                    """
                    SELECT command.status, start.status
                    FROM command_requests AS command
                    JOIN cargo_start_requests AS start USING(request_id)
                    WHERE command.request_id=?
                    """,
                    (request_id,),
                ).fetchone()
            self.assertEqual(("completed", "start_pending"), tuple(statuses))
            self.assertEqual(first, duplicate)
            self.assertEqual([request_id], scheduled)

    def test_transactional_admission_rolls_back_partial_start_and_persists_failure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            journal = CommandRequestJournal(database)
            request_id = "8" * 32

            def fail_after_start(connection):
                connection.execute(
                    """
                    INSERT INTO cargo_start_requests(
                        request_id, reservation_id, job_id, session_id, command_json,
                        status, acknowledged_at, deadline_at
                    ) VALUES (?, 'reservation-b', 'job-b', 'session-a', '["cargo"]',
                              'start_pending', '2026-07-23T00:00:00+00:00',
                              '2026-07-23T00:15:00+00:00')
                    """,
                    (request_id,),
                )
                raise CoordinatorError("admission_failed", "simulated admission failure")

            with self.assertRaises(CoordinatorError):
                journal.execute_transactional(
                    request_id, "cargo.run_reserved", {"job_id": "job-b"}, fail_after_start
                )

            state = journal.get(request_id)
            self.assertEqual("failed", state["request"]["status"])
            with database.connect() as connection:
                start_count = connection.execute(
                    "SELECT COUNT(*) FROM cargo_start_requests WHERE request_id=?",
                    (request_id,),
                ).fetchone()[0]
            self.assertEqual(0, start_count)

    def test_completed_reserved_request_replays_without_current_admission_gate(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = CoordinatorConfig.for_repo(
                init_repo(root / "repo"), state_root=root / "state", port=0
            )
            request_id = "7" * 32
            arguments = {"job_id": "job-a"}
            with RunningCoordinator.start(config) as running:
                application = running.httpd.application
                expected = application.command_requests.execute(
                    request_id,
                    "cargo.run_reserved",
                    arguments,
                    lambda: {"start": {"status": "start_pending"}},
                )
                with mock.patch.object(
                    application.supervision,
                    "require_mutation_allowed_in_connection",
                    side_effect=AssertionError("completed replay must not re-enter admission"),
                ):
                    replay = application.execute_command_request(
                        "cargo.run_reserved", arguments, request_id=request_id
                    )

        self.assertEqual(expected, replay)

    def test_reserved_http_admission_preserves_read_only_branch_guard(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = CoordinatorConfig.for_repo(
                init_repo(root / "repo"), state_root=root / "state", port=0
            )
            request_id = "6" * 32
            with RunningCoordinator.start(config) as running:
                application = running.httpd.application
                with (
                    mock.patch.object(
                        type(application),
                        "read_only",
                        new_callable=mock.PropertyMock,
                        return_value=True,
                    ),
                    self.assertRaises(CoordinatorError) as rejected,
                ):
                    application.execute_command_request("cargo.run_reserved", {}, request_id=request_id)
                persisted = application.command_requests.get(request_id)

        self.assertEqual("not_on_main", rejected.exception.code)
        self.assertEqual("failed", persisted["request"]["status"])

    def test_startup_reconciliation_terminalizes_interrupted_generic_request(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            request_id = "5" * 32
            with database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO command_requests(
                        request_id, command, arguments_hash, status, received_at, accepted_at
                    ) VALUES (?, 'baseline.scan', ?, 'accepted', 'now', 'now')
                    """,
                    (request_id, "0" * 64),
                )
            journal = CommandRequestJournal(database)

            reconciled = journal.reconcile_interrupted()
            persisted = journal.get(request_id)

        self.assertEqual((request_id,), reconciled)
        self.assertEqual("failed", persisted["request"]["status"])
        self.assertEqual("command_execution_interrupted", persisted["error"]["code"])

    def test_large_response_uses_bounded_stable_completion_tombstone(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            journal = CommandRequestJournal(database)
            request_id = "4" * 32
            arguments: dict[str, object] = {}

            first = journal.execute(
                request_id,
                "control.snapshot",
                arguments,
                lambda: {"payload": "x" * (MAX_COMMAND_RESPONSE_BYTES + 1)},
            )
            duplicate = journal.execute(
                request_id,
                "control.snapshot",
                arguments,
                lambda: self.fail("large response duplicate must not re-execute"),
            )
            with database.connect() as connection:
                stored_bytes = len(
                    connection.execute(
                        "SELECT response_json FROM command_requests WHERE request_id=?",
                        (request_id,),
                    ).fetchone()[0].encode("utf-8")
                )
            query = journal.get(request_id)
            response_sha = duplicate["responseSha256"]
            journal.prune(
                now=datetime.now(UTC) + timedelta(days=8),
                retention_days=7,
                max_terminal=10,
            )
            compacted_query = journal.get(request_id)

        self.assertGreater(len(first["payload"]), MAX_COMMAND_RESPONSE_BYTES)
        self.assertTrue(duplicate["responseOmitted"])
        self.assertGreater(duplicate["responseBytes"], MAX_COMMAND_RESPONSE_BYTES)
        self.assertEqual(64, len(duplicate["responseSha256"]))
        self.assertLess(stored_bytes, 1024)
        self.assertEqual(duplicate, query["result"])
        self.assertEqual(response_sha, compacted_query["result"]["responseSha256"])
        self.assertTrue(compacted_query["result"]["responseOmitted"])
        self.assertNotIn("responseExpired", compacted_query["result"])

    def test_large_failure_preserves_original_digest_across_compaction(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            journal = CommandRequestJournal(database)
            request_id = "5" * 32

            def fail() -> dict[str, object]:
                raise CoordinatorError(
                    "large_failure",
                    "large command failure",
                    details={"payload": "x" * (MAX_COMMAND_RESPONSE_BYTES + 1)},
                )

            with self.assertRaises(CoordinatorError):
                journal.execute(request_id, "baseline.scan", {}, fail)
            before = journal.get(request_id)["error"]
            pruned = journal.prune(
                now=datetime.now(UTC) + timedelta(days=8),
                retention_days=7,
                max_terminal=10,
            )
            after = journal.get(request_id)["error"]
            pruned_again = journal.prune(
                now=datetime.now(UTC) + timedelta(days=8),
                retention_days=7,
                max_terminal=10,
            )
            with database.connect() as connection:
                compacted_at = connection.execute(
                    "SELECT payload_compacted_at FROM command_requests WHERE request_id=?",
                    (request_id,),
                ).fetchone()[0]

        self.assertTrue(before["details"]["detailsOmitted"])
        self.assertEqual(64, len(before["details"]["errorSha256"]))
        self.assertEqual(before, after)
        self.assertEqual(1, pruned)
        self.assertEqual(0, pruned_again)
        self.assertIsNotNone(compacted_at)

    def test_terminal_request_retention_never_deletes_accepted_rows(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            journal = CommandRequestJournal(database)
            with database.transaction() as connection:
                connection.executemany(
                    """
                    INSERT INTO command_requests(
                        request_id, command, arguments_hash, status,
                        received_at, accepted_at, completed_at, response_json
                    ) VALUES (?, 'watch.scan', ?, ?, ?, ?, ?, ?)
                    """,
                    (
                        ("1" * 32, "1" * 64, "completed", "old", "old", "2026-07-01T00:00:00+00:00", "{}"),
                        ("2" * 32, "2" * 64, "completed", "new", "new", "2026-07-22T00:00:00+00:00", "{}"),
                        ("3" * 32, "3" * 64, "accepted", "old", "old", None, None),
                    ),
                )

            pruned = journal.prune(
                now=datetime(2026, 7, 23, tzinfo=UTC), retention_days=7, max_terminal=10
            )
            pruned_again = journal.prune(
                now=datetime(2026, 7, 23, tzinfo=UTC), retention_days=7, max_terminal=10
            )
            old = journal.get("1" * 32)
            with database.connect() as connection:
                compacted_at = connection.execute(
                    "SELECT payload_compacted_at FROM command_requests WHERE request_id=?",
                    ("1" * 32,),
                ).fetchone()[0]
                remaining = tuple(
                    row[0]
                    for row in connection.execute(
                        "SELECT request_id FROM command_requests ORDER BY request_id"
                    )
                )
                query_plan = tuple(
                    row[3]
                    for row in connection.execute(
                        """
                        EXPLAIN QUERY PLAN
                        SELECT * FROM command_requests
                        WHERE retention_class='durable'
                          AND status IN ('completed', 'failed')
                          AND payload_compacted_at IS NULL
                          AND completed_at IS NOT NULL AND completed_at<?
                        """,
                        ("2026-07-16T00:00:00+00:00",),
                    )
                )
                compaction_index_sql = connection.execute(
                    """
                    SELECT sql FROM sqlite_master
                    WHERE type='index' AND name='command_requests_compaction_candidates'
                    """
                ).fetchone()[0]

        self.assertEqual(1, pruned)
        self.assertEqual(0, pruned_again)
        self.assertEqual(("1" * 32, "2" * 32, "3" * 32), remaining)
        self.assertIsNotNone(compacted_at)
        self.assertTrue(old["result"]["responseExpired"])
        self.assertTrue(
            any("command_requests_compaction_candidates" in detail for detail in query_plan)
        )
        self.assertIn("WHERE retention_class='durable'", compaction_index_sql)
        self.assertIn("payload_compacted_at IS NULL", compaction_index_sql)

    def test_high_frequency_request_keys_are_bounded_and_cargo_start_stays_coherent(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            journal = CommandRequestJournal(database)
            ephemeral_ids = tuple(f"{index:032x}" for index in range(1, 6))
            for request_id in ephemeral_ids:
                journal.execute(
                    request_id,
                    "session.list",
                    {},
                    lambda: {"sessions": []},
                    retention_class="ephemeral",
                )
            durable_id = "d" * 32
            durable_first = journal.execute(
                durable_id,
                "cargo.run_reserved",
                {"job_id": "job-d"},
                lambda: {"start": {"status": "start_pending"}},
            )
            with database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO cargo_start_requests(
                        request_id, reservation_id, job_id, session_id, command_json,
                        status, acknowledged_at, deadline_at
                    ) VALUES (?, 'reservation-d', 'job-d', 'session-d', '["cargo"]',
                              'start_pending', 'now', 'later')
                    """,
                    (durable_id,),
                )
                protected_ephemeral_id = "e" * 32
                connection.execute(
                    """
                    INSERT INTO command_requests(
                        request_id, command, arguments_hash, status, received_at,
                        accepted_at, completed_at, response_json, retention_class
                    ) VALUES (?, 'session.list', ?, 'completed', 'old', 'old',
                              '2026-07-01T00:00:00+00:00', '{}', 'ephemeral')
                    """,
                    (protected_ephemeral_id, "e" * 64),
                )
                connection.execute(
                    """
                    INSERT INTO cargo_start_requests(
                        request_id, reservation_id, job_id, session_id, command_json,
                        status, acknowledged_at, deadline_at
                    ) VALUES (?, 'reservation-e', 'job-e', 'session-e', '["cargo"]',
                              'start_pending', 'now', 'later')
                    """,
                    (protected_ephemeral_id,),
                )

            journal.prune(
                now=datetime.now(UTC),
                retention_days=365,
                max_terminal=100,
                ephemeral_retention_days=365,
                max_ephemeral=2,
            )
            journal.prune(
                now=datetime.now(UTC) + timedelta(days=8),
                retention_days=1,
                max_terminal=1,
                ephemeral_retention_days=365,
                max_ephemeral=2,
            )
            durable = journal.get(durable_id)
            durable_duplicate = journal.execute(
                durable_id,
                "cargo.run_reserved",
                {"job_id": "job-d"},
                lambda: self.fail("durable start replay must not execute again"),
            )
            protected_ephemeral = journal.get(protected_ephemeral_id)
            with database.connect() as connection:
                durable_compacted_at = connection.execute(
                    "SELECT payload_compacted_at FROM command_requests WHERE request_id=?",
                    (durable_id,),
                ).fetchone()[0]
                remaining_ephemeral = tuple(
                    row[0]
                    for row in connection.execute(
                        """
                        SELECT request_id FROM command_requests
                        WHERE retention_class='ephemeral' ORDER BY request_id
                        """
                    )
                )

        self.assertEqual(
            tuple(sorted((*ephemeral_ids[-2:], protected_ephemeral_id))),
            remaining_ephemeral,
        )
        self.assertEqual("start_pending", durable["start"]["status"])
        self.assertEqual(durable_first, durable_duplicate)
        self.assertIsNone(durable_compacted_at)
        self.assertEqual("start_pending", protected_ephemeral["start"]["status"])

    def test_server_classifies_list_and_heartbeats_as_ephemeral_request_keys(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = CoordinatorConfig.for_repo(
                init_repo(root / "repo"), state_root=root / "state", port=0
            )
            with RunningCoordinator.start(config) as running:
                application = running.httpd.application
                application.execute_command_request(
                    "session.register",
                    {"session_id": "session-a"},
                    request_id="a" * 32,
                )
                application.execute_command_request(
                    "session.list", {}, request_id="b" * 32
                )
                application.execute_command_request(
                    "session.heartbeat",
                    {"session_id": "session-a"},
                    request_id="c" * 32,
                )
                with application.database.connect() as connection:
                    classes = {
                        row["command"]: row["retention_class"]
                        for row in connection.execute(
                            """
                            SELECT command, retention_class FROM command_requests
                            WHERE request_id IN (?, ?, ?)
                            """,
                            ("a" * 32, "b" * 32, "c" * 32),
                        )
                    }

        self.assertEqual("durable", classes["session.register"])
        self.assertEqual("ephemeral", classes["session.list"])
        self.assertEqual("ephemeral", classes["session.heartbeat"])
        self.assertNotIn("cleanup.plan", CoordinatorApplication.BOUNDED_REQUEST_KEY_COMMANDS)

    def test_prune_has_a_fixed_per_tick_budget_and_converges(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            journal = CommandRequestJournal(database)
            with database.transaction() as connection:
                rows = []
                for index in range(20):
                    rows.append(
                        (
                            f"{index + 1:032x}",
                            f"{index + 1:064x}",
                            "ephemeral",
                            "2026-07-22T00:00:00+00:00",
                        )
                    )
                for index in range(20, 40):
                    rows.append(
                        (
                            f"{index + 1:032x}",
                            f"{index + 1:064x}",
                            "durable",
                            "2026-07-01T00:00:00+00:00",
                        )
                    )
                connection.executemany(
                    """
                    INSERT INTO command_requests(
                        request_id, command, arguments_hash, status, received_at,
                        accepted_at, completed_at, response_json, retention_class
                    ) VALUES (?, 'session.list', ?, 'completed', 'received', 'accepted',
                              ?, '{}', ?)
                    """,
                    ((request_id, digest, completed_at, retention) for request_id, digest, retention, completed_at in rows),
                )

            changes: list[int] = []
            for _ in range(20):
                changed = journal.prune(
                    now=datetime(2026, 7, 23, tzinfo=UTC),
                    retention_days=7,
                    max_terminal=100,
                    ephemeral_retention_days=365,
                    max_ephemeral=2,
                    batch_size=5,
                )
                changes.append(changed)
                if changed == 0:
                    break
            with database.connect() as connection:
                ephemeral_count = connection.execute(
                    "SELECT COUNT(*) FROM command_requests WHERE retention_class='ephemeral'"
                ).fetchone()[0]
                durable_uncompacted = connection.execute(
                    """
                    SELECT COUNT(*) FROM command_requests
                    WHERE retention_class='durable' AND payload_compacted_at IS NULL
                    """
                ).fetchone()[0]

        self.assertTrue(changes)
        self.assertTrue(all(0 <= changed <= 5 for changed in changes))
        self.assertEqual(0, changes[-1])
        self.assertEqual(2, ephemeral_count)
        self.assertEqual(0, durable_uncompacted)

    def test_request_is_queryable_while_handler_is_still_executing(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = CoordinatorConfig.for_repo(
                init_repo(root / "repo"), state_root=root / "state", port=0
            )
            request_id = "c" * 32
            entered = threading.Event()
            release = threading.Event()
            response: dict[str, object] = {}

            with RunningCoordinator.start(config) as running:
                application = running.httpd.application

                def slow_command(*_args, **_kwargs):
                    entered.set()
                    release.wait(timeout=2)
                    return {"status": "done"}

                def post() -> None:
                    response.update(
                        self._request(
                            running.base_url,
                            running.token,
                            "POST",
                            "/command",
                            {
                                "request_id": request_id,
                                "command": "watch.scan",
                                "arguments": {},
                            },
                        )
                    )

                with mock.patch.object(application, "command", side_effect=slow_command):
                    worker = threading.Thread(target=post, daemon=True)
                    worker.start()
                    try:
                        self.assertTrue(entered.wait(timeout=1))
                        query = self._request(
                            running.base_url,
                            running.token,
                            "GET",
                            f"/command/requests/{request_id}",
                        )
                    finally:
                        release.set()
                        worker.join(timeout=2)

        self.assertEqual(200, query["_httpStatus"])
        self.assertEqual("accepted", query["request"]["status"])
        self.assertEqual("done", response["status"])

    def test_schema_49_upgrade_preserves_terminal_incident_rows(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            with mock.patch(
                "tools.session_coordinator.migrations.LATEST_SCHEMA_VERSION", 49
            ):
                self.assertEqual(49, migrate(database))
            with database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO sessions(
                        session_id, status, created_at, updated_at, last_heartbeat_at
                    ) VALUES ('incident-owner', 'archived', 'now', 'now', 'now')
                    """
                )
                connection.execute(
                    """
                    INSERT INTO cargo_jobs(
                        job_id, session_id, lane_kind, target_dir, status,
                        created_at, last_heartbeat_at
                    ) VALUES (
                        '9b9a39441aaf43e4a3b7df878268f2de', 'incident-owner', 'test',
                        'D:\\cargo-targets\\incident', 'orphaned',
                        '2026-07-23T03:38:16.573418+00:00',
                        '2026-07-23T03:43:43.822692+00:00'
                    )
                    """
                )
                for reservation_id, job_id in (
                    ("65ce472dd30245d9b1a844ee5a352496", None),
                    (
                        "f021112ff2a4423399bb2ee808c71555",
                        "9b9a39441aaf43e4a3b7df878268f2de",
                    ),
                ):
                    connection.execute(
                        """
                        INSERT INTO cargo_lane_reservations(
                            reservation_id, session_id, lane_scope, compatibility_key,
                            command_fingerprint, job_id, status, created_at, expires_at,
                            completed_at
                        ) VALUES (?, 'incident-owner', 'cpu', 'compatibility',
                                  'command', ?, 'expired', 'created', 'expires', 'completed')
                        """,
                        (reservation_id, job_id),
                    )
            with database.connect() as connection:
                before = [
                    tuple(row)
                    for row in connection.execute(
                        """
                        SELECT reservation_id, job_id, status, completed_at
                        FROM cargo_lane_reservations
                        WHERE reservation_id IN (
                            '65ce472dd30245d9b1a844ee5a352496',
                            'f021112ff2a4423399bb2ee808c71555'
                        ) ORDER BY reservation_id
                        """
                    )
                ]

            self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))

            with database.connect() as connection:
                after = [
                    tuple(row)
                    for row in connection.execute(
                        """
                        SELECT reservation_id, job_id, status, completed_at
                        FROM cargo_lane_reservations
                        WHERE reservation_id IN (
                            '65ce472dd30245d9b1a844ee5a352496',
                            'f021112ff2a4423399bb2ee808c71555'
                        ) ORDER BY reservation_id
                        """
                    )
                ]
                job = tuple(
                    connection.execute(
                        """
                        SELECT status, pid, command_json, started_at, exit_code
                        FROM cargo_jobs WHERE job_id='9b9a39441aaf43e4a3b7df878268f2de'
                        """
                    ).fetchone()
                )
                schema_objects = {
                    row[0]
                    for row in connection.execute(
                        """
                        SELECT name FROM sqlite_master
                        WHERE name IN (
                            'command_requests', 'cargo_start_requests',
                            'cargo_jobs_preserve_valid_start_pending'
                        )
                        """
                    )
                }
                request_columns = {
                    row[1] for row in connection.execute("PRAGMA table_info(command_requests)")
                }
                request_indexes = {
                    row[1] for row in connection.execute("PRAGMA index_list(command_requests)")
                }
        self.assertEqual(before, after)
        self.assertEqual(("orphaned", None, "[]", None, None), job)
        self.assertEqual(
            {
                "command_requests",
                "cargo_start_requests",
                "cargo_jobs_preserve_valid_start_pending",
            },
            schema_objects,
        )
        self.assertIn("payload_compacted_at", request_columns)
        self.assertIn("retention_class", request_columns)
        self.assertIn("command_requests_compaction_candidates", request_indexes)
        self.assertIn("command_requests_ephemeral_retention", request_indexes)


if __name__ == "__main__":
    unittest.main()
