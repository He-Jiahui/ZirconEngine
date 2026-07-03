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


class PluginValidateAssetImporterContractTests(unittest.TestCase):
    def test_plugin_validate_rejects_malformed_source_extensions(
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
                "\n".join(
                    [
                        'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                        "",
                        "[[asset_importers]]",
                        'id = "native_dynamic_fixture.data_json"',
                        'plugin_id = "native_dynamic_fixture"',
                        "priority = 100",
                        'source_extensions = ["json", ".zui", "JSON"]',
                        'output_kind = "Data"',
                        "importer_version = 1",
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                    ]
                ),
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
                "plugin native_dynamic_fixture asset_importers[0].source_extensions[1] "
                "must be a lowercase extension without dots; use full_suffixes "
                "for dotted suffixes",
                report["diagnostics"],
            )
            self.assertIn(
                "plugin native_dynamic_fixture asset_importers[0].source_extensions[2] "
                "must be lowercase",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_duplicate_source_extensions(
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
                "\n".join(
                    [
                        'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                        "",
                        "[[asset_importers]]",
                        'id = "native_dynamic_fixture.data_json"',
                        'plugin_id = "native_dynamic_fixture"',
                        "priority = 100",
                        'source_extensions = ["json", "json"]',
                        'output_kind = "Data"',
                        "importer_version = 1",
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                    ]
                ),
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
                "plugin native_dynamic_fixture asset_importers[0].source_extensions[1] "
                "duplicates entry 0",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_duplicate_full_suffixes(
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
                "\n".join(
                    [
                        'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                        "",
                        "[[asset_importers]]",
                        'id = "native_dynamic_fixture.zui_component"',
                        'plugin_id = "native_dynamic_fixture"',
                        "priority = 10",
                        'full_suffixes = [".zui", ".zui"]',
                        'output_kind = "UiWidget"',
                        "importer_version = 2",
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                    ]
                ),
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
                "plugin native_dynamic_fixture asset_importers[0].full_suffixes[1] "
                "duplicates entry 0",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_duplicate_asset_importer_metadata_arrays(
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
                "\n".join(
                    [
                        'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                        "",
                        "[[asset_importers]]",
                        'id = "native_dynamic_fixture.data_json"',
                        'plugin_id = "native_dynamic_fixture"',
                        "priority = 100",
                        'source_extensions = ["json"]',
                        'output_kind = "Data"',
                        'additional_output_kinds = ["Mesh", "Mesh"]',
                        "importer_version = 1",
                        'required_capabilities = ['
                        '"runtime.plugin.native_dynamic_fixture", '
                        '"runtime.plugin.native_dynamic_fixture"]',
                    ]
                ),
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
                "plugin native_dynamic_fixture asset_importers[0]."
                "additional_output_kinds[1] duplicates entry 0",
                report["diagnostics"],
            )
            self.assertIn(
                "plugin native_dynamic_fixture asset_importers[0]."
                "required_capabilities[1] duplicates entry 0",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_empty_asset_importer_metadata_arrays(
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
                "\n".join(
                    [
                        'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                        "",
                        "[[asset_importers]]",
                        'id = "native_dynamic_fixture.data_json"',
                        'plugin_id = "native_dynamic_fixture"',
                        "priority = 100",
                        'source_extensions = ["json"]',
                        'output_kind = "Data"',
                        "additional_output_kinds = []",
                        "importer_version = 1",
                        "required_capabilities = []",
                    ]
                ),
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
                "plugin native_dynamic_fixture asset_importers[0]."
                "additional_output_kinds must not be empty when declared",
                report["diagnostics"],
            )
            self.assertIn(
                "plugin native_dynamic_fixture asset_importers[0]."
                "required_capabilities must not be empty when declared",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_asset_importer_required_capability_namespace(
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
                "\n".join(
                    [
                        'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                        "",
                        "[[asset_importers]]",
                        'id = "native_dynamic_fixture.data_json"',
                        'plugin_id = "native_dynamic_fixture"',
                        "priority = 100",
                        'source_extensions = ["json"]',
                        'output_kind = "Data"',
                        "importer_version = 1",
                        'required_capabilities = ["badcap", "Runtime.Plugin"]',
                    ]
                ),
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
                "plugin native_dynamic_fixture asset_importers[0]."
                "required_capabilities[0] must use at least two "
                "dot-separated namespace segments",
                report["diagnostics"],
            )
            self.assertIn(
                "plugin native_dynamic_fixture asset_importers[0]."
                "required_capabilities[1] must contain only lowercase ASCII letters, "
                "digits, underscores, and dots",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_asset_importer_undeclared_required_capability(
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
                "\n".join(
                    [
                        'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                        "",
                        "[[asset_importers]]",
                        'id = "native_dynamic_fixture.data_json"',
                        'plugin_id = "native_dynamic_fixture"',
                        "priority = 100",
                        'source_extensions = ["json"]',
                        'output_kind = "Data"',
                        "importer_version = 1",
                        'required_capabilities = [',
                        '  "runtime.plugin.native_dynamic_fixture",',
                        '  "runtime.plugin.missing_feature",',
                        '  "runtime.capability.asset_registry",',
                        '  "runtime.asset.importer.native",',
                        "]",
                    ]
                ),
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
                "plugin native_dynamic_fixture asset_importers[0]."
                "required_capabilities[1] runtime.plugin.missing_feature "
                "should reference a declared static package/feature capability "
                "or an explicitly host-owned capability",
                report["diagnostics"],
            )
            for host_capability in (
                "runtime.capability.asset_registry",
                "runtime.asset.importer.native",
            ):
                self.assertFalse(
                    any(host_capability in diagnostic for diagnostic in report["diagnostics"]),
                    f"{host_capability} should be accepted as host-owned",
                )

    def test_plugin_validate_accepts_asset_importer_optional_feature_capability(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                'maturity = "experimental"',
                "\n".join(
                    [
                        'maturity = "experimental"',
                        "",
                        "[[optional_features]]",
                        'id = "native_dynamic_fixture.preview_assets"',
                        'display_name = "Preview Assets"',
                        'owner_plugin_id = "native_dynamic_fixture"',
                        'provider_package_id = "native_dynamic_fixture_preview_assets"',
                        'capabilities = ["runtime.feature.native_dynamic_fixture.preview_assets"]',
                        'default_packaging = ["source_template", "library_embed", "native_dynamic"]',
                        "enabled_by_default = false",
                        "",
                        "[[optional_features.dependencies]]",
                        'plugin_id = "native_dynamic_fixture"',
                        'capability = "runtime.plugin.native_dynamic_fixture"',
                        "primary = true",
                    ]
                ),
            )
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                "\n".join(
                    [
                        'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                        "",
                        "[[asset_importers]]",
                        'id = "native_dynamic_fixture.preview_json"',
                        'plugin_id = "native_dynamic_fixture"',
                        "priority = 100",
                        'source_extensions = ["json"]',
                        'output_kind = "Data"',
                        "importer_version = 1",
                        'required_capabilities = ["runtime.feature.native_dynamic_fixture.preview_assets"]',
                    ]
                ),
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
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"])
            self.assertEqual(report["diagnostics"], [])

    def test_plugin_validate_rejects_malformed_full_suffixes(
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
                "\n".join(
                    [
                        'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                        "",
                        "[[asset_importers]]",
                        'id = "native_dynamic_fixture.zui_component"',
                        'plugin_id = "native_dynamic_fixture"',
                        "priority = 10",
                        'full_suffixes = ["zui", ".ZUI"]',
                        'output_kind = "UiWidget"',
                        "importer_version = 2",
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                    ]
                ),
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
                "plugin native_dynamic_fixture asset_importers[0].full_suffixes[0] "
                "must be a dotted suffix",
                report["diagnostics"],
            )
            self.assertIn(
                "plugin native_dynamic_fixture asset_importers[0].full_suffixes[1] "
                "must be lowercase",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_asset_importer_plugin_id_mismatch(
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
                "\n".join(
                    [
                        'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                        "",
                        "[[asset_importers]]",
                        'id = "native_dynamic_fixture.data_json"',
                        'plugin_id = "other_plugin"',
                        "priority = 100",
                        'source_extensions = ["json"]',
                        'output_kind = "Data"',
                        "importer_version = 1",
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                    ]
                ),
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
                "plugin native_dynamic_fixture asset_importers[0].plugin_id "
                "must match package id native_dynamic_fixture",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_asset_importer_without_source_selector(
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
                "\n".join(
                    [
                        'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                        "",
                        "[[asset_importers]]",
                        'id = "native_dynamic_fixture.data_json"',
                        'plugin_id = "native_dynamic_fixture"',
                        "priority = 100",
                        'output_kind = "Data"',
                        "importer_version = 1",
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                    ]
                ),
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
                "plugin native_dynamic_fixture asset_importers[0] "
                "must declare source_extensions or full_suffixes",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_empty_asset_importer_selector_arrays(
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
                "\n".join(
                    [
                        'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                        "",
                        "[[asset_importers]]",
                        'id = "native_dynamic_fixture.data_json"',
                        'plugin_id = "native_dynamic_fixture"',
                        "priority = 100",
                        "source_extensions = []",
                        'output_kind = "Data"',
                        "importer_version = 1",
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                        "",
                        "[[asset_importers]]",
                        'id = "native_dynamic_fixture.ui_document"',
                        'plugin_id = "native_dynamic_fixture"',
                        "priority = 90",
                        "full_suffixes = []",
                        'output_kind = "UiWidget"',
                        "importer_version = 1",
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                    ]
                ),
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
                "plugin native_dynamic_fixture asset_importers[0].source_extensions "
                "must not be empty when declared",
                report["diagnostics"],
            )
            self.assertIn(
                "plugin native_dynamic_fixture asset_importers[1].full_suffixes "
                "must not be empty when declared",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_empty_asset_importers_array(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                "[distribution]",
                "asset_importers = []\n\n[distribution]",
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
                "plugin native_dynamic_fixture asset_importers "
                "must not be empty when declared",
                report["diagnostics"],
            )
