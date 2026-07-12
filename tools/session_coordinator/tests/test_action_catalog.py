from __future__ import annotations

import dataclasses
import unittest

from tools.session_coordinator.control_plane.actions.catalog import ACTION_CATALOG, action_spec
from tools.session_coordinator.control_plane.actions.models import ActionKind, ActionRisk
from tools.session_coordinator.models import CoordinatorError, WebControlRole


class ActionCatalogTests(unittest.TestCase):
    def test_catalog_is_closed_typed_and_enables_only_reviewed_red_actions(self) -> None:
        expected = {kind.value for kind in ActionKind}
        self.assertEqual(expected, set(ACTION_CATALOG))
        self.assertTrue(all(dataclasses.is_dataclass(spec) for spec in ACTION_CATALOG.values()))
        enabled_red = {
            spec.kind
            for spec in ACTION_CATALOG.values()
            if spec.risk is ActionRisk.RED and spec.enabled
        }
        self.assertEqual(
            {
                ActionKind.MILESTONE_COMMIT,
                ActionKind.SESSION_COMPLETE,
                ActionKind.SERVICE_STOP,
                ActionKind.SERVICE_RESTART,
                ActionKind.SERVICE_FORCE_STOP,
            },
            enabled_red,
        )
        forbidden = {"shell", "command", "git", "cargo", "sql", "path", "webhook"}
        self.assertFalse(
            forbidden.intersection(" ".join(ACTION_CATALOG).casefold().split("."))
        )

    def test_action_spec_is_immutable_and_unknown_kinds_are_rejected(self) -> None:
        spec = action_spec(ActionKind.SESSION_HEARTBEAT.value)
        with self.assertRaises(dataclasses.FrozenInstanceError):
            spec.enabled = False  # type: ignore[misc]
        with self.assertRaises(CoordinatorError) as rejected:
            action_spec("shell.execute")
        self.assertEqual("action_kind_unknown", rejected.exception.code)

    def test_parameter_parser_rejects_browser_paths_and_commands(self) -> None:
        spec = action_spec(ActionKind.LEASE_CLAIM.value)
        with self.assertRaises(CoordinatorError) as rejected:
            spec.parse_parameters(
                {"sessionId": "session-a", "paths": ["../../outside"], "command": "git status"}
            )
        self.assertEqual("action_parameters_invalid", rejected.exception.code)
        self.assertEqual(WebControlRole.OPERATOR, spec.required_role)

    def test_topology_refresh_accepts_only_complete_typed_review(self) -> None:
        spec = action_spec(ActionKind.TOPOLOGY_REFRESH.value)
        basic = spec.parse_parameters({"sessionId": "session-a"})
        self.assertEqual({"sessionId": "session-a"}, basic.to_payload())
        review = spec.parse_parameters(
            {
                "sessionId": "session-a",
                "executorSessionId": "session-b",
                "runId": "run-a",
                "milestoneId": "M1",
                "criticalCount": 0,
                "importantCount": 0,
                "summary": "independent review accepted",
            }
        )
        self.assertEqual("M1", review.to_payload()["milestoneId"])
        self.assertEqual("session-b", review.to_payload()["executorSessionId"])
        with self.assertRaises(CoordinatorError) as self_review:
            spec.parse_parameters(
                {
                    "sessionId": "session-a", "executorSessionId": "session-a",
                    "runId": "run-a", "milestoneId": "M1", "criticalCount": 0,
                    "importantCount": 0, "summary": "self approval",
                }
            )
        self.assertEqual("workflow_review_not_independent", self_review.exception.code)
        with self.assertRaises(CoordinatorError):
            spec.parse_parameters(
                {"sessionId": "session-a", "summary": "incomplete browser payload"}
            )

    def test_lifecycle_parameters_are_service_scoped_and_bounded(self) -> None:
        for kind in (
            ActionKind.SERVICE_DRAIN,
            ActionKind.SERVICE_RESUME,
            ActionKind.SERVICE_STOP,
            ActionKind.SERVICE_RESTART,
            ActionKind.SERVICE_FORCE_STOP,
        ):
            spec = action_spec(kind.value)
            self.assertFalse(spec.session_bound)
            self.assertEqual(
                {"timeoutSeconds": 30},
                spec.parse_parameters({"timeoutSeconds": 30}).to_payload(),
            )
            with self.assertRaises(CoordinatorError):
                spec.parse_parameters({"timeoutSeconds": 301})
            with self.assertRaises(CoordinatorError):
                spec.parse_parameters(
                    {"timeoutSeconds": 30, "command": "Stop-Process -Id 1"}
                )

    def test_codex_reconcile_is_maintainer_only_and_parameterless(self) -> None:
        spec = action_spec(ActionKind.CODEX_RECONCILE.value)

        self.assertEqual(WebControlRole.MAINTAINER, spec.required_role)
        self.assertFalse(spec.session_bound)
        self.assertEqual({}, spec.parse_parameters({}).to_payload())
        for payload in (
            {"path": "C:/Users/private/.codex"},
            {"threadId": "thread-one"},
            {"payload": {"prompt": "secret"}},
        ):
            with self.assertRaises(CoordinatorError):
                spec.parse_parameters(payload)


if __name__ == "__main__":
    unittest.main()
