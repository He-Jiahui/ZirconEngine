from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.deployment.manager import DeploymentManager


class ManagerTests(unittest.TestCase):
    def test_reconcile_marks_identity_unknown_without_killing(self):
        class S:
            controller = {"listenAddress": "127.0.0.1", "httpPort": 1}
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); paths = type("P", (), {"state": root, "logs": root, "repo": root})()
            m = DeploymentManager(S(), paths, root / "java", root / "war")
            m._write({"pid": 42, "birth": "old", "executable": "x", "state": "starting"})
            with patch("tools.jenkins.deployment.manager.identity", return_value={"pid":42,"birth":"new","executable":"x"}), patch.object(m, "health", return_value={"ready":False}):
                result = m.reconcile()
            self.assertFalse(result["observedAlive"])


if __name__ == "__main__": unittest.main()
