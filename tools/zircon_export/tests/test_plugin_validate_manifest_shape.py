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


def _write_manifest_shape_fixture(repo_root: Path) -> Path:
    crate_name = "zircon_plugin_native_dynamic_fixture_native"
    _write_dist_plugin_workspace(repo_root, crate_name)
    _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
    return repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml"


class PluginValidateManifestShapeTests(unittest.TestCase):
    def test_plugin_validate_rejects_manifest_identity_and_display_name_shape(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_manifest_shape_fixture(repo_root)
            _replace_manifest_line(
                plugin_manifest_path,
                'id = "native_dynamic_fixture"',
                'id = "Native__dynamic_fixture"',
            )
            _replace_manifest_line(
                plugin_manifest_path,
                'display_name = "Native Dynamic Fixture"',
                'display_name = " Native Dynamic Fixture "',
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture id Native__dynamic_fixture must "
                "contain only lowercase ASCII letters, digits, underscores, "
                "and dots in non-empty segments",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture id Native__dynamic_fixture must "
                "start with a lowercase ASCII letter",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture id Native__dynamic_fixture "
                "segments must not end with an underscore or contain repeated "
                "underscores",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture display_name must be a "
                "non-empty trimmed string",
                diagnostics,
            )

    def test_plugin_validate_rejects_manifest_version_shape(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_manifest_shape_fixture(repo_root)
            _replace_manifest_line(
                plugin_manifest_path,
                'sdk_api_version = "0.1.0"',
                'sdk_api_version = "1.two.3"',
            )
            _replace_manifest_line(
                plugin_manifest_path,
                'version = "0.1.0"',
                'version = "1.2"',
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture version 1.2 must use "
                "MAJOR.MINOR.PATCH form",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture sdk_api_version 1.two.3 minor "
                "component two must contain ASCII digits",
                diagnostics,
            )

    def test_plugin_validate_rejects_manifest_version_numeric_boundaries(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_manifest_shape_fixture(repo_root)
            _replace_manifest_line(
                plugin_manifest_path,
                'sdk_api_version = "0.1.0"',
                'sdk_api_version = "4294967296.0.0"',
            )
            _replace_manifest_line(
                plugin_manifest_path,
                'version = "0.1.0"',
                'version = "01.2.3"',
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture version 01.2.3 major component "
                "01 must not use leading zeroes",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture sdk_api_version 4294967296.0.0 "
                "major component 4294967296 must fit in u32",
                diagnostics,
            )

    def test_plugin_validate_rejects_unknown_root_manifest_fields(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_manifest_shape_fixture(repo_root)
            _replace_manifest_line(
                plugin_manifest_path,
                'id = "native_dynamic_fixture"',
                'id = "native_dynamic_fixture"\nunknown_root = "drift"',
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin native_dynamic_fixture unknown_root "
                "is not a known manifest root field",
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
