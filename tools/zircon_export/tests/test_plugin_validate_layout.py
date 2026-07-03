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


class PluginValidateLayoutTests(unittest.TestCase):
    def test_plugin_validate_rejects_package_coordinate_drift(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                'id = "native_dynamic_fixture"',
                "\n".join(
                    [
                        'id = "native_dynamic_fixture"',
                        'package_prefix = "com..Example"',
                        'package_company = "BadCompany"',
                    ]
                ),
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture package coordinates must declare "
                "package_prefix, package_company, and package_name together or "
                "leave all empty",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture package_prefix com..Example "
                "must contain only non-empty lowercase coordinate segments",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture package_company BadCompany "
                "must be a non-empty lowercase coordinate segment",
                diagnostics,
            )

    def test_plugin_validate_rejects_layout_public_metadata_drift(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                'category = "runtime"',
                'category = " runtime "',
            )
            _replace_manifest_line(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                'description = "Native dynamic fixture plugin."',
                'description = " Native dynamic fixture plugin. "',
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture category "
                "must be a non-empty trimmed string",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture description "
                "must be trimmed when present",
                diagnostics,
            )

    def test_plugin_validate_rejects_layout_target_and_platform_drift(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                'supported_targets = ["client_runtime", "editor_host"]',
                'supported_targets = ["client_runtime", "client_runtime", "mobile_runtime", " editor_host"]',
            )
            _replace_manifest_line(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                'supported_platforms = ["windows", "linux", "macos"]',
                'supported_platforms = ["windows", "windows", "playdate", " linux"]',
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture supported_targets[1] "
                "client_runtime duplicates supported_targets[0]",
                diagnostics,
            )
            self.assertIn(
                'plugin native_dynamic_fixture supported_targets[2] "mobile_runtime" '
                "is unsupported; expected one of client_runtime, server_runtime, editor_host",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture supported_targets[3] "
                "must be a non-empty trimmed string",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture supported_platforms[1] "
                "windows duplicates supported_platforms[0]",
                diagnostics,
            )
            self.assertIn(
                'plugin native_dynamic_fixture supported_platforms[2] "playdate" '
                "is unsupported; expected one of windows, linux, macos, android, "
                "ios, web_gpu, wasm, headless, windows-x86_64, linux-x86_64, "
                "macos-aarch64",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture supported_platforms[3] "
                "must be a non-empty trimmed string",
                diagnostics,
            )

    def test_plugin_validate_rejects_layout_root_path_drift(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                'supported_platforms = ["windows", "linux", "macos"]',
                "\n".join(
                    [
                        'supported_platforms = ["windows", "linux", "macos"]',
                        'asset_roots = ["assets", "assets", "/absolute", "textures//bad", " textures"]',
                        r"content_roots = ['content\bad', 'content/../bad']",
                    ]
                ),
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture asset_roots[1] "
                "assets duplicates asset_roots[0]",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture asset_roots[2] /absolute "
                "must be relative",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture asset_roots[3] textures//bad "
                "must not contain empty, current, or parent path segments",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture asset_roots[4] "
                "must be a non-empty trimmed string",
                diagnostics,
            )
            self.assertIn(
                r"plugin native_dynamic_fixture content_roots[0] content\bad "
                "must use forward slashes",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture content_roots[1] content/../bad "
                "must not contain empty, current, or parent path segments",
                diagnostics,
            )

    def test_plugin_validate_rejects_layout_root_drive_separator_drift(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                'supported_platforms = ["windows", "linux", "macos"]',
                "\n".join(
                    [
                        'supported_platforms = ["windows", "linux", "macos"]',
                        'asset_roots = ["C:/assets"]',
                        'content_roots = ["D:/content"]',
                    ]
                ),
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture asset_roots[0] C:/assets "
                "must not contain a drive separator",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture content_roots[0] D:/content "
                "must not contain a drive separator",
                diagnostics,
            )


if __name__ == "__main__":
    unittest.main()
