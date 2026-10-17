"""Boundary cases for the Windows engine path budget."""
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from tools.audits.path_names import audit, files, owned_path, path_units, RELATIVE_LIMIT


class PathNamesTests(unittest.TestCase):
    def test_utf16_budget_counts_surrogate_pairs(self):
        self.assertEqual(path_units("a\U0001f600"), 3)

    def test_relative_and_checkout_limits(self):
        path = "a" * RELATIVE_LIMIT
        self.assertTrue(audit([path])["passed"])
        self.assertFalse(audit([path + "a"])["passed"])
        self.assertFalse(audit([path], "E:/" + "x" * 49)["passed"])

    def test_generated_outputs_are_excluded_but_source_target_modules_are_owned(self):
        self.assertFalse(owned_path("zircon_runtime/target/debug/a"))
        self.assertFalse(owned_path("zircon_plugins/a/.zircon-cache/a"))
        self.assertTrue(owned_path("zircon_runtime/src/export/target/mod.rs"))

    def test_file_discovery_without_host_tools_preserves_audit_scope(self):
        included = [
            "zircon_runtime/src/lib.rs",
            "zircon_runtime/src/export/target/mod.rs",
            "zircon_plugins/.hidden/plugin.toml",
            "zircon_plugins/ignored_by_git/plugin.toml",
        ]
        excluded = [
            "tools/script.py",
            "zircon_runtime/target/debug/build.rs",
            "zircon_runtime/vendor/dependency.rs",
            "zircon_plugins/node_modules/package.json",
            "zircon_plugins/.git/config",
            "zircon_plugins/.cache/cache.json",
            "zircon_plugins/.zircon-cache/cache.json",
            "zircon_plugins/.zircon/cache/cache.json",
            "zircon_runtime/src/lib.rs.tmp",
        ]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in included + excluded:
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.touch()
            (root / ".gitignore").write_text("ignored_by_git/\n", encoding="utf-8")
            with patch.dict(os.environ, {"PATH": ""}):
                self.assertEqual(files(root), sorted(included))

    def test_file_discovery_in_empty_checkout(self):
        with tempfile.TemporaryDirectory() as directory:
            self.assertEqual(files(Path(directory)), [])


if __name__ == "__main__":
    unittest.main()
