from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.cli import main
from tools.zircon_export.plugin_validate_distribution_engine_compat import (
    plugin_validate_engine_compat_matches,
    plugin_validate_parse_engine_version,
)
from tools.zircon_export.tests.plugin_validate_support import (
    _replace_manifest_line,
    _write_complete_native_dynamic_fixture_manifest,
)
from tools.zircon_export.tests.test_plugin_build import _write_dist_plugin_workspace


class PluginValidateDistributionContractTests(unittest.TestCase):
    def test_plugin_validate_reports_missing_dist_form(self) -> None:
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
                        "native_dynamic_fixture",
                        "--repo-root",
                        str(repo_root),
                        "--json",
                    ]
                )

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(
                1,
                report["diagnostics"].count(
                    "plugin native_dynamic_fixture distribution.forms must include dist"
                ),
            )

    def test_plugin_validate_reports_unknown_distribution_form(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                'forms = ["dist"]',
                'forms = ["dist", "sidecar"]',
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
                'plugin native_dynamic_fixture distribution.forms[1] "sidecar" '
                "is unsupported; expected one of dist, embed",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_unknown_distribution_fields(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"\n'
                'preview_channel = "nightly"',
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
                "plugin native_dynamic_fixture distribution.preview_channel "
                "is not a known distribution field",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_unknown_default_packaging(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                'default_packaging = ["native_dynamic"]',
                'default_packaging = ["native_dynamic", "zip"]',
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
                'plugin native_dynamic_fixture distribution.default_packaging[1] "zip" '
                "is unsupported; expected one of source_template, library_embed, native_dynamic",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_duplicate_distribution_packaging_values(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            plugin_manifest_path = (
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml"
            )
            _replace_manifest_line(
                plugin_manifest_path,
                'forms = ["dist"]',
                'forms = ["dist", "dist"]',
            )
            _replace_manifest_line(
                plugin_manifest_path,
                'default_packaging = ["native_dynamic"]',
                'default_packaging = ["native_dynamic", "native_dynamic"]',
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
                "plugin native_dynamic_fixture distribution.forms[1] dist "
                "duplicates distribution.forms[0]",
                report["diagnostics"],
            )
            self.assertIn(
                "plugin native_dynamic_fixture distribution.default_packaging[1] "
                "native_dynamic duplicates distribution.default_packaging[0]",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_malformed_distribution_forms_entry(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                'forms = ["dist"]',
                'forms = ["dist", " padded "]',
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
                "plugin native_dynamic_fixture distribution.forms[1] must be a non-empty trimmed string",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_invalid_engine_compat_version_shape(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                'engine_compat = ">=0.1, <0.2"',
                'engine_compat = ">=0"',
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
                'plugin native_dynamic_fixture distribution.engine_compat ">=0" '
                'is invalid: version "0" must be major.minor[.patch]',
                report["diagnostics"],
            )

    def test_plugin_validate_reports_engine_compat_range_excludes_current_engine(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                'engine_compat = ">=0.1, <0.2"',
                'engine_compat = ">=9.0, <10.0"',
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
                'plugin native_dynamic_fixture distribution.engine_compat ">=9.0, <10.0" '
                "does not include engine 0.1.0",
                report["diagnostics"],
            )

    def test_distribution_engine_compat_leaf_matches_semver_ranges(self) -> None:
        self.assertTrue(
            plugin_validate_engine_compat_matches(">=0.1, <0.2", "0.1.0")
        )
        self.assertTrue(plugin_validate_engine_compat_matches("=1.2.3", "1.2.3"))
        self.assertFalse(plugin_validate_engine_compat_matches(">=0.2", "0.1.9"))
        self.assertEqual(
            (1, 2, 0),
            plugin_validate_parse_engine_version("1.2+build.7"),
        )

    def test_plugin_validate_reports_descriptor_symbol_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                'descriptor_symbol = "zircon_native_plugin_descriptor_v3"',
                'descriptor_symbol = "zircon_native_plugin_descriptor_v2"',
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
                "plugin native_dynamic_fixture distribution.descriptor_symbol "
                "must equal zircon_native_plugin_descriptor_v3",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_missing_runtime_and_editor_entry(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"\n',
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
                "plugin native_dynamic_fixture distribution must declare runtime_entry or editor_entry",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_distribution_assets_not_array(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"\n'
                'assets = "assets/**"',
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
                "plugin native_dynamic_fixture distribution.assets must be an array",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_distribution_assets_untrimmed_entry(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"\n'
                'assets = [" assets/**"]',
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
                "plugin native_dynamic_fixture distribution.assets[0] must be trimmed",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_distribution_assets_plugin_relative_glob(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"\n'
                'assets = ["../outside/**"]',
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
                "plugin native_dynamic_fixture distribution.assets[0] must be a plugin-relative glob",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_distribution_assets_empty_glob(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"\n'
                'assets = ["assets/missing.wgsl"]',
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
                "plugin native_dynamic_fixture distribution.assets[0] matched no plugin asset files",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_distribution_assets_with_retired_ui_suffix_patterns(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            plugin_root = repo_root / "zircon_plugins" / "native_dynamic_fixture"
            _replace_manifest_line(
                plugin_root / "plugin.toml",
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"\n'
                'assets = ["editor/ui/missing_component.ui.toml", '
                '"editor/ui/missing_panel.v2.ui.toml"]',
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
                "plugin native_dynamic_fixture distribution.assets[0] targets retired UI asset suffix editor/ui/missing_component.ui.toml; use .zui",
                report["diagnostics"],
            )
            self.assertIn(
                "plugin native_dynamic_fixture distribution.assets[1] targets retired UI asset suffix editor/ui/missing_panel.v2.ui.toml; use .zui",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_distribution_assets_with_retired_ui_suffixes(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            plugin_root = repo_root / "zircon_plugins" / "native_dynamic_fixture"
            ui_root = plugin_root / "editor" / "ui"
            ui_root.mkdir(parents=True, exist_ok=True)
            (ui_root / "retired_component.ui.toml").write_text(
                '[asset]\nkind = "component"\n',
                encoding="utf-8",
            )
            (ui_root / "retired_panel.v2.ui.toml").write_text(
                '[asset]\nkind = "view"\n',
                encoding="utf-8",
            )
            _replace_manifest_line(
                plugin_root / "plugin.toml",
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"\n'
                'assets = ["editor/ui/*"]',
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
                "plugin native_dynamic_fixture distribution.assets[0] matched retired UI asset suffix editor/ui/retired_component.ui.toml; use .zui",
                report["diagnostics"],
            )
            self.assertIn(
                "plugin native_dynamic_fixture distribution.assets[0] matched retired UI asset suffix editor/ui/retired_panel.v2.ui.toml; use .zui",
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
