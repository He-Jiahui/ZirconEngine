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
    _write_plugin_workspace_members,
)
from tools.zircon_export.tests.test_plugin_build import _write_dist_plugin_workspace


class PluginValidateAllTargetTests(unittest.TestCase):

    def test_plugin_validate_all_reports_malformed_root_distribution(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            manifest_path = (
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml"
            )
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                manifest_path,
                "[distribution]",
                'distribution = "dist"',
            )
            _replace_manifest_line(
                manifest_path,
                'default_packaging = ["native_dynamic"]',
                'distribution_default_packaging = ["native_dynamic"]',
            )

            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                exit_code = main(
                    [
                        "plugin",
                        "validate",
                        "--all",
                        "--repo-root",
                        str(repo_root),
                        "--json",
                    ]
                )

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["target_count"], 0)
            self.assertEqual(report["failed_count"], 0)
            self.assertIn(
                f"{manifest_path} distribution must be a table",
                report["diagnostics"],
            )

    def test_plugin_validate_all_reports_failed_target_diagnostics(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                'forms = ["dist"]',
                'forms = ["embed"]',
            )

            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                exit_code = main(
                    [
                        "plugin",
                        "validate",
                        "--all",
                        "--repo-root",
                        str(repo_root),
                        "--json",
                    ]
                )

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["target_count"], 1)
            self.assertEqual(report["failed_count"], 1)
            self.assertEqual(report["diagnostics"], [])
            self.assertEqual(report["items"][0]["package_id"], "native_dynamic_fixture")
            self.assertIn(
                "plugin native_dynamic_fixture distribution.forms must include dist",
                report["items"][0]["diagnostics"],
            )

    def test_plugin_validate_all_rejects_duplicate_option_keys(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            extra_crate_name = "zircon_plugin_extra_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _write_plugin_workspace_members(
                repo_root,
                [
                    "native_dynamic_fixture/native",
                    "extra_dynamic_fixture/native",
                ],
            )
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                "[distribution]",
                "\n".join(
                    [
                        "[[options]]",
                        'key = "shared.option_key"',
                        'display_name = "Shared Option"',
                        'value_type = "bool"',
                        'default_value = "false"',
                        "",
                        "[distribution]",
                    ]
                ),
            )
            extra_plugin_root = repo_root / "zircon_plugins" / "extra_dynamic_fixture"
            extra_crate_root = extra_plugin_root / "native"
            extra_crate_root.mkdir(parents=True)
            (extra_plugin_root / "plugin.toml").write_text(
                "\n".join(
                    [
                        'id = "extra_dynamic_fixture"',
                        'version = "0.1.0"',
                        'sdk_api_version = "0.1.0"',
                        'display_name = "Extra Dynamic Fixture"',
                        'category = "runtime"',
                        'description = "Extra dynamic fixture plugin."',
                        'supported_targets = ["client_runtime", "editor_host"]',
                        'supported_platforms = ["windows", "linux", "macos"]',
                        'capabilities = ["runtime.plugin.extra_dynamic_fixture"]',
                        'default_packaging = ["source_template", "library_embed", "native_dynamic"]',
                        'maturity = "experimental"',
                        "",
                        "[[options]]",
                        'key = "shared.option_key"',
                        'display_name = "Shared Option"',
                        'value_type = "bool"',
                        'default_value = "false"',
                        "",
                        "[distribution]",
                        'forms = ["dist"]',
                        'default_packaging = ["native_dynamic"]',
                        "abi_version = 3",
                        'engine_compat = ">=0.1, <0.2"',
                        f'dist_crate = "{extra_crate_name}"',
                        'descriptor_symbol = "zircon_native_plugin_descriptor_v3"',
                        'runtime_entry = "zircon_extra_dynamic_fixture_runtime_entry_v3"',
                        "",
                        "[[modules]]",
                        'name = "extra_dynamic_fixture.runtime"',
                        'kind = "runtime"',
                        f'crate_name = "{extra_crate_name}"',
                        'target_modes = ["client_runtime", "editor_host"]',
                        'capabilities = ["runtime.plugin.extra_dynamic_fixture"]',
                    ]
                ),
                encoding="utf-8",
            )
            (extra_crate_root / "Cargo.toml").write_text(
                "\n".join(
                    [
                        "[package]",
                        f'name = "{extra_crate_name}"',
                        'version = "0.1.0"',
                        'edition = "2021"',
                        "",
                        "[lib]",
                        'crate-type = ["cdylib"]',
                        "",
                        "[features]",
                        'default = ["dist"]',
                        "dist = []",
                        "",
                        "[dependencies]",
                        'zircon_plugin_sdk = { workspace = true, default-features = false, features = ["native"] }',
                    ]
                ),
                encoding="utf-8",
            )

            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                exit_code = main(
                    [
                        "plugin",
                        "validate",
                        "--all",
                        "--repo-root",
                        str(repo_root),
                        "--json",
                    ]
                )

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["target_count"], 2)
            self.assertEqual(report["failed_count"], 0)
            self.assertIn(
                "plugin validate options key shared.option_key is duplicated by "
                "plugin extra_dynamic_fixture options[0].key and "
                "plugin native_dynamic_fixture options[0].key",
                report["diagnostics"],
            )

    def test_plugin_validate_all_rejects_duplicate_asset_importer_ids(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            extra_crate_name = "zircon_plugin_extra_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _write_plugin_workspace_members(
                repo_root,
                [
                    "native_dynamic_fixture/native",
                    "extra_dynamic_fixture/native",
                ],
            )
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                "\n".join(
                    [
                        'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                        "",
                        "[[asset_importers]]",
                        'id = "shared.importer"',
                        'plugin_id = "native_dynamic_fixture"',
                        "priority = 100",
                        'source_extensions = ["json"]',
                        'output_kind = "Data"',
                        "importer_version = 1",
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                    ]
                ),
            )
            extra_plugin_root = repo_root / "zircon_plugins" / "extra_dynamic_fixture"
            extra_crate_root = extra_plugin_root / "native"
            extra_crate_root.mkdir(parents=True)
            (extra_plugin_root / "plugin.toml").write_text(
                "\n".join(
                    [
                        'id = "extra_dynamic_fixture"',
                        'version = "0.1.0"',
                        'sdk_api_version = "0.1.0"',
                        'display_name = "Extra Dynamic Fixture"',
                        'category = "runtime"',
                        'description = "Extra dynamic fixture plugin."',
                        'supported_targets = ["client_runtime", "editor_host"]',
                        'supported_platforms = ["windows", "linux", "macos"]',
                        'capabilities = ["runtime.plugin.extra_dynamic_fixture"]',
                        'default_packaging = ["source_template", "library_embed", "native_dynamic"]',
                        'maturity = "experimental"',
                        "",
                        "[distribution]",
                        'forms = ["dist"]',
                        'default_packaging = ["native_dynamic"]',
                        "abi_version = 3",
                        'engine_compat = ">=0.1, <0.2"',
                        f'dist_crate = "{extra_crate_name}"',
                        'descriptor_symbol = "zircon_native_plugin_descriptor_v3"',
                        'runtime_entry = "zircon_extra_dynamic_fixture_runtime_entry_v3"',
                        "",
                        "[[modules]]",
                        'name = "extra_dynamic_fixture.runtime"',
                        'kind = "runtime"',
                        f'crate_name = "{extra_crate_name}"',
                        'target_modes = ["client_runtime", "editor_host"]',
                        'capabilities = ["runtime.plugin.extra_dynamic_fixture"]',
                        "",
                        "[[asset_importers]]",
                        'id = "shared.importer"',
                        'plugin_id = "extra_dynamic_fixture"',
                        "priority = 100",
                        'source_extensions = ["toml"]',
                        'output_kind = "Data"',
                        "importer_version = 1",
                        'required_capabilities = ["runtime.plugin.extra_dynamic_fixture"]',
                    ]
                ),
                encoding="utf-8",
            )
            (extra_crate_root / "Cargo.toml").write_text(
                "\n".join(
                    [
                        "[package]",
                        f'name = "{extra_crate_name}"',
                        'version = "0.1.0"',
                        'edition = "2021"',
                        "",
                        "[lib]",
                        'crate-type = ["cdylib"]',
                        "",
                        "[features]",
                        'default = ["dist"]',
                        "dist = []",
                        "",
                        "[dependencies]",
                        'zircon_plugin_sdk = { workspace = true, default-features = false, features = ["native"] }',
                    ]
                ),
                encoding="utf-8",
            )

            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                exit_code = main(
                    [
                        "plugin",
                        "validate",
                        "--all",
                        "--repo-root",
                        str(repo_root),
                        "--json",
                    ]
                )

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["target_count"], 2)
            self.assertEqual(report["failed_count"], 0)
            self.assertIn(
                "plugin validate asset_importers id shared.importer is duplicated "
                "by plugin extra_dynamic_fixture asset_importers[0].id and "
                "plugin native_dynamic_fixture asset_importers[0].id",
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
