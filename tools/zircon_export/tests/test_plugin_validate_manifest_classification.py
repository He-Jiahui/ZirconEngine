from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.cli import main
from tools.zircon_export.tests.plugin_validate_support import (
    _replace_manifest_line,
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


def _write_manifest_classification_fixture(repo_root: Path) -> Path:
    crate_name = "zircon_plugin_native_dynamic_fixture_native"
    _write_dist_plugin_workspace(repo_root, crate_name)
    _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
    return repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml"


class PluginValidateManifestClassificationTests(unittest.TestCase):
    def test_plugin_validate_rejects_missing_manifest_maturity(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_manifest_classification_fixture(repo_root)
            _replace_manifest_line(
                plugin_manifest_path,
                'maturity = "experimental"',
                "",
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin native_dynamic_fixture maturity is required",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_unknown_manifest_maturity(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_manifest_classification_fixture(repo_root)
            _replace_manifest_line(
                plugin_manifest_path,
                'maturity = "experimental"',
                'maturity = "preview"',
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin native_dynamic_fixture maturity preview is unsupported; "
                "expected one of stable, beta, experimental",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_unknown_manifest_category(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_manifest_classification_fixture(repo_root)
            _replace_manifest_line(
                plugin_manifest_path,
                'category = "runtime"',
                'category = "sandbox"',
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin native_dynamic_fixture category sandbox is unsupported; "
                "expected one of asset_importer, authoring, diagnostics, platform, rendering, runtime, sdk",
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
