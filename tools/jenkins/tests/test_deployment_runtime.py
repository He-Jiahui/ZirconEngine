from __future__ import annotations

import contextlib
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.deployment.lifecycle import check_port_available, launch_command, wait_for_health
from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment.manager import DeploymentManager
from tools.jenkins.deployment.__main__ import main


class DeploymentRuntimeTests(unittest.TestCase):
    def test_start_cli_reports_missing_boot_evidence_as_blocked_json(self):
        output = io.StringIO()
        with patch.object(sys, "argv", ["jenkins-deployment", "start"]), \
                patch("tools.jenkins.deployment.__main__.load_spec", side_effect=JenkinsError(
                    "boot_evidence_unavailable", "Windows boot time could not be established")), \
                contextlib.redirect_stdout(output):
            status = main()
        result = json.loads(output.getvalue())
        self.assertEqual(2, status)
        self.assertEqual("blocked", result["status"])
        self.assertEqual("boot_evidence_unavailable", result["reasonCode"])
        self.assertFalse(result["retryable"])

    def test_port_preflight_rejects_an_occupied_endpoint(self):
        import socket
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as owner:
            owner.bind(("127.0.0.1", 0)); owner.listen(1)
            with self.assertRaises(JenkinsError) as raised:
                check_port_available("127.0.0.1", owner.getsockname()[1])
        self.assertEqual("controller_port_unavailable", raised.exception.code)

    def test_health_wait_stops_when_owned_root_exits(self):
        class Exited:
            def poll(self):
                return 1
        self.assertFalse(wait_for_health("http://127.0.0.1:1", timeout=5, process=Exited()))

    def test_launch_command_uses_repo_home_and_external_temp(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            paths = type("Paths", (), {"home": root / "home", "tmp": root / "tmp",
                                       "cache": root / "cache"})()
            java = root / "java.exe"; war = root / "jenkins.war"
            with patch("tools.jenkins.deployment.lifecycle.java_major", return_value=21):
                args, env = launch_command(paths, java, war, 53748)
            self.assertEqual(env["JENKINS_HOME"], str(paths.home))
            self.assertEqual(env["TEMP"], str(paths.tmp))
            self.assertIn("--httpPort=53748", args)
            self.assertIn(str(paths.cache / "plugins"), " ".join(args))

    def test_health_requires_api_token_and_does_not_print_it(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); state = root / "state"; state.mkdir()
            token = "secret-token-value"
            (state / "credentials.json").write_text(json.dumps({
                "authenticationMode": "api-token", "username": "admin", "apiToken": token
            }), encoding="utf-8")
            paths = type("Paths", (), {"state": state, "root": root, "home": root / "home"})()
            spec = type("Spec", (), {"controller": {"listenAddress": "127.0.0.1", "httpPort": 53748},
                                      "agent": {"name": "zircon-windows-agent"}})()
            manager = DeploymentManager(spec, paths, root / "java", root / "war")
            with patch.object(manager, "_request", side_effect=RuntimeError("offline")):
                result = manager.health()
            self.assertFalse(result["ready"])
            self.assertNotIn(token, json.dumps(result))


if __name__ == "__main__":
    unittest.main()
