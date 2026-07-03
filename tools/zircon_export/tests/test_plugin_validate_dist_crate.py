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


class PluginValidateDistCrateTests(unittest.TestCase):
    def test_plugin_validate_accepts_dist_package_contract(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)

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

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 0)
            self.assertEqual(report["command"], "plugin validate")
            self.assertFalse(report["fatal"])
            self.assertEqual(report["requested_plugin_id"], "native_dynamic_fixture")
            self.assertEqual(report["package_id"], "native_dynamic_fixture")
            self.assertEqual(report["dist_crate"], crate_name)
            self.assertEqual(report["diagnostics"], [])

    def test_plugin_validate_reports_missing_dist_crate(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(
                repo_root,
                "zircon_plugin_missing_dist",
            )

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

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin native_dynamic_fixture distribution dist_crate "
                "zircon_plugin_missing_dist is not a cdylib workspace member",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_dist_crate_missing_dist_feature(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root
                / "zircon_plugins"
                / "native_dynamic_fixture"
                / "native"
                / "Cargo.toml",
                "dist = []",
                "",
            )

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

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin native_dynamic_fixture dist crate "
                "zircon_plugin_native_dynamic_fixture_native must declare Cargo feature dist",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_malformed_dist_feature_entry(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root
                / "zircon_plugins"
                / "native_dynamic_fixture"
                / "native"
                / "Cargo.toml",
                "dist = []",
                "dist = [1]",
            )

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

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin native_dynamic_fixture dist crate "
                "zircon_plugin_native_dynamic_fixture_native Cargo feature dist[0] "
                "must be a non-empty trimmed string",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_dist_crate_missing_sdk_native_dependency(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root
                / "zircon_plugins"
                / "native_dynamic_fixture"
                / "native"
                / "Cargo.toml",
                'zircon_plugin_sdk = { workspace = true, default-features = false, features = ["native"] }',
                "",
            )

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

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin native_dynamic_fixture dist crate "
                "zircon_plugin_native_dynamic_fixture_native must depend on zircon_plugin_sdk",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_dist_feature_forbidden_zircon_runtime_feature_route(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root
                / "zircon_plugins"
                / "native_dynamic_fixture"
                / "native"
                / "Cargo.toml",
                "dist = []",
                'dist = ["zircon_runtime/native"]',
            )

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

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin native_dynamic_fixture dist crate "
                "zircon_plugin_native_dynamic_fixture_native Cargo feature dist "
                "must not enable forbidden dependency zircon_runtime",
                report["diagnostics"],
            )
