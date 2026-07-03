from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.cli import main
from tools.zircon_export.tests.plugin_validate_support import (
    _write_complete_native_dynamic_fixture_manifest,
)
from tools.zircon_export.tests.test_plugin_build import _write_dist_plugin_workspace


def _run_plugin_validate(repo_root: Path) -> tuple[int, dict[str, object]]:
    output = io.StringIO()
    with contextlib.redirect_stdout(output):
        exit_code = main(
            [
                "plugin",
                "validate",
                "native_dynamic_fixture",
                "--repo-root",
                str(repo_root),
                "--json",
            ]
        )
    return exit_code, json.loads(output.getvalue())


def _write_capabilities_fixture(repo_root: Path) -> Path:
    crate_name = "zircon_plugin_native_dynamic_fixture_native"
    _write_dist_plugin_workspace(repo_root, crate_name)
    _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
    return repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml"


def _replace_first_manifest_line(manifest_path: Path, old: str, new: str) -> None:
    text = manifest_path.read_text(encoding="utf-8")
    if old not in text:
        raise AssertionError(f"manifest fixture is missing line: {old}")
    manifest_path.write_text(text.replace(old, new, 1), encoding="utf-8")


class PluginValidateCapabilitiesTests(unittest.TestCase):
    def test_plugin_validate_rejects_malformed_root_capabilities(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_capabilities_fixture(repo_root)
            _replace_first_manifest_line(
                plugin_manifest_path,
                'capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                'capabilities = ["badcap", "runtime..bad", "Runtime.Bad", "runtime..bad"]',
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture capabilities[0] badcap "
                "should use at least two dot-separated namespace segments",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture capabilities[1] runtime..bad "
                "should not contain empty namespace segments",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture capabilities[2] Runtime.Bad "
                "should contain only lowercase ASCII letters, digits, underscores, and dots",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture capabilities[3] runtime..bad "
                "duplicates capabilities capabilities[1]",
                diagnostics,
            )

    def test_plugin_validate_rejects_empty_root_capabilities(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_capabilities_fixture(repo_root)
            _replace_first_manifest_line(
                plugin_manifest_path,
                'capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                "capabilities = []",
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "plugin native_dynamic_fixture capabilities must be a non-empty string array",
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
