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
    _write_complete_sound_manifest,
)
from tools.zircon_export.tests.test_plugin_build import (
    _write_dist_plugin_workspace,
    _write_feature_provider_workspace,
)


class PluginValidateDistributionModulesTests(unittest.TestCase):
    def test_plugin_validate_reports_dist_crate_not_declared_by_root_module(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                f'crate_name = "{crate_name}"',
                'crate_name = "zircon_plugin_native_dynamic_fixture_wrong_module"',
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
                "plugin native_dynamic_fixture distribution.dist_crate "
                "zircon_plugin_native_dynamic_fixture_native is not declared by any module crate_name",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_runtime_entry_without_runtime_target_mode(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                f'crate_name = "{crate_name}"\n'
                'target_modes = ["client_runtime", "editor_host"]',
                f'crate_name = "{crate_name}"\n'
                'target_modes = ["editor_host"]',
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
                "plugin native_dynamic_fixture distribution.runtime_entry "
                "requires dist module target_modes to include client_runtime or server_runtime",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_editor_entry_without_editor_target_mode(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                'editor_entry = "zircon_native_dynamic_fixture_editor_entry_v3"',
            )
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                f'crate_name = "{crate_name}"\n'
                'target_modes = ["client_runtime", "editor_host"]',
                f'crate_name = "{crate_name}"\n'
                'target_modes = ["client_runtime"]',
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
                "plugin native_dynamic_fixture distribution.editor_entry "
                "requires dist module target_modes to include editor_host",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_unknown_dist_module_target_mode(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                f'crate_name = "{crate_name}"\n'
                'target_modes = ["client_runtime", "editor_host"]',
                f'crate_name = "{crate_name}"\n'
                'target_modes = ["client_runtime", "editor_host", "desktop"]',
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
                'plugin native_dynamic_fixture distribution.dist_crate '
                'zircon_plugin_native_dynamic_fixture_native modules[0].target_modes[2] '
                '"desktop" is unsupported; expected one of client_runtime, '
                "server_runtime, editor_host",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_dist_crate_not_declared_by_feature_module(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_sound_timeline_animation_dist"
            _write_feature_provider_workspace(repo_root, crate_name)
            _write_complete_sound_manifest(repo_root)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "sound" / "plugin.toml",
                f'crate_name = "{crate_name}"',
                'crate_name = "zircon_plugin_sound_timeline_animation_wrong_module"',
            )

            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                exit_code = main(
                    [
                        "plugin",
                        "validate",
                        "sound_timeline_animation_track",
                        "--repo-root",
                        str(repo_root),
                        "--json",
                    ]
                )

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin sound_timeline_animation_track distribution.dist_crate "
                "zircon_plugin_sound_timeline_animation_dist is not declared by any module crate_name",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_feature_runtime_entry_without_runtime_target_mode(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_sound_timeline_animation_dist"
            _write_feature_provider_workspace(repo_root, crate_name)
            _write_complete_sound_manifest(repo_root)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "sound" / "plugin.toml",
                f'crate_name = "{crate_name}"\n'
                'target_modes = ["client_runtime", "editor_host"]',
                f'crate_name = "{crate_name}"\n'
                'target_modes = ["editor_host"]',
            )

            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                exit_code = main(
                    [
                        "plugin",
                        "validate",
                        "sound_timeline_animation_track",
                        "--repo-root",
                        str(repo_root),
                        "--json",
                    ]
                )

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin sound_timeline_animation_track distribution.runtime_entry "
                "requires dist module target_modes to include client_runtime or server_runtime",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_feature_unknown_dist_module_target_mode(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_sound_timeline_animation_dist"
            _write_feature_provider_workspace(repo_root, crate_name)
            _write_complete_sound_manifest(repo_root)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "sound" / "plugin.toml",
                f'crate_name = "{crate_name}"\n'
                'target_modes = ["client_runtime", "editor_host"]',
                f'crate_name = "{crate_name}"\n'
                'target_modes = ["client_runtime", "desktop"]',
            )

            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                exit_code = main(
                    [
                        "plugin",
                        "validate",
                        "sound_timeline_animation_track",
                        "--repo-root",
                        str(repo_root),
                        "--json",
                    ]
                )

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                'plugin sound_timeline_animation_track distribution.dist_crate '
                'zircon_plugin_sound_timeline_animation_dist modules[1].target_modes[1] '
                '"desktop" is unsupported; expected one of client_runtime, '
                "server_runtime, editor_host",
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
