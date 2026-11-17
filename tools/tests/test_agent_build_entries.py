"""Public PowerShell entry contracts: read-only plans only, never compile Cargo."""
from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[2]
SHELL = shutil.which("pwsh") or shutil.which("powershell")


@unittest.skipUnless(SHELL and os.name == "nt", "Windows PowerShell entry contracts")
class AgentBuildEntryTests(unittest.TestCase):
    def invoke(self, script, *arguments, success=True):
        command = [SHELL, "-NoProfile", "-NonInteractive", "-File", str(ROOT / script),
                   "-PythonInterpreter", sys.executable, *arguments]
        environment = dict(os.environ, ZIRCON_JENKINS_IDENTITY_FILE="must-not-be-read",
                           JENKINS_URL="https://must-not-be-contacted.invalid")
        result = subprocess.run(command, cwd=ROOT, env=environment, capture_output=True,
                                text=True, encoding="utf-8", errors="replace", timeout=90)
        lines = [line for line in result.stdout.splitlines() if line.strip().startswith("{")]
        self.assertTrue(lines, result.stdout + result.stderr)
        plan = json.loads(lines[-1])
        if success:
            self.assertEqual(0, result.returncode, (plan, result.stderr))
            self.assertEqual("planned", plan["status"])
            self.assertEqual("Agent", plan["executionOwner"])
            self.assertFalse(plan["commandExecuted"])
            self.assertFalse(plan["formalAcceptance"])
            self.assertIn("--no-default-features", plan["cargoArguments"])
            self.assertIn("--locked", plan["cargoArguments"])
            self.assertNotIn("receipt", plan)
        else:
            self.assertNotEqual(0, result.returncode, plan)
            self.assertFalse(plan["commandExecuted"])
        return plan

    def test_module_root_exact_owner_without_jenkins_installation(self):
        plan = self.invoke("tools/dev/dev-module-build.ps1", "-Package", "zr_math", "-DryRun")
        self.assertEqual("zr_math", plan["package"])
        self.assertTrue(plan["packageManifest"].replace("\\", "/").endswith("/zircon_runtime/crates/zr_math/Cargo.toml"))
        self.assertEqual(str(ROOT / "Cargo.toml"), plan["workspaceManifest"])
        self.assertEqual([], plan["features"])

    def test_module_default_runtime_is_explicit_core_min(self):
        plan = self.invoke("tools/dev/dev-module-build.ps1", "-DryRun")
        self.assertEqual("zircon_runtime", plan["package"])
        self.assertEqual(["core-min"], plan["features"])

    def test_module_plugin_owner_has_no_hardcoded_single_recipe(self):
        package = "zircon_plugin_rendering_reflection_probes_runtime"
        plan = self.invoke("tools/dev/dev-module-build.ps1", "-Package", package, "-Action", "test",
                           "-LibTests", "-TestFilter", "consume::tests", "-DryRun")
        self.assertEqual(str(ROOT / "zircon_plugins/Cargo.toml"), plan["workspaceManifest"])
        self.assertIn("consume::tests", plan["cargoArguments"])
        self.assertIn("--lib", plan["cargoArguments"])
        separator = plan["cargoArguments"].index("--")
        self.assertEqual(["--", "--test-threads", "1"], plan["cargoArguments"][separator:])

    def test_all_named_fast_presets_read_the_canonical_features(self):
        presets = tomllib.loads((ROOT / "zircon_runtime/runtime-feature-presets.toml").read_text(encoding="utf-8"))
        for preset in presets["profiles"]:
            with self.subTest(profile=preset["id"]):
                plan = self.invoke("tools/dev/dev-fast-build.ps1", "-Profile", preset["id"], "-DryRun")
                self.assertEqual("zircon_app", plan["package"])
                self.assertEqual([preset["cargo_feature"]], plan["features"])
                # Dependency expansion syntax belongs inside Cargo.toml, not --features.
                self.assertFalse(any(value.startswith('dep:') or '?' in value for value in plan['features']))

    def test_query_only_is_local_readonly_and_keeps_exact_features(self):
        plan = self.invoke("tools/dev/dev-fast-build.ps1", "-Profile", "server", "-QueryOnly")
        self.assertTrue(plan["queryOnly"])
        self.assertEqual(["target-server"], plan["features"])

    def test_retired_identity_is_rejected_before_execution_in_both_entries(self):
        for script in ("tools/dev/dev-module-build.ps1", "tools/dev/dev-fast-build.ps1"):
            for value in ("", "identity.json"):
                with self.subTest(script=script, value=value):
                    result = self.invoke(script, "-JenkinsIdentityFile", value, "-DryRun", success=False)
                    self.assertIn("retired", result["message"])

    def test_unknown_owner_and_invalid_test_scope_are_not_silently_broadened(self):
        for arguments in (("-Package", "zr_not_a_package"),
                          ("-Action", "test"),
                          ("-Action", "test", "-LibTests", "-TestTarget", "other")):
            with self.subTest(arguments=arguments):
                self.invoke("tools/dev/dev-module-build.ps1", *arguments, "-DryRun", success=False)

    def test_unsupported_fast_options_are_rejected_not_ignored(self):
        for arguments in (("-Action", "run"), ("-Linker", "lld"), ("-StorageMode", "compact"),
                          ("-SourceSnapshot", ""), ("-PublishArtifact", "")):
            with self.subTest(arguments=arguments):
                self.invoke("tools/dev/dev-fast-build.ps1", *arguments, "-DryRun", success=False)


if __name__ == "__main__":
    unittest.main()
