"""Actual state projection rejects partial authentication and identity observations."""
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock, patch

from tools.jenkins.tray import service


class StatusTests(unittest.TestCase):
    def observe(self, *, agent_live=True, offline=False, info=None, builds=None):
        config = SimpleNamespace(state_dir=Path(r"E:\cargo-targets\tray-test-state"))
        client = Mock()
        client.api.side_effect = [info or {"quietingDown": False}, {"offline": offline}]
        with patch.object(service, "validate_runtime", return_value={}), patch.object(service, "_runtime", return_value=({"controller": {}, "agent": {}}, {}, "http://127.0.0.1:34567/")), patch.object(service, "_owner", return_value={"live": True}), patch.object(service, "_runtime_evidence", side_effect=[True, agent_live]), patch.object(service, "_client", return_value=client), patch.object(service, "_builds", return_value=(builds or [], 0)), patch.object(service, "read_json", return_value=None):
            return service.read_status(config)

    def test_ready_requires_authenticated_api_and_live_online_agent(self):
        self.assertEqual(self.observe()["state"], "ready")
        self.assertEqual(self.observe(agent_live=False)["state"], "degraded")
        self.assertEqual(self.observe(offline=True)["state"], "degraded")

    def test_api_unknown_does_not_enable_stop(self):
        state = self.observe(info={"mode": "NORMAL"})
        self.assertEqual(state["state"], "error")
        self.assertFalse(state["canStart"])
        self.assertFalse(state["canStop"])

    def test_quiet_mode_is_visible(self):
        state = self.observe(info={"quietingDown": True})
        self.assertEqual(state["state"], "stopping")
        self.assertFalse(state["canStart"])

    def test_busy_build_is_visible(self):
        state = self.observe(builds=[{"number": 2}])
        self.assertEqual(state["state"], "busy")

    def test_unknown_native_identity_refuses_observation(self):
        with patch.object(service, "process_matches_creation_time", side_effect=OSError("denied")):
            with self.assertRaises(service.TrayError):
                service._live({"pid": 5, "creationTime": "birth"})

    def test_truncated_build_export_is_never_idle(self):
        client = Mock()
        client.api.side_effect = [{"jobs": [{"name": "zircon-pilot"}]}, {"items": []}, {"builds": []}]
        with self.assertRaises(service.TrayError):
            service._builds(client, {})


if __name__ == "__main__":
    unittest.main()
