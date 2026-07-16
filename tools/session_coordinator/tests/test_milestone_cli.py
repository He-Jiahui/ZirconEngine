from __future__ import annotations

import unittest
from unittest import mock

from tools.session_coordinator import cli
from tools.session_coordinator.client import CoordinatorClient
from tools.session_coordinator.config import DEFAULT_COORDINATOR_PORT


class MilestoneControlClientTests(unittest.TestCase):
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
            {"sessionId": "session-a"},
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
