from __future__ import annotations

import json
import sqlite3
import subprocess
import unittest
from unittest import mock

from tools.session_coordinator import cli
from tools.session_coordinator.client import CoordinatorClient
from tools.session_coordinator.config import CoordinatorConfig, DEFAULT_COORDINATOR_PORT
from tools.session_coordinator.models import CoordinatorError


class MilestoneControlClientTests(unittest.TestCase):
    def test_cli_port_override_supports_an_isolated_listener(self) -> None:
        arguments = cli._parser().parse_args(
            ["--repo-root", ".", "--port", "0", "status"]
        )

        config = cli._config(arguments)

        self.assertEqual(0, config.port)

    def test_control_action_confirms_its_preview_with_the_returned_phrase(self) -> None:
        client = CoordinatorClient(f"http://127.0.0.1:{DEFAULT_COORDINATOR_PORT}", "")
        requests: list[tuple[str, str, dict[str, object]]] = []

        def request(
            _client: CoordinatorClient,
            method: str,
            path: str,
            payload: dict[str, object] | None = None,
        ):
            requests.append((method, path, payload or {}))
            if path.endswith("/preview"):
                return {
                    "action": {
                        "actionId": "action-1",
                        "confirmationPhrase": "CONFIRM TOPOLOGY.REFRESH",
                    }
                }
            return {"action": {"status": "succeeded", "result": {"runId": "run-1"}}}

        with mock.patch.object(
            CoordinatorClient, "control_request", autospec=True, side_effect=request
        ):
            action = client.execute_control_action(
                "topology.refresh",
                {"sessionId": "session-a"},
                reason="prepare milestone M1",
            )

        self.assertEqual("succeeded", action["status"])
        self.assertEqual(
            [
                (
                    "POST",
                    "/control/v1/actions/preview",
                    {"kind": "topology.refresh", "parameters": {"sessionId": "session-a"}},
                ),
                (
                    "POST",
                    "/control/v1/actions/action-1/confirm",
                    {
                        "phrase": "CONFIRM TOPOLOGY.REFRESH",
                        "reason": "prepare milestone M1",
                    },
                ),
            ],
            requests,
        )

    def test_control_action_polls_executing_confirmation_until_terminal(self) -> None:
        client = CoordinatorClient(f"http://127.0.0.1:{DEFAULT_COORDINATOR_PORT}", "")
        requests: list[tuple[str, str, dict[str, object]]] = []
        detail_statuses = iter(("executing", "succeeded"))

        def request(
            _client: CoordinatorClient,
            method: str,
            path: str,
            payload: dict[str, object] | None = None,
        ):
            requests.append((method, path, payload or {}))
            if path.endswith("/preview"):
                return {
                    "action": {
                        "actionId": "action-1",
                        "confirmationPhrase": "CONFIRM VALIDATION.START",
                    }
                }
            if path.endswith("/confirm"):
                return {"action": {"actionId": "action-1", "status": "executing"}}
            status = next(detail_statuses)
            result = {"jobId": "job-1"} if status == "succeeded" else None
            return {
                "action": {
                    "actionId": "action-1",
                    "status": status,
                    "result": result,
                }
            }

        with (
            mock.patch.object(
                CoordinatorClient, "control_request", autospec=True, side_effect=request
            ),
            mock.patch("time.sleep"),
        ):
            action = client.execute_control_action(
                "validation.start",
                {
                    "sessionId": "session-a",
                    "runId": "run-1",
                    "milestoneId": "M1.3",
                    "template": "coordinator-actions",
                },
                reason="start validation",
            )

        self.assertEqual("succeeded", action["status"])
        self.assertEqual({"jobId": "job-1"}, action["result"])
        self.assertEqual(
            [
                ("GET", "/control/v1/actions/action-1", {}),
                ("GET", "/control/v1/actions/action-1", {}),
            ],
            requests[2:],
        )


class MilestoneCliTests(unittest.TestCase):
    def test_runtime14_rust_template_is_a_closed_cli_choice(self) -> None:
        arguments = cli._parser().parse_args(
            [
                "--repo-root",
                ".",
                "milestone",
                "validate",
                "--session-id",
                "session-a",
                "--run-id",
                "run-a",
                "--milestone",
                "M4",
                "--template",
                "runtime14-rust-focused",
            ]
        )

        self.assertEqual("runtime14-rust-focused", arguments.template)

    @mock.patch("tools.session_coordinator.cli.CoordinatorClient.from_runtime")
    def test_failure_deferral_cli_uses_one_durable_typed_command(self, from_runtime) -> None:
        client = from_runtime.return_value
        client.command.return_value = {"deferral": {"targetMilestoneId": "M3"}}
        arguments = cli._parser().parse_args(
            [
                "milestone",
                "defer-failure",
                "--session-id",
                "session-a",
                "--source-milestone",
                "m2",
                "--target-milestone",
                "m3",
                "--failure-lifecycle-key",
                "failure-key",
            ]
        )

        result = cli._run(arguments)

        self.assertEqual("M3", result["deferral"]["targetMilestoneId"])
        client.command.assert_called_once_with(
            "milestone.defer_failure",
            {
                "session_id": "session-a",
                "source_milestone_key": "M2",
                "target_milestone_key": "M3",
                "failure_lifecycle_key": "failure-key",
                "actor": "session-a",
            },
        )

    def test_main_serializes_database_failures_without_traceback_or_raw_details(self) -> None:
        with mock.patch.object(
            cli, "_run", side_effect=sqlite3.DatabaseError("raw database detail")
        ), mock.patch("builtins.print") as output:
            exit_code = cli.main(["--repo-root", ".", "--json", "status"])

        self.assertEqual(2, exit_code)
        payload = json.loads(output.call_args.args[0])
        self.assertEqual("error", payload["status"])
        self.assertEqual("coordinator_database_error", payload["error"]["code"])
        self.assertNotIn("raw database detail", output.call_args.args[0])

    @mock.patch("tools.session_coordinator.cli.ctypes.WinDLL")
    def test_windows_kernel32_uses_thread_local_last_error(self, windll) -> None:
        cli._windows_kernel32()

        windll.assert_called_once_with("kernel32", use_last_error=True)

    @mock.patch("tools.session_coordinator.cli.os.kill", side_effect=PermissionError)
    def test_predecessor_handle_does_not_treat_an_identity_error_as_exit(
        self, _kill
    ) -> None:
        handle = cli._PredecessorHandle(
            {"pid": 101, "process_creation_time": "old-created"}
        )

        with self.assertRaises(CoordinatorError) as rejected:
            cli._predecessor_handle_exited(handle)

        self.assertEqual("bootstrap_predecessor_identity_unavailable", rejected.exception.code)

    @mock.patch("tools.session_coordinator.cli._shutdown_predecessor")
    @mock.patch("tools.session_coordinator.cli.prepare_proof_bound_handoff")
    @mock.patch("tools.session_coordinator.cli._close_predecessor_handle")
    @mock.patch("tools.session_coordinator.cli._capture_predecessor_handle")
    @mock.patch("tools.session_coordinator.cli._runtime_descriptor")
    def test_bootstrap_prepare_rejection_closes_handle_and_preserves_error(
        self, runtime_descriptor, capture, close, prepare, shutdown
    ) -> None:
        runtime = {
            "instance_id": "old-instance",
            "pid": 101,
            "process_creation_time": "created",
        }
        handle = cli._PredecessorHandle(runtime)
        runtime_descriptor.return_value = runtime
        capture.return_value = handle
        prepare.side_effect = CoordinatorError("bootstrap_proof_rejected", "rejected")

        with self.assertRaises(CoordinatorError) as rejected:
            cli.bootstrap_proof_bound_handoff(
                CoordinatorConfig.for_repo("."),
                reservation_id="hgi-reservation",
                maintenance_session_ids=("repair-owner", "hgi-owner"),
                actor="bootstrap-owner",
            )

        self.assertEqual("bootstrap_proof_rejected", rejected.exception.code)
        close.assert_called_once_with(handle)
        shutdown.assert_not_called()

    def test_bootstrap_post_prepare_operational_failures_return_structured_blockers(
        self,
    ) -> None:
        runtime = {
            "instance_id": "old-instance",
            "pid": 101,
            "process_creation_time": "created",
            "executable": "python.exe",
            "command_line": ["coordinator.py", "serve"],
        }
        failures = (
            (
                "shutdown",
                subprocess.CalledProcessError(1, ["taskkill.exe"]),
                "bootstrap_process_command_failed",
            ),
            (
                "wait",
                subprocess.TimeoutExpired(["taskkill.exe"], 15),
                "bootstrap_process_timeout",
            ),
            ("start", OSError("cannot start successor"), "bootstrap_process_os_error"),
        )

        for phase, failure, expected_code in failures:
            with self.subTest(phase=phase), mock.patch.object(
                cli, "_runtime_descriptor", return_value=runtime
            ), mock.patch.object(
                cli,
                "_capture_predecessor_handle",
                return_value=mock.sentinel.predecessor_handle,
            ), mock.patch.object(
                cli,
                "prepare_proof_bound_handoff",
                return_value={
                    "actionId": "proof-action",
                    "proofBound": True,
                    "reservationId": "hgi-reservation",
                },
            ), mock.patch.object(cli, "_shutdown_predecessor") as shutdown, mock.patch.object(
                cli, "_wait_for_predecessor_exit"
            ) as wait_predecessor, mock.patch.object(
                cli, "_start_successor"
            ) as start_successor, mock.patch.object(
                cli, "_close_predecessor_handle"
            ) as close:
                {
                    "shutdown": shutdown,
                    "wait": wait_predecessor,
                    "start": start_successor,
                }[phase].side_effect = failure

                result = cli.bootstrap_proof_bound_handoff(
                    CoordinatorConfig.for_repo("."),
                    reservation_id="hgi-reservation",
                    maintenance_session_ids=("repair-owner", "hgi-owner"),
                    actor="bootstrap-owner",
                )

                self.assertFalse(result["ready"])
                self.assertEqual(
                    [{"kind": "predecessor_handoff", "code": expected_code}],
                    result["blockers"],
                )
                self.assertEqual("proof-action", result["actionId"])
                close.assert_called_once_with(mock.sentinel.predecessor_handle)

    def test_bootstrap_validation_database_failure_returns_proof_bound_blocker(self) -> None:
        runtime = {
            "instance_id": "old-instance",
            "pid": 101,
            "process_creation_time": "created",
            "executable": "python.exe",
            "command_line": ["coordinator.py", "serve"],
        }
        prepared = {
            "actionId": "proof-action",
            "proofBound": True,
            "reservationId": "hgi-reservation",
        }
        successor = {
            "runtime": {"instance_id": "successor-instance", "schema_version": 50},
            "health": {"supervision": {"maintenanceHold": True}},
        }
        with mock.patch.object(
            cli, "_runtime_descriptor", return_value=runtime
        ), mock.patch.object(
            cli,
            "_capture_predecessor_handle",
            return_value=mock.sentinel.predecessor_handle,
        ), mock.patch.object(
            cli, "prepare_proof_bound_handoff", return_value=prepared
        ), mock.patch.object(
            cli, "_shutdown_predecessor"
        ), mock.patch.object(
            cli, "_wait_for_predecessor_exit"
        ), mock.patch.object(
            cli, "_start_successor"
        ), mock.patch.object(
            cli, "_wait_for_successor", return_value=successor
        ), mock.patch.object(
            cli,
            "validate_proof_bound_handoff",
            side_effect=sqlite3.DatabaseError("audit unavailable"),
        ), mock.patch.object(cli, "_close_predecessor_handle"):
            result = cli.bootstrap_proof_bound_handoff(
                CoordinatorConfig.for_repo("."),
                reservation_id="hgi-reservation",
                maintenance_session_ids=("repair-owner", "hgi-owner"),
                actor="bootstrap-owner",
            )

        self.assertFalse(result["ready"])
        self.assertEqual("proof-action", result["actionId"])
        self.assertEqual(
            [{"kind": "predecessor_handoff", "code": "bootstrap_database_error"}],
            result["blockers"],
        )

    @mock.patch("tools.session_coordinator.cli.validate_proof_bound_handoff")
    @mock.patch("tools.session_coordinator.cli._wait_for_successor")
    @mock.patch("tools.session_coordinator.cli._start_successor")
    @mock.patch("tools.session_coordinator.cli._wait_for_predecessor_exit")
    @mock.patch("tools.session_coordinator.cli._shutdown_predecessor")
    @mock.patch("tools.session_coordinator.cli.prepare_proof_bound_handoff")
    @mock.patch("tools.session_coordinator.cli._close_predecessor_handle")
    @mock.patch("tools.session_coordinator.cli._capture_predecessor_handle")
    @mock.patch("tools.session_coordinator.cli._runtime_descriptor")
    def test_bootstrap_handoff_stops_predecessor_before_post_exit_not_ready_audit(
        self,
        runtime_descriptor,
        capture_predecessor,
        close_predecessor,
        prepare,
        shutdown,
        wait_predecessor,
        start_successor,
        wait_successor,
        validate,
    ) -> None:
        runtime_descriptor.return_value = {
            "instance_id": "old-instance",
            "pid": 101,
            "process_creation_time": "old-created",
            "executable": "python.exe",
            "command_line": ["coordinator.py", "serve"],
        }
        captured_handle = mock.sentinel.predecessor_handle
        capture_predecessor.return_value = captured_handle
        call_order: list[str] = []
        capture_predecessor.side_effect = lambda _runtime: (
            call_order.append("capture") or captured_handle
        )
        prepare.return_value = {
            "actionId": "proof-action",
            "proofBound": True,
            "reservationId": "hgi-reservation",
        }
        prepare.side_effect = lambda *args, **kwargs: (
            call_order.append("proof") or {
                "actionId": "proof-action",
                "proofBound": True,
                "reservationId": "hgi-reservation",
            }
        )
        shutdown.side_effect = lambda _handle: call_order.append("shutdown")
        wait_predecessor.side_effect = lambda _handle: call_order.append("wait")
        wait_successor.return_value = {
            "runtime": {"instance_id": "successor-instance", "schema_version": 50},
            "health": {"supervision": {"maintenanceHold": True}},
        }
        validate.return_value = {
            "ready": False,
            "blockers": [{"kind": "cargo", "jobId": "legacy-job"}],
        }
        arguments = cli._parser().parse_args(["--repo-root", ".", "status"])
        config = cli._config(arguments)

        result = cli.bootstrap_proof_bound_handoff(
            config,
            reservation_id="hgi-reservation",
            maintenance_session_ids=("repair-owner", "hgi-owner"),
            actor="bootstrap-owner",
        )

        prepare.assert_called_once_with(
            config,
            reservation_id="hgi-reservation",
            maintenance_session_ids=("repair-owner", "hgi-owner"),
            actor="bootstrap-owner",
            expected_daemon_instance_id="old-instance",
            expected_process_id=101,
            expected_process_creation_time="old-created",
        )
        capture_predecessor.assert_called_once_with(runtime_descriptor.return_value)
        shutdown.assert_called_once_with(captured_handle)
        wait_predecessor.assert_called_once_with(captured_handle)
        start_successor.assert_called_once_with(runtime_descriptor.return_value)
        wait_successor.assert_called_once_with(
            config, predecessor_instance_id="old-instance", predecessor_pid=101
        )
        validate.assert_called_once_with(
            config, action_id="proof-action", reservation_id="hgi-reservation"
        )
        self.assertFalse(result["ready"])
        self.assertEqual("successor-instance", result["successorInstanceId"])
        self.assertEqual(50, result["successorSchemaVersion"])
        close_predecessor.assert_called_once_with(captured_handle)
        self.assertEqual(["capture", "proof", "shutdown", "wait"], call_order)

    @mock.patch("tools.session_coordinator.cli.bootstrap_proof_bound_handoff")
    def test_bootstrap_handoff_routes_only_existing_reservation_and_maintenance_scope(
        self, bootstrap
    ) -> None:
        bootstrap.return_value = {"proofBound": True, "reservationId": "hgi-reservation"}
        arguments = cli._parser().parse_args(
            [
                "--repo-root",
                ".",
                "bootstrap-handoff",
                "--reservation-id",
                "hgi-reservation",
                "--maintenance-session-id",
                "repair-owner",
                "--maintenance-session-id",
                "hgi-owner",
                "--actor",
                "bootstrap-owner",
            ]
        )

        result = cli._run(arguments)

        self.assertEqual({"proofBound": True, "reservationId": "hgi-reservation"}, result)
        bootstrap.assert_called_once_with(
            cli._config(arguments),
            reservation_id="hgi-reservation",
            maintenance_session_ids=("repair-owner", "hgi-owner"),
            actor="bootstrap-owner",
        )

    @mock.patch("tools.session_coordinator.cli.CoordinatorClient.from_runtime")
    def test_prepare_routes_registered_session_to_controlled_topology_action(self, from_runtime) -> None:
        client = from_runtime.return_value
        client.execute_control_action.return_value = {
            "status": "succeeded",
            "result": {"runId": "run-1", "topologyVersionId": "version-1"},
        }
        arguments = cli._parser().parse_args(
            [
                "--repo-root",
                ".",
                "milestone",
                "prepare",
                "--session-id",
                "session-a",
                "--milestone",
                "M1",
            ]
        )

        result = cli._run(arguments)

        self.assertEqual({"runId": "run-1", "topologyVersionId": "version-1"}, result)
        client.execute_control_action.assert_called_once_with(
            "topology.refresh",
            {"sessionId": "session-a", "milestoneId": "M1"},
            reason="prepare milestone M1",
        )

    @mock.patch("tools.session_coordinator.cli.CoordinatorClient.from_runtime")
    def test_review_routes_a_distinct_reviewer_to_the_workflow_gate_refresh(self, from_runtime) -> None:
        client = from_runtime.return_value
        client.execute_control_action.return_value = {
            "status": "succeeded",
            "result": {"review": {"verdict": "accepted"}, "gates": {"refreshed": True}},
        }
        arguments = cli._parser().parse_args(
            [
                "milestone",
                "review",
                "--session-id",
                "reviewer-b",
                "--executor-session-id",
                "session-a",
                "--run-id",
                "run-1",
                "--milestone",
                "M1",
                "--critical-count",
                "0",
                "--important-count",
                "0",
                "--summary",
                "architecture accepted",
            ]
        )

        result = cli._run(arguments)

        self.assertEqual({"verdict": "accepted"}, result["review"])
        client.execute_control_action.assert_called_once_with(
            "topology.refresh",
            {
                "sessionId": "reviewer-b",
                "executorSessionId": "session-a",
                "runId": "run-1",
                "milestoneId": "M1",
                "criticalCount": 0,
                "importantCount": 0,
                "summary": "architecture accepted",
            },
            reason="submit independent review for M1",
        )

    @mock.patch("tools.session_coordinator.cli.CoordinatorClient.from_runtime")
    def test_commit_routes_executor_summary_to_the_controlled_action(self, from_runtime) -> None:
        client = from_runtime.return_value
        client.execute_control_action.return_value = {
            "status": "succeeded",
            "result": {"commitSha": "abc123", "message": "feat(runtime): add cache diagnostics"},
        }
        arguments = cli._parser().parse_args(
            [
                "milestone",
                "commit",
                "--session-id",
                "session-a",
                "--run-id",
                "run-1",
                "--milestone",
                "M1",
                "--summary",
                "add cache diagnostics",
            ]
        )

        result = cli._run(arguments)

        self.assertEqual("abc123", result["commitSha"])
        client.execute_control_action.assert_called_once_with(
            "milestone.commit",
            {
                "sessionId": "session-a",
                "runId": "run-1",
                "milestoneId": "M1",
                "summary": "add cache diagnostics",
            },
            reason="commit M1 with context: add cache diagnostics",
        )


if __name__ == "__main__":
    unittest.main()
