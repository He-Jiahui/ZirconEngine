"""Workspace defaults and real command previews for bounded module validation."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[3]


class BuildReuseTests(unittest.TestCase):
    def test_independent_workspaces_use_compact_profiles_and_retain_hot_runtime_incremental(self):
        for relative in ("Cargo.toml", "zircon_plugins/Cargo.toml", "examples/woc/native/Cargo.toml"):
            with self.subTest(workspace=relative):
                manifest = tomllib.loads((ROOT / relative).read_text(encoding="utf-8"))
                for name in ("dev", "test"):
                    profile = manifest["profile"][name]
                    self.assertEqual(profile["debug"], 0)
                    self.assertIs(profile["incremental"], False)
                    self.assertIs(profile["package"]["zircon_runtime"]["incremental"], True)

    def preview(self, *arguments):
        powershell = shutil.which("powershell.exe")
        if os.name != "nt" or powershell is None:
            self.skipTest("Windows PowerShell command preview")
        result = subprocess.run([powershell, "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
            "-File", str(ROOT / "tools/dev/dev-module-build.ps1"), "-DryRun", *arguments],
            cwd=ROOT, capture_output=True, timeout=30)
        return result.returncode, json.loads(result.stdout.decode("utf-8-sig"))

    def test_default_preview_selects_only_core_min_without_default_features(self):
        status, value = self.preview()
        self.assertEqual(status, 0)
        self.assertEqual(value["package"], "zircon_runtime")
        arguments = value["cargoArguments"]
        self.assertEqual(arguments[0], "check")
        self.assertIn("--no-default-features", arguments)
        self.assertEqual(arguments[arguments.index("--features") + 1], "core-min")
        self.assertEqual(arguments.count("--package"), 1)
        self.assertFalse(value["commandExecuted"])
        self.assertTrue(set(arguments).isdisjoint({"--workspace", "--all-features", "--all-targets"}))

    def test_leaf_package_does_not_acquire_runtime_features(self):
        status, value = self.preview("-Package", "zr_math")
        self.assertEqual(status, 0)
        self.assertEqual(value["package"], "zr_math")
        self.assertNotIn("--features", value["cargoArguments"])

    def test_test_command_requires_a_bounded_target(self):
        status, value = self.preview("-Action", "test")
        self.assertEqual(status, 2)
        self.assertIn("exactly one", value["message"])

    def test_library_tests_keep_package_and_filter(self):
        status, value = self.preview("-Action", "test", "-Package", "zr_math", "-LibTests",
                                     "-TestFilter", "vector")
        self.assertEqual(status, 0)
        arguments = value["cargoArguments"]
        self.assertIn("--lib", arguments)
        self.assertIn("vector", arguments)
        self.assertEqual(arguments[-3:], ["--", "--test-threads", "1"])

    def test_one_integration_target_does_not_enable_all_tests(self):
        status, value = self.preview("-Action", "test", "-TestTarget", "module_contract", "-NoRun")
        self.assertEqual(status, 0)
        arguments = value["cargoArguments"]
        self.assertEqual(arguments[arguments.index("--test") + 1], "module_contract")
        self.assertIn("--no-run", arguments)
        self.assertNotIn("--tests", arguments)
        self.assertNotIn("--", arguments)


if __name__ == "__main__":
    unittest.main()
