from __future__ import annotations

import hashlib
import json
import sqlite3
import tempfile
import threading
import unittest
import urllib.error
import urllib.request
from contextlib import contextmanager
from pathlib import Path
from unittest import mock

from tools.session_coordinator.command_requests import CommandRequestJournal
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.server import RunningCoordinator
from tools.session_coordinator.tests.failure_fixture import FailureGraphFixture
from tools.session_coordinator.tests.helpers import init_repo


class SessionRegisterDurabilityTests(unittest.TestCase):
    @staticmethod
    def _request(
        base_url: str,
        token: str,
        method: str,
        path: str,
        payload: dict[str, object] | None = None,
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
            with urllib.request.urlopen(request, timeout=5) as response:
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

    @staticmethod
    def _payload(request_id: str, session_id: str, **arguments) -> dict[str, object]:
        return {
            "request_id": request_id,
            "command": "session.register",
            "arguments": {"session_id": session_id, **arguments},
        }

    def test_two_registration_requests_are_terminal_and_admit_a_lease(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = CoordinatorConfig.for_repo(
                init_repo(root / "repo"), state_root=root / "state", port=0
            )
            with RunningCoordinator.start(config) as running:
                for ordinal, session_id in enumerate(("session-a", "session-b"), 1):
                    request_id = str(ordinal) * 32
                    registered = self._request(
                        running.base_url,
                        running.token,
                        "POST",
                        "/command",
                        self._payload(
                            request_id,
                            session_id,
                            display_name=f"Session {ordinal}",
                            write_scope=["README.md"],
                            session_role="primary",
                        ),
                    )
                    terminal = self._request(
                        running.base_url,
                        running.token,
                        "GET",
                        f"/command/requests/{request_id}",
                    )

                    self.assertEqual(200, registered["_httpStatus"])
                    self.assertEqual("completed", terminal["request"]["status"])
                    self.assertEqual(session_id, terminal["result"]["session"]["session_id"])
                    self.assertEqual(
                        ["README.md"], terminal["result"]["session"]["write_scope"]
                    )

                lease = running.httpd.application.command(
                    "lease.claim",
                    {"session_id": "session-a", "paths": ["README.md"]},
                )

            self.assertTrue(lease["lease"]["acquired"])

    def test_plan_registration_rolls_back_graph_then_prioritizes_open_handoff(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            fixture = FailureGraphFixture(repo)
            origin = fixture.add_plan("docs/plans/editor/01-editor.md")
            fixing = fixture.add_plan("docs/plans/runtime/02-runtime.md")
            fixture.add_handoff(origin, fixing, "durable-register")
            config = CoordinatorConfig.for_repo(
                repo, state_root=root / "state", port=0
            )
            request_id = "f" * 32
            with RunningCoordinator.start(config) as running:
                application = running.httpd.application
                with mock.patch.object(
                    application.command_requests,
                    "_bounded_response_json",
                    side_effect=CoordinatorError(
                        "response_serialization_failed", "injected completion failure"
                    ),
                ), self.assertRaises(CoordinatorError) as rejected:
                    application.execute_command_request(
                        "session.register",
                        {
                            "session_id": "session-a",
                            "plan_path": fixing.path.relative_to(repo).as_posix(),
                        },
                        request_id=request_id,
                    )
                persisted = application.command_requests.get(request_id)
                with application.database.connect() as connection:
                    session_count = connection.execute(
                        "SELECT COUNT(*) FROM sessions WHERE session_id=?", ("session-a",)
                    ).fetchone()[0]
                    event_count = connection.execute(
                        "SELECT COUNT(*) FROM events WHERE session_id=?", ("session-a",)
                    ).fetchone()[0]
                    workflow_run_count = connection.execute(
                        "SELECT COUNT(*) FROM workflow_runs WHERE session_id=?", ("session-a",)
                    ).fetchone()[0]
                    workflow_node_count = connection.execute(
                        """
                        SELECT COUNT(*) FROM workflow_nodes AS node
                        JOIN workflow_runs AS run USING(run_id)
                        WHERE run.session_id=?
                        """,
                        ("session-a",),
                    ).fetchone()[0]
                    failure_node_count = connection.execute(
                        "SELECT COUNT(*) FROM failure_nodes"
                    ).fetchone()[0]

                completed = application.execute_command_request(
                    "session.register",
                    {
                        "session_id": "session-a",
                        "plan_path": fixing.path.relative_to(repo).as_posix(),
                    },
                    request_id="4" * 32,
                )
                with application.database.connect() as connection:
                    imported_failure = connection.execute(
                        "SELECT summary_slug FROM failure_nodes"
                    ).fetchone()[0]
                    workflow = connection.execute(
                        """
                        SELECT run.state, node.state
                        FROM workflow_runs AS run
                        JOIN workflow_nodes AS node USING(run_id)
                        WHERE run.session_id=? AND node.node_key='goal'
                        """,
                        ("session-a",),
                    ).fetchone()

        self.assertEqual("response_serialization_failed", rejected.exception.code)
        self.assertEqual("failed", persisted["request"]["status"])
        self.assertEqual(0, session_count)
        self.assertEqual(0, event_count)
        self.assertEqual(0, workflow_run_count)
        self.assertEqual(0, workflow_node_count)
        self.assertEqual(0, failure_node_count)
        self.assertEqual("resolving_failure", completed["session"]["status"])
        self.assertEqual(
            ["durable-register"],
            [item["summary_slug"] for item in completed["open_failures"]],
        )
        self.assertEqual("durable-register", imported_failure)
        self.assertEqual(("resolving_failure", "waiting_external"), tuple(workflow))

    def test_second_phase_admission_error_terminalizes_accepted_request(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = CoordinatorConfig.for_repo(
                init_repo(root / "repo"), state_root=root / "state", port=0
            )
            with RunningCoordinator.start(config) as running:
                application = running.httpd.application
                original_transaction = application.database.transaction
                transaction_attempts = 0

                @contextmanager
                def fail_only_the_second_phase(*args, **kwargs):
                    nonlocal transaction_attempts
                    if threading.current_thread() is threading.main_thread():
                        transaction_attempts += 1
                        if transaction_attempts == 2:
                            raise sqlite3.OperationalError("injected phase-two admission error")
                    with original_transaction(*args, **kwargs) as connection:
                        yield connection

                with mock.patch.object(
                    application.database,
                    "transaction",
                    new=fail_only_the_second_phase,
                ), self.assertRaises(sqlite3.OperationalError):
                    application.execute_command_request(
                        "session.register",
                        {"session_id": "session-a"},
                        request_id="9" * 32,
                    )
                persisted = application.command_requests.get("9" * 32)
                with application.database.connect() as connection:
                    session_count = connection.execute(
                        "SELECT COUNT(*) FROM sessions WHERE session_id='session-a'"
                    ).fetchone()[0]
                    event_count = connection.execute(
                        "SELECT COUNT(*) FROM events WHERE session_id='session-a'"
                    ).fetchone()[0]

        self.assertEqual(3, transaction_attempts)
        self.assertEqual("failed", persisted["request"]["status"])
        self.assertEqual("internal_error", persisted["error"]["code"])
        self.assertEqual(0, session_count)
        self.assertEqual(0, event_count)

    def test_deferred_second_phase_failure_terminalizes_after_database_recovers(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            journal = CommandRequestJournal(database)
            original_transaction = database.transaction
            transaction_attempts = 0

            @contextmanager
            def fail_the_admission_and_first_recovery(*args, **kwargs):
                nonlocal transaction_attempts
                transaction_attempts += 1
                if transaction_attempts in {2, 3}:
                    raise sqlite3.OperationalError("injected sustained phase-two lock")
                with original_transaction(*args, **kwargs) as connection:
                    yield connection

            with mock.patch.object(
                database,
                "transaction",
                new=fail_the_admission_and_first_recovery,
            ):
                with self.assertRaises(sqlite3.OperationalError):
                    journal.execute_accepted_transactionally(
                        "8" * 32,
                        "session.register",
                        {"session_id": "session-a"},
                        lambda _connection: ({"session": {"session_id": "session-a"}}, None),
                    )
                accepted = journal.get("8" * 32)
                recovered = journal.retry_deferred_failures()
                terminal = journal.get("8" * 32)

        self.assertEqual("accepted", accepted["request"]["status"])
        self.assertEqual(("8" * 32,), recovered)
        self.assertEqual("failed", terminal["request"]["status"])
        self.assertEqual("internal_error", terminal["error"]["code"])
        self.assertEqual(4, transaction_attempts)

    def test_accepted_request_replay_skips_pre_admission_preparation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            journal = CommandRequestJournal(database)
            preparations: list[str] = []

            first = journal.execute_accepted_transactionally(
                "6" * 32,
                "session.register",
                {"session_id": "session-a"},
                lambda _connection: ({"session": {"session_id": "session-a"}}, None),
                before_admission=lambda: preparations.append("prepared"),
            )
            replay = journal.execute_accepted_transactionally(
                "6" * 32,
                "session.register",
                {"session_id": "session-a"},
                lambda _connection: self.fail("replay invoked admission callback"),
                before_admission=lambda: self.fail("replay invoked preparation callback"),
            )

        self.assertEqual(["prepared"], preparations)
        self.assertEqual(first, replay)

    def test_maintenance_tick_retries_deferred_terminalization(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = CoordinatorConfig.for_repo(
                init_repo(root / "repo"), state_root=root / "state", port=0
            )
            with RunningCoordinator.start(config) as running:
                application = running.httpd.application
                request_id = "7" * 32
                with application.database.transaction() as connection:
                    connection.execute(
                        """INSERT INTO command_requests(
                               request_id, command, arguments_hash, status,
                               received_at, accepted_at
                           ) VALUES (?, 'session.register', ?, 'accepted', 'now', 'now')""",
                        (request_id, "0" * 64),
                    )
                deferred_error = CoordinatorError(
                    "internal_error", "injected deferred terminalization"
                )
                with application.command_requests._deferred_failure_lock:
                    application.command_requests._deferred_failures[request_id] = deferred_error

                application._maintenance_tick(
                    {
                        "apply_cleanup": False,
                        "apply_retention": False,
                        "apply_legacy_archive": False,
                        "apply_lifecycle": False,
                    }
                )
                terminal = application.command_requests.get(request_id)

        self.assertEqual("failed", terminal["request"]["status"])
        self.assertEqual("internal_error", terminal["error"]["code"])

    def test_read_only_scheduled_maintenance_retries_deferred_terminalization(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = CoordinatorConfig.for_repo(
                init_repo(root / "repo"), state_root=root / "state", port=0
            )
            with RunningCoordinator.start(config) as running:
                application = running.httpd.application
                request_id = "9" * 32
                with application.database.transaction() as connection:
                    connection.execute(
                        """INSERT INTO command_requests(
                               request_id, command, arguments_hash, status,
                               received_at, accepted_at
                           ) VALUES (?, 'session.register', ?, 'accepted', 'now', 'now')""",
                        (request_id, "0" * 64),
                    )
                with application.command_requests._deferred_failure_lock:
                    application.command_requests._deferred_failures[request_id] = CoordinatorError(
                        "internal_error", "read-only deferred terminalization"
                    )

                with mock.patch.object(
                    type(application),
                    "read_only",
                    new_callable=mock.PropertyMock,
                    return_value=True,
                ), mock.patch.object(application, "_maintenance_tick") as ordinary_maintenance:
                    RunningCoordinator._run_scheduled_maintenance(application)
                terminal = application.command_requests.get(request_id)

        self.assertEqual("failed", terminal["request"]["status"])
        self.assertEqual("internal_error", terminal["error"]["code"])
        ordinary_maintenance.assert_not_called()

    def test_maintenance_retry_leaves_unrelated_accepted_request_live(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = CoordinatorConfig.for_repo(
                init_repo(root / "repo"), state_root=root / "state", port=0
            )
            with RunningCoordinator.start(config) as running:
                application = running.httpd.application
                live_request_id = "a" * 32
                deferred_request_id = "b" * 32
                with application.database.transaction() as connection:
                    for request_id in (live_request_id, deferred_request_id):
                        connection.execute(
                            """INSERT INTO command_requests(
                                   request_id, command, arguments_hash, status,
                                   received_at, accepted_at
                               ) VALUES (?, 'session.register', ?, 'accepted', 'now', 'now')""",
                            (request_id, "0" * 64),
                        )
                with application.command_requests._deferred_failure_lock:
                    application.command_requests._deferred_failures[deferred_request_id] = CoordinatorError(
                        "internal_error", "only this request may be terminalized"
                    )

                application._maintenance_tick(
                    {
                        "apply_cleanup": False,
                        "apply_retention": False,
                        "apply_legacy_archive": False,
                        "apply_lifecycle": False,
                    }
                )
                live = application.command_requests.get(live_request_id)
                deferred = application.command_requests.get(deferred_request_id)

        self.assertEqual("accepted", live["request"]["status"])
        self.assertEqual("failed", deferred["request"]["status"])

    def test_duplicate_registration_request_executes_mutation_once(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = CoordinatorConfig.for_repo(
                init_repo(root / "repo"), state_root=root / "state", port=0
            )
            request_id = "e" * 32
            payload = self._payload(request_id, "session-a")
            with RunningCoordinator.start(config) as running:
                application = running.httpd.application
                original = application.sessions.register
                executions = 0

                def counted(*args, **kwargs):
                    nonlocal executions
                    executions += 1
                    return original(*args, **kwargs)

                with mock.patch.object(
                    application.sessions, "register", side_effect=counted
                ):
                    first = self._request(
                        running.base_url, running.token, "POST", "/command", payload
                    )
                    second = self._request(
                        running.base_url, running.token, "POST", "/command", payload
                    )

        first.pop("_httpStatus")
        second.pop("_httpStatus")
        self.assertEqual(first, second)
        self.assertEqual(1, executions)

    def test_plan_backed_duplicate_does_not_reprepare_or_reimport_failure_graph(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = CoordinatorConfig.for_repo(
                init_repo(root / "repo"), state_root=root / "state", port=0
            )
            request_id = "1" * 32
            payload = self._payload(
                request_id,
                "session-a",
                plan_path="docs/plans/runtime/01-runtime.md",
            )
            with RunningCoordinator.start(config) as running:
                application = running.httpd.application
                with (
                    mock.patch.object(
                        application.failures,
                        "prepare_import_snapshot",
                        wraps=application.failures.prepare_import_snapshot,
                    ) as prepared,
                    mock.patch.object(
                        application.failures,
                        "import_prepared_snapshot",
                        wraps=application.failures.import_prepared_snapshot,
                    ) as imported,
                ):
                    first = self._request(
                        running.base_url, running.token, "POST", "/command", payload
                    )
                    second = self._request(
                        running.base_url, running.token, "POST", "/command", payload
                    )

        first.pop("_httpStatus")
        second.pop("_httpStatus")
        self.assertEqual(first, second)
        self.assertEqual(1, prepared.call_count)
        self.assertEqual(1, imported.call_count)

    def test_rejected_plan_backed_registration_does_not_import_failure_graph(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = CoordinatorConfig.for_repo(
                init_repo(root / "repo"), state_root=root / "state", port=0
            )
            request_id = "2" * 32
            with RunningCoordinator.start(config) as running:
                application = running.httpd.application
                with (
                    mock.patch.object(
                        type(application),
                        "read_only",
                        new_callable=mock.PropertyMock,
                        return_value=True,
                    ),
                    mock.patch.object(
                        application.failures, "import_repository"
                    ) as imported,
                    self.assertRaises(CoordinatorError) as rejected,
                ):
                    application.execute_command_request(
                        "session.register",
                        {
                            "session_id": "session-a",
                            "plan_path": "docs/plans/runtime/01-runtime.md",
                        },
                        request_id=request_id,
                    )
                persisted = application.command_requests.get(request_id)

        self.assertEqual("not_on_main", rejected.exception.code)
        self.assertEqual("failed", persisted["request"]["status"])
        imported.assert_not_called()

    def test_supervision_rejection_does_not_import_failure_graph(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = CoordinatorConfig.for_repo(
                init_repo(root / "repo"), state_root=root / "state", port=0
            )
            request_id = "3" * 32
            with RunningCoordinator.start(config) as running:
                application = running.httpd.application
                with (
                    mock.patch.object(
                        application.supervision,
                        "require_mutation_allowed_in_connection",
                        side_effect=CoordinatorError(
                            "maintenance_hold_active", "injected supervision rejection"
                        ),
                    ),
                    mock.patch.object(
                        application.failures, "import_repository"
                    ) as imported,
                    self.assertRaises(CoordinatorError) as rejected,
                ):
                    application.execute_command_request(
                        "session.register",
                        {
                            "session_id": "session-a",
                            "plan_path": "docs/plans/runtime/01-runtime.md",
                        },
                        request_id=request_id,
                    )
                persisted = application.command_requests.get(request_id)

        self.assertEqual("maintenance_hold_active", rejected.exception.code)
        self.assertEqual("failed", persisted["request"]["status"])
        imported.assert_not_called()

    def test_registration_request_is_queryable_while_execution_is_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = CoordinatorConfig.for_repo(
                init_repo(root / "repo"), state_root=root / "state", port=0
            )
            request_id = "a" * 32
            entered = threading.Event()
            release = threading.Event()
            response: dict[str, object] = {}
            with RunningCoordinator.start(config) as running:
                application = running.httpd.application
                original = application.sessions.register

                def gated_register(*args, **kwargs):
                    entered.set()
                    release.wait(timeout=3)
                    return original(*args, **kwargs)

                def register() -> None:
                    response.update(
                        self._request(
                            running.base_url,
                            running.token,
                            "POST",
                            "/command",
                            self._payload(request_id, "session-a"),
                        )
                    )

                with mock.patch.object(
                    application.sessions, "register", side_effect=gated_register
                ):
                    worker = threading.Thread(target=register, daemon=True)
                    worker.start()
                    try:
                        self.assertTrue(entered.wait(timeout=2))
                        accepted = self._request(
                            running.base_url,
                            running.token,
                            "GET",
                            f"/command/requests/{request_id}",
                        )
                    finally:
                        release.set()
                        worker.join(timeout=5)
                terminal = self._request(
                    running.base_url,
                    running.token,
                    "GET",
                    f"/command/requests/{request_id}",
                )

            self.assertFalse(worker.is_alive())
            self.assertEqual("accepted", accepted["request"]["status"])
            self.assertEqual(200, response["_httpStatus"])
            self.assertEqual("completed", terminal["request"]["status"])

    def test_restart_terminalizes_unexecuted_registration_without_partial_session(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            database = Database(config.database_path)
            migrate(database)
            request_id = "b" * 32
            arguments = {"session_id": "session-a"}
            fingerprint = hashlib.sha256(
                json.dumps(
                    {"command": "session.register", "arguments": arguments},
                    sort_keys=True,
                    separators=(",", ":"),
                ).encode("utf-8")
            ).hexdigest()
            with database.transaction() as connection:
                connection.execute(
                    """INSERT INTO command_requests(
                           request_id, command, arguments_hash, status,
                           received_at, accepted_at
                       ) VALUES (?, 'session.register', ?, 'accepted', 'now', 'now')""",
                    (request_id, fingerprint),
                )

            with RunningCoordinator.start(config) as running:
                terminal = self._request(
                    running.base_url,
                    running.token,
                    "GET",
                    f"/command/requests/{request_id}",
                )
                with running.httpd.application.database.connect() as connection:
                    session = connection.execute(
                        "SELECT * FROM sessions WHERE session_id = 'session-a'"
                    ).fetchone()

            self.assertEqual("failed", terminal["request"]["status"])
            self.assertEqual("command_execution_interrupted", terminal["error"]["code"])
            self.assertIsNone(session)

    def test_failed_reregistration_preserves_session_and_terminalizes_request(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = CoordinatorConfig.for_repo(
                init_repo(root / "repo"), state_root=root / "state", port=0
            )
            with RunningCoordinator.start(config) as running:
                first_id = "c" * 32
                failed_id = "d" * 32
                self._request(
                    running.base_url,
                    running.token,
                    "POST",
                    "/command",
                    self._payload(
                        first_id,
                        "session-a",
                        display_name="Original",
                        write_scope=["README.md"],
                    ),
                )
                failed = self._request(
                    running.base_url,
                    running.token,
                    "POST",
                    "/command",
                    self._payload(
                        failed_id,
                        "session-a",
                        display_name="Overwritten",
                        write_scope=["foreign.txt"],
                    ),
                )
                terminal = self._request(
                    running.base_url,
                    running.token,
                    "GET",
                    f"/command/requests/{failed_id}",
                )
                session = running.httpd.application.sessions.get("session-a")
                with running.httpd.application.database.connect() as connection:
                    registered_events = connection.execute(
                        """SELECT COUNT(*) FROM events
                           WHERE session_id = 'session-a'
                             AND event_type = 'session.registered'"""
                    ).fetchone()[0]

            self.assertGreaterEqual(failed["_httpStatus"], 400)
            self.assertEqual("failed", terminal["request"]["status"])
            self.assertEqual("session_write_scope_immutable", terminal["error"]["code"])
            self.assertEqual("Original", session.display_name)
            self.assertEqual(("README.md",), session.write_scope)
            self.assertEqual(1, registered_events)


if __name__ == "__main__":
    unittest.main()
