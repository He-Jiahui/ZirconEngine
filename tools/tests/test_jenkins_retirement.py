"""Retired workflow adapters must be terminal without state/service side effects."""
from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import unittest
from unittest.mock import patch

from tools.build import build_editor, zircon_build
from tools.jenkins import frontend
from tools.jenkins.contracts import JenkinsError

ROOT = Path(__file__).resolve().parents[2]


def management_library():
    spec = importlib.util.spec_from_file_location("retirement_management_fixture", ROOT / "tools/jenkins-management/entry.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class JenkinsRetirementTests(unittest.TestCase):
    def assert_terminal(self, invoke):
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            self.assertEqual(2, invoke())
        result = json.loads(output.getvalue())
        self.assertEqual("retired", result["status"])
        self.assertEqual("jenkins_workflow_retired", result["reasonCode"])
        self.assertFalse(result["commandExecuted"])
        self.assertFalse(result["hostServicesChanged"])
        self.assertFalse(result["historicalDataChanged"])
        self.assertFalse(result["formalAcceptance"])

    def test_frontend_cli_never_reads_gate_or_loads_runtime(self):
        with patch.object(frontend, "sole_entry_enforced", side_effect=AssertionError("gate read")), \
                patch.object(frontend, "management_entry", side_effect=AssertionError("capsule read")), \
                patch.object(frontend, "State", side_effect=AssertionError("database opened")):
            for action in ("gate", "dispatch", "capability"):
                with self.subTest(action=action):
                    self.assert_terminal(lambda: frontend.main([action, "--repo-root", str(ROOT)]))

    def test_stale_environment_adapter_is_terminal_for_both_old_routes(self):
        with patch.dict(os.environ, {"ZIRCON_JENKINS_IDENTITY_FILE": "not-an-identity-file"}), \
                patch.object(frontend, "State", side_effect=AssertionError("database opened")), \
                patch.object(frontend, "_read_json", side_effect=AssertionError("identity read")), \
                patch.object(frontend, "management_client_path", side_effect=AssertionError("capsule read")):
            for functional in (False, True):
                with self.subTest(functional=functional), self.assertRaises(JenkinsError) as raised:
                    frontend.entry_from_environment(ROOT, functional=functional)
                self.assertEqual("jenkins_workflow_retired", raised.exception.code)
            with self.assertRaises(JenkinsError) as raised:
                frontend.management_entry(ROOT)
            self.assertEqual("jenkins_workflow_retired", raised.exception.code)

    def test_management_cli_never_dispatches_even_with_explicit_old_input(self):
        module = management_library()
        with patch.object(module, "dispatch", side_effect=AssertionError("dispatch")), \
                patch.object(module, "capability", side_effect=AssertionError("activation")), \
                patch.object(module, "cargo_entry", side_effect=AssertionError("Cargo")):
            for args in (["capability", "--repo-root", str(ROOT)],
                         ["cargo", "--identity-file", "", "--repo-root", str(ROOT), "--", "check"]):
                with self.subTest(args=args):
                    self.assert_terminal(lambda: module.main(args))

    def test_product_query_is_local_even_with_stale_identity(self):
        with patch.dict(os.environ, {"ZIRCON_JENKINS_IDENTITY_FILE": "not-an-identity-file"}), \
                patch.object(frontend, "entry_from_environment", side_effect=AssertionError("dispatch")), \
                patch.object(zircon_build, "discover_plugins", return_value=[]) as discover, \
                patch.object(zircon_build, "print_plugin_catalog") as display, \
                patch.object(zircon_build, "build", side_effect=AssertionError("compiled")):
            self.assertEqual(0, zircon_build.main(["--list-plugins"]))
        discover.assert_called_once()
        display.assert_called_once_with([])

    def test_editor_rejects_explicit_retired_identity_including_empty(self):
        for value in ("", "original.json"):
            with self.subTest(value=value), patch.object(build_editor, "source_inventory", side_effect=AssertionError("source read")), \
                    contextlib.redirect_stderr(io.StringIO()) as output:
                code = build_editor.main(["--repo-root", str(ROOT), "--jenkins-identity-file", value])
                self.assertNotEqual(0, code)
                self.assertIn("retired", output.getvalue())


if __name__ == "__main__":
    unittest.main()
