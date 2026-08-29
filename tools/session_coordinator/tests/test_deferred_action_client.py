from __future__ import annotations

import json
import os
import unittest
import tempfile
from contextlib import redirect_stdout
from io import StringIO
from pathlib import Path
from unittest import mock

from tools.session_coordinator.client import (
    CoordinatorClient,
    CoordinatorClientError,
)
from tools.session_coordinator import cli
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.offline_queue import OfflineCommandSpool
from tools.session_coordinator.processes import process_creation_time


class DeferredActionClientTests(unittest.TestCase):
    def _client(self, *, timeout: float = 1.0) -> CoordinatorClient:
        return CoordinatorClient(
            "http://127.0.0.1:6518",
            "",
            command_timeout_seconds=timeout,
        )

    @staticmethod
    def _preview() -> dict[str, object]:
        return {
            "action": {
                "actionId": "action-a",
                "confirmationPhrase": "CONFIRM",
                "status": "previewed",
            }
        }

    def test_terminal_failure_states_stop_polling_and_return_action(self) -> None:
        for status in ("failed", "cancelled", "expired", "state_changed", "denied"):
            with self.subTest(status=status):
                terminal = {
                    "actionId": "action-a",
                    "status": status,
                    "errorCode": f"action_{status}",
                    "result": None,
                }
                with mock.patch.object(
                    CoordinatorClient,
                    "control_request",
                    autospec=True,
                    side_effect=(self._preview(), {"action": terminal}),
                ) as request:
                    result = self._client().execute_control_action(
                        "validation.start",
                        {"sessionId": "session-a"},
                        reason="exercise terminal status",
                    )

                self.assertEqual(terminal, result)
                self.assertEqual(2, request.call_count)

    def test_poll_timeout_is_typed_and_does_not_repeat_confirmation(self) -> None:
        executing = {
            "actionId": "action-a",
            "status": "executing",
            "result": None,
        }
        with mock.patch.object(
            CoordinatorClient,
            "control_request",
            autospec=True,
            side_effect=(self._preview(), {"action": executing}),
        ) as request:
            with self.assertRaises(CoordinatorClientError) as rejected:
                self._client(timeout=0.0).execute_control_action(
                    "validation.start",
                    {"sessionId": "session-a"},
                    reason="exercise timeout",
                )

        self.assertEqual("command_timeout", rejected.exception.code)
        self.assertEqual(
            {"actionId": "action-a", "kind": "validation.start"},
            rejected.exception.details,
        )
        self.assertEqual(2, request.call_count)

    def test_rollover_status_poll_recovers_after_listener_transition(self) -> None:
        executing = {
            "actionId": "action-a",
            "status": "executing",
            "result": None,
        }
        succeeded = {
            "actionId": "action-a",
            "status": "succeeded",
            "result": {"successorInstanceId": "daemon-b"},
        }
        with (
            mock.patch.object(
                CoordinatorClient,
                "control_request",
                autospec=True,
                side_effect=(
                    self._preview(),
                    {"action": executing},
                    CoordinatorClientError("offline", "listener is restarting"),
                    {"action": succeeded},
                ),
            ) as request,
            mock.patch("tools.session_coordinator.client.time.sleep", return_value=None),
        ):
            result = self._client().execute_control_action(
                "service.rollover",
                {"timeoutSeconds": 30},
                reason="recover the successor action record after listener handoff",
            )

        self.assertEqual(succeeded, result)
        self.assertEqual(4, request.call_count)
        self.assertTrue(str(request.call_args_list[1].args[2]).endswith("/confirm"))

    def test_rollover_status_poll_recovers_while_successor_reconciles_action_identity(self) -> None:
        executing = {
            "actionId": "action-a",
            "status": "executing",
            "result": None,
        }
        succeeded = {
            "actionId": "action-a",
            "status": "succeeded",
            "result": {"successorInstanceId": "daemon-b"},
        }
        with (
            mock.patch.object(
                CoordinatorClient,
                "control_request",
                autospec=True,
                side_effect=(
                    self._preview(),
                    {"action": executing},
                    CoordinatorClientError(
                        "action_instance_mismatch",
                        "action belongs to the predecessor instance",
                    ),
                    {"action": succeeded},
                ),
            ) as request,
            mock.patch("tools.session_coordinator.client.time.sleep", return_value=None),
        ):
            result = self._client().execute_control_action(
                "service.rollover",
                {"timeoutSeconds": 30},
                reason="wait for successor action reconciliation",
            )

        self.assertEqual(succeeded, result)
        self.assertEqual(4, request.call_count)

    def test_rollover_status_poll_refreshes_successor_runtime_token(self) -> None:
        executing = {
            "actionId": "action-a",
            "status": "executing",
            "result": None,
        }
        succeeded = {
            "actionId": "action-a",
            "status": "succeeded",
            "result": {"successorInstanceId": "daemon-b"},
        }
        calls: list[tuple[str, str, str]] = []
        with tempfile.TemporaryDirectory() as directory:
            runtime_path = Path(directory) / "runtime.json"
            runtime_path.write_text(
                json.dumps(
                    {
                        "host": "127.0.0.1",
                        "port": 6518,
                        "token": "successor-secret",
                        "repository_key": "repository-a",
                    }
                ),
                encoding="utf-8",
            )
            client = CoordinatorClient(
                "http://127.0.0.1:6518",
                "predecessor-secret",
                expected_repository_key="repository-a",
                runtime_path=runtime_path,
                command_timeout_seconds=1.0,
            )

            def request(current, method, path, payload=None):
                calls.append((current.token, method, path))
                if path == "/control/v1/actions/preview":
                    return self._preview()
                if path.endswith("/confirm"):
                    return {"action": executing}
                if current.token == "predecessor-secret":
                    raise CoordinatorClientError(
                        "unauthorized", "successor rejected predecessor token"
                    )
                return {"action": succeeded}

            with (
                mock.patch.object(CoordinatorClient, "control_request", new=request),
                mock.patch("tools.session_coordinator.client.time.sleep", return_value=None),
            ):
                result = client.execute_control_action(
                    "service.rollover",
                    {"timeoutSeconds": 30},
                    reason="follow the durable action across credential rotation",
                )

        self.assertEqual(succeeded, result)
        self.assertEqual(1, sum(path.endswith("/confirm") for _, _, path in calls))
        self.assertEqual("successor-secret", calls[-1][0])
        self.assertEqual("/control/v1/actions/action-a", calls[-1][2])

    def test_rollover_status_poll_survives_successor_descriptor_gap(self) -> None:
        executing = {
            "actionId": "action-a",
            "status": "executing",
            "result": None,
        }
        succeeded = {
            "actionId": "action-a",
            "status": "succeeded",
            "result": {"successorInstanceId": "daemon-b"},
        }
        predecessor = CoordinatorClient(
            "http://127.0.0.1:6518",
            "predecessor-secret",
            runtime_path=Path("runtime.json"),
            command_timeout_seconds=1.0,
        )
        successor = CoordinatorClient(
            "http://127.0.0.1:6518",
            "successor-secret",
            runtime_path=Path("runtime.json"),
            command_timeout_seconds=1.0,
        )
        calls: list[tuple[str, str]] = []

        def request(current, method, path, payload=None):
            calls.append((current.token, path))
            if path == "/control/v1/actions/preview":
                return self._preview()
            if path.endswith("/confirm"):
                return {"action": executing}
            if current.token == "predecessor-secret":
                raise CoordinatorClientError(
                    "unauthorized", "successor rejected predecessor token"
                )
            return {"action": succeeded}

        descriptor_absent = CoordinatorClientError(
            "offline",
            "Coordinator successor descriptor is unavailable",
            details={"transport": "descriptor_absent"},
        )
        with (
            mock.patch.object(CoordinatorClient, "control_request", new=request),
            mock.patch.object(
                CoordinatorClient,
                "_refresh_runtime_client",
                autospec=True,
                side_effect=(descriptor_absent, successor),
            ) as refresh,
            mock.patch("tools.session_coordinator.client.time.sleep", return_value=None),
        ):
            result = predecessor.execute_control_action(
                "service.rollover",
                {"timeoutSeconds": 30},
                reason="keep polling through the successor descriptor gap",
            )

        self.assertEqual(succeeded, result)
        self.assertEqual(2, refresh.call_count)
        self.assertEqual(1, sum(path.endswith("/confirm") for _, path in calls))
        self.assertTrue(
            all(
                path in {
                    "/control/v1/actions/preview",
                    "/control/v1/actions/action-a/confirm",
                    "/control/v1/actions/action-a",
                }
                for _, path in calls
            )
        )
        self.assertEqual(("successor-secret", "/control/v1/actions/action-a"), calls[-1])

    def test_non_rollover_action_identity_mismatch_is_not_retried(self) -> None:
        executing = {
            "actionId": "action-a",
            "status": "executing",
            "result": None,
        }
        with (
            mock.patch.object(
                CoordinatorClient,
                "control_request",
                autospec=True,
                side_effect=(
                    self._preview(),
                    {"action": executing},
                    CoordinatorClientError(
                        "action_instance_mismatch",
                        "action belongs to another instance",
                    ),
                ),
            ) as request,
            mock.patch("tools.session_coordinator.client.time.sleep", return_value=None),
        ):
            with self.assertRaises(CoordinatorClientError) as rejected:
                self._client().execute_control_action(
                    "validation.start",
                    {"sessionId": "session-a"},
                    reason="preserve action identity enforcement",
                )

        self.assertEqual("action_instance_mismatch", rejected.exception.code)
        self.assertEqual(3, request.call_count)

    def test_malformed_polled_detail_is_typed_invalid_response(self) -> None:
        executing = {
            "actionId": "action-a",
            "status": "executing",
            "result": None,
        }
        with mock.patch.object(
            CoordinatorClient,
            "control_request",
            autospec=True,
            side_effect=(
                self._preview(),
                {"action": executing},
                {"action": "malformed"},
            ),
        ), mock.patch(
            "tools.session_coordinator.client.time.sleep",
            return_value=None,
        ):
            with self.assertRaises(CoordinatorClientError) as rejected:
                self._client().execute_control_action(
                    "validation.start",
                    {"sessionId": "session-a"},
                    reason="exercise malformed detail",
                )

        self.assertEqual("invalid_response", rejected.exception.code)
        self.assertIn("detail omitted", rejected.exception.message)

    def test_descriptor_absence_queues_an_allowed_session_command_for_later_replay(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = StringIO()
            with redirect_stdout(output):
                exit_code = cli.main(
                    [
                        "--repo-root",
                        ".",
                        "--state-root",
                        temporary,
                        "--json",
                        "session",
                        "heartbeat",
                        "--session-id",
                        "session-a",
                    ]
                )

            self.assertEqual(0, exit_code)
            result = __import__("json").loads(output.getvalue())
            self.assertEqual("queued", result["status"])
            config = CoordinatorConfig.for_repo(".", state_root=temporary)
            spool = OfflineCommandSpool(
                config.offline_command_queue_root,
                repository_key=config.repository_key,
            )
            queued = spool.validated_pending()
            self.assertEqual(1, len(queued))
            self.assertEqual("session.heartbeat", queued[0].command)
            self.assertEqual({"session_id": "session-a"}, queued[0].arguments)

    @unittest.skipUnless(os.name == "nt", "Windows ctypes process identity contract")
    def test_offline_spool_reuses_shared_windows_process_identity_contract(self) -> None:
        process_creation_time(os.getpid())
        with tempfile.TemporaryDirectory() as temporary:
            spool = OfflineCommandSpool(
                Path(temporary),
                repository_key="a" * 64,
            )

            queued = spool.enqueue("session.heartbeat", {"session_id": "session-a"})

            self.assertEqual("session.heartbeat", queued.command)
            self.assertEqual(1, spool.snapshot().pending)

    def test_offline_implicit_session_registration_is_not_queued_with_a_new_random_identity(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = StringIO()
            with redirect_stdout(output), mock.patch.dict("os.environ", {}, clear=True):
                exit_code = cli.main(
                    [
                        "--repo-root",
                        ".",
                        "--state-root",
                        temporary,
                        "--json",
                        "session",
                        "register",
                    ]
                )

            self.assertEqual(3, exit_code)
            config = CoordinatorConfig.for_repo(".", state_root=temporary)
            spool = OfflineCommandSpool(
                config.offline_command_queue_root,
                repository_key=config.repository_key,
            )
            self.assertEqual(0, spool.snapshot().pending)

    def test_offline_cargo_command_is_not_queued(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = StringIO()
            with redirect_stdout(output):
                exit_code = cli.main(
                    [
                        "--repo-root",
                        ".",
                        "--state-root",
                        temporary,
                        "--json",
                        "cargo",
                        "acquire",
                        "test",
                        "--session-id",
                        "session-a",
                    ]
                )

            self.assertEqual(3, exit_code)
            config = CoordinatorConfig.for_repo(".", state_root=temporary)
            spool = OfflineCommandSpool(
                config.offline_command_queue_root,
                repository_key=config.repository_key,
            )
            self.assertEqual(0, spool.snapshot().pending)

    def test_offline_session_status_change_is_not_queued(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = StringIO()
            with redirect_stdout(output):
                exit_code = cli.main(
                    [
                        "--repo-root",
                        ".",
                        "--state-root",
                        temporary,
                        "--json",
                        "session",
                        "set-status",
                        "active",
                        "--session-id",
                        "session-a",
                    ]
                )

            self.assertEqual(3, exit_code)
            config = CoordinatorConfig.for_repo(".", state_root=temporary)
            spool = OfflineCommandSpool(
                config.offline_command_queue_root,
                repository_key=config.repository_key,
            )
            self.assertEqual(0, spool.snapshot().pending)

    def test_ambiguous_connection_loss_is_not_queued_after_client_dispatch_begins(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            client = mock.Mock()
            client.command.side_effect = CoordinatorClientError(
                "offline",
                "connection lost",
                details={"transport": "connection_uncertain"},
            )
            output = StringIO()
            with (
                mock.patch.object(CoordinatorClient, "from_runtime", return_value=client),
                redirect_stdout(output),
            ):
                exit_code = cli.main(
                    [
                        "--repo-root",
                        ".",
                        "--state-root",
                        temporary,
                        "--json",
                        "session",
                        "heartbeat",
                        "--session-id",
                        "session-a",
                    ]
                )

            self.assertEqual(3, exit_code)
            config = CoordinatorConfig.for_repo(".", state_root=temporary)
            spool = OfflineCommandSpool(
                config.offline_command_queue_root,
                repository_key=config.repository_key,
            )
            self.assertEqual(0, spool.snapshot().pending)

    def test_healthy_status_replays_pending_work_before_returning(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            config = CoordinatorConfig.for_repo(".", state_root=temporary)
            spool = OfflineCommandSpool(
                config.offline_command_queue_root,
                repository_key=config.repository_key,
            )
            spool.enqueue("session.heartbeat", {"session_id": "session-a"})
            client = mock.Mock()
            client.health.return_value = {"status": "ok"}
            arguments = cli._parser().parse_args(
                ["--repo-root", ".", "--state-root", temporary, "status"]
            )

            with mock.patch.object(CoordinatorClient, "from_runtime", return_value=client):
                result = cli._run(arguments)

            self.assertEqual("ok", result["status"])
            self.assertEqual({"acknowledged": 1, "failed": 0, "quarantined": 0, "retained": 0}, result["offlineReplay"])
            client.command.assert_called_once_with(
                "session.heartbeat", {"session_id": "session-a"}
            )


if __name__ == "__main__":
    unittest.main()
