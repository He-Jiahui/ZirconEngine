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


def _validate_native_dynamic_fixture(repo_root: Path) -> tuple[int, dict[str, object]]:
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


class PluginValidateAssetImporterTests(unittest.TestCase):
    def test_plugin_validate_accepts_asset_importer_zui_suffix(self) -> None:
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
                        'full_suffixes = [".zui"]',
                        'output_kind = "UiWidget"',
                        "importer_version = 2",
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                    ]
                ),
            )

            exit_code, report = _validate_native_dynamic_fixture(repo_root)
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"])
            self.assertEqual(report["diagnostics"], [])

    def test_plugin_validate_rejects_asset_importers_with_retired_ui_suffixes(
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
                        'id = "native_dynamic_fixture.legacy_ui"',
                        'plugin_id = "native_dynamic_fixture"',
                        "priority = 10",
                        'full_suffixes = [".ui.toml", ".v2.ui.toml"]',
                        'output_kind = "UiWidget"',
                        "importer_version = 1",
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                    ]
                ),
            )

            exit_code, report = _validate_native_dynamic_fixture(repo_root)
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin native_dynamic_fixture asset_importers[0].full_suffixes[0] "
                "declares retired UI asset suffix .ui.toml; use .zui",
                report["diagnostics"],
            )
            self.assertIn(
                "plugin native_dynamic_fixture asset_importers[0].full_suffixes[1] "
                "declares retired UI asset suffix .v2.ui.toml; use .zui",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_malformed_asset_importer_schema(self) -> None:
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
                        'id = " native_dynamic_fixture.bad_importer "',
                        "plugin_id = 42",
                        'priority = "high"',
                        'full_suffixes = [".zui"]',
                        'output_kind = ""',
                        "importer_version = 0",
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture", ""]',
                    ]
                ),
            )

            exit_code, report = _validate_native_dynamic_fixture(repo_root)
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            for diagnostic in (
                "plugin native_dynamic_fixture asset_importers[0].id must be trimmed",
                "plugin native_dynamic_fixture asset_importers[0].plugin_id "
                "must be a non-empty string",
                "plugin native_dynamic_fixture asset_importers[0].priority "
                "must be an integer",
                "plugin native_dynamic_fixture asset_importers[0].output_kind "
                "must be a non-empty string",
                "plugin native_dynamic_fixture asset_importers[0].importer_version "
                "must be a positive integer",
                "plugin native_dynamic_fixture asset_importers[0].required_capabilities[1] "
                "must be a non-empty string",
            ):
                self.assertIn(diagnostic, report["diagnostics"])

    def test_plugin_validate_rejects_unknown_asset_importer_fields(self) -> None:
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
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                        'sidecar = "drift"',
                    ]
                ),
            )

            exit_code, report = _validate_native_dynamic_fixture(repo_root)
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin native_dynamic_fixture asset_importers[0].sidecar "
                "is not a known asset_importer field",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_malformed_asset_importer_ids(self) -> None:
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
                        'id = "badid"',
                        'plugin_id = "native_dynamic_fixture"',
                        "priority = 100",
                        'source_extensions = ["toml"]',
                        'output_kind = "Data"',
                        "importer_version = 1",
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                        "",
                        "[[asset_importers]]",
                        'id = "native_dynamic_fixture..json"',
                        'plugin_id = "native_dynamic_fixture"',
                        "priority = 100",
                        'source_extensions = ["json"]',
                        'output_kind = "Data"',
                        "importer_version = 1",
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                        "",
                        "[[asset_importers]]",
                        'id = "native_dynamic_fixture.Bad-Importer"',
                        'plugin_id = "native_dynamic_fixture"',
                        "priority = 100",
                        'source_extensions = ["yaml"]',
                        'output_kind = "Data"',
                        "importer_version = 1",
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                    ]
                ),
            )

            exit_code, report = _validate_native_dynamic_fixture(repo_root)
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            for diagnostic in (
                "plugin native_dynamic_fixture asset_importers[0].id "
                "must use at least two dot-separated namespace segments",
                "plugin native_dynamic_fixture asset_importers[1].id "
                "must not contain empty namespace segments",
                "plugin native_dynamic_fixture asset_importers[2].id must contain "
                "only lowercase ASCII letters, digits, underscores, and dots",
            ):
                self.assertIn(diagnostic, report["diagnostics"])

    def test_plugin_validate_rejects_asset_importer_numeric_range_overflow(
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
                        "priority = 2147483648",
                        'source_extensions = ["json"]',
                        'output_kind = "Data"',
                        "importer_version = 4294967296",
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                        "",
                        "[[asset_importers]]",
                        'id = "native_dynamic_fixture.data_toml"',
                        'plugin_id = "native_dynamic_fixture"',
                        "priority = -2147483649",
                        'source_extensions = ["toml"]',
                        'output_kind = "Data"',
                        "importer_version = 1",
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                    ]
                ),
            )

            exit_code, report = _validate_native_dynamic_fixture(repo_root)
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            for diagnostic in (
                "plugin native_dynamic_fixture asset_importers[0].priority "
                "must fit i32",
                "plugin native_dynamic_fixture asset_importers[0].importer_version "
                "must be a positive u32",
                "plugin native_dynamic_fixture asset_importers[1].priority "
                "must fit i32",
            ):
                self.assertIn(diagnostic, report["diagnostics"])

    def test_plugin_validate_rejects_malformed_asset_importer_string_arrays(
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
                        'source_extensions = ["json", " bad "]',
                        'additional_output_kinds = ["Data", ""]',
                        'output_kind = "Data"',
                        "importer_version = 1",
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                    ]
                ),
            )

            exit_code, report = _validate_native_dynamic_fixture(repo_root)
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin native_dynamic_fixture asset_importers[0].source_extensions[1] "
                "must be trimmed",
                report["diagnostics"],
            )
            self.assertIn(
                "plugin native_dynamic_fixture "
                "asset_importers[0].additional_output_kinds[1] "
                "must be a non-empty string",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_unknown_asset_importer_output_kinds(
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
                        'additional_output_kinds = ["Mesh", "UnknownMesh"]',
                        'output_kind = "UnknownAsset"',
                        "importer_version = 1",
                        'required_capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                    ]
                ),
            )

            exit_code, report = _validate_native_dynamic_fixture(repo_root)
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin native_dynamic_fixture asset_importers[0].output_kind "
                "must be a known ResourceKind",
                report["diagnostics"],
            )
            self.assertIn(
                "plugin native_dynamic_fixture "
                "asset_importers[0].additional_output_kinds[1] "
                "must be a known ResourceKind",
                report["diagnostics"],
            )
