from copy import deepcopy
import unittest
from unittest.mock import Mock

from tools.jenkins.pilot.acceptance import _require_terminal_build
from tools.jenkins.pilot.contracts import PilotError, RequestIdentity


class AcceptanceTests(unittest.TestCase):
    def setUp(self):
        self.identity = RequestIdentity("session", "request", "attempt", 1, "a" * 64, "python-static")
        parameters = {**self.identity.parameters(), "BUNDLE_HASH": "b" * 64}
        self.build = {"number": 3, "building": False, "result": "SUCCESS", "actions": [
            {"parameters": [{"name": name, "value": value} for name, value in parameters.items()]}]}

    def test_current_build_must_confirm_terminal_success(self):
        api = Mock()
        api.api.return_value = self.build
        _require_terminal_build(api, "pilot", 3, self.identity, "b" * 64)
        api.api.assert_called_once()
        for key, value in (("building", True), ("building", None), ("result", "FAILURE"),
                           ("number", 4), ("number", True)):
            api.api.return_value = {**self.build, key: value}
            with self.subTest(key=key, value=value), self.assertRaises(PilotError):
                _require_terminal_build(api, "pilot", 3, self.identity, "b" * 64)

    def test_current_build_parameters_must_match_input_and_attempt(self):
        api = Mock()
        for name, value in (("INPUT_HASH", "c" * 64), ("BUNDLE_HASH", "c" * 64),
                            ("ATTEMPT_ID", "older"), ("REQUEST_ID", "foreign")):
            changed = deepcopy(self.build)
            parameter = next(item for item in changed["actions"][0]["parameters"] if item["name"] == name)
            parameter["value"] = value
            api.api.return_value = changed
            with self.subTest(parameter=name), self.assertRaises(PilotError):
                _require_terminal_build(api, "pilot", 3, self.identity, "b" * 64)


if __name__ == "__main__":
    unittest.main()
