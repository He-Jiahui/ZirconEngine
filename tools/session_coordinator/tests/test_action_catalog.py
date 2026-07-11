from __future__ import annotations

import dataclasses
import unittest

from tools.session_coordinator.control_plane.actions.catalog import ACTION_CATALOG, action_spec
from tools.session_coordinator.control_plane.actions.models import ActionKind, ActionRisk
from tools.session_coordinator.models import CoordinatorError, WebControlRole


class ActionCatalogTests(unittest.TestCase):
    def test_catalog_is_closed_typed_and_keeps_red_actions_disabled(self) -> None:
        expected = {kind.value for kind in ActionKind}
        self.assertEqual(expected, set(ACTION_CATALOG))
        self.assertTrue(all(dataclasses.is_dataclass(spec) for spec in ACTION_CATALOG.values()))
        self.assertTrue(
            all(
                not spec.enabled
                for spec in ACTION_CATALOG.values()
                if spec.risk is ActionRisk.RED
            )
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


if __name__ == "__main__":
    unittest.main()
