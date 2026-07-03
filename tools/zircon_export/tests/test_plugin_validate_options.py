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


class PluginValidateOptionsTests(unittest.TestCase):
    def test_plugin_validate_rejects_option_undeclared_required_capability(
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
                "\n".join(
                    [
                        "[[options]]",
                        'key = "native_dynamic_fixture.missing_gate"',
                        'display_name = "Missing Gate"',
                        'value_type = "bool"',
                        'default_value = "false"',
                        'required_capability = "runtime.plugin.missing_feature"',
                        "",
                        "[[options]]",
                        'key = "native_dynamic_fixture.host_gate"',
                        'display_name = "Host Gate"',
                        'value_type = "bool"',
                        'default_value = "false"',
                        'required_capability = "runtime.capability.asset_registry"',
                        "",
                        "[distribution]",
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
                "plugin native_dynamic_fixture options[0].required_capability "
                "runtime.plugin.missing_feature should reference a declared "
                "static package/feature capability or an explicitly host-owned capability",
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "runtime.capability.asset_registry" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                "runtime.capability.asset_registry should be accepted as host-owned",
            )

    def test_plugin_validate_accepts_option_optional_feature_required_capability(
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
                        'id = "native_dynamic_fixture.preview_options"',
                        'display_name = "Preview Options"',
                        'owner_plugin_id = "native_dynamic_fixture"',
                        'provider_package_id = "native_dynamic_fixture_preview_options"',
                        'capabilities = ["runtime.feature.native_dynamic_fixture.preview_options"]',
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
                "[distribution]",
                "\n".join(
                    [
                        "[[options]]",
                        'key = "native_dynamic_fixture.preview_gate"',
                        'display_name = "Preview Gate"',
                        'value_type = "bool"',
                        'default_value = "false"',
                        'required_capability = "runtime.feature.native_dynamic_fixture.preview_options"',
                        "",
                        "[distribution]",
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

    def test_plugin_validate_rejects_malformed_options_schema(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                "[distribution]",
                "\n".join(
                    [
                        "[[options]]",
                        'key = "badkey"',
                        'display_name = " Missing Gate "',
                        'value_type = "flag"',
                        'default_value = "true"',
                        'enum_values = ["on"]',
                        "",
                        "[[options]]",
                        'key = "native_dynamic_fixture.quality"',
                        'display_name = "Quality"',
                        'value_type = "enum"',
                        'default_value = "high"',
                        'enum_values = ["Low", "medium", "medium"]',
                        "",
                        "[distribution]",
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
            for diagnostic in (
                "plugin native_dynamic_fixture options[0].key must use at least two "
                "dot-separated namespace segments",
                "plugin native_dynamic_fixture options[0].display_name "
                "must be a non-empty trimmed string",
                'plugin native_dynamic_fixture options[0].value_type "flag" '
                "is unsupported; expected one of bool, integer, number, string, enum",
                "plugin native_dynamic_fixture options[0].enum_values "
                "must only be declared for enum options",
                "plugin native_dynamic_fixture options[1].enum_values[0] must contain "
                "only lowercase ASCII letters, digits, underscores, or hyphens",
                "plugin native_dynamic_fixture options[1].enum_values[2] "
                "duplicates entry 1",
                "plugin native_dynamic_fixture options[1].default_value "
                "must be declared in enum_values",
            ):
                self.assertIn(diagnostic, report["diagnostics"])

    def test_plugin_validate_rejects_unknown_option_fields(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                "[distribution]",
                "\n".join(
                    [
                        "[[options]]",
                        'key = "native_dynamic_fixture.preview"',
                        'display_name = "Preview"',
                        'value_type = "bool"',
                        'default_value = "false"',
                        'sidecar = "unexpected"',
                        "",
                        "[distribution]",
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
                "plugin native_dynamic_fixture options[0].sidecar "
                "is not a known option field",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_malformed_option_default_values(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                "[distribution]",
                "\n".join(
                    [
                        "[[options]]",
                        'key = "native_dynamic_fixture.enabled"',
                        'display_name = "Enabled"',
                        'value_type = "bool"',
                        'default_value = "yes"',
                        "",
                        "[[options]]",
                        'key = "native_dynamic_fixture.count"',
                        'display_name = "Count"',
                        'value_type = "integer"',
                        'default_value = "1.5"',
                        "",
                        "[[options]]",
                        'key = "native_dynamic_fixture.scale"',
                        'display_name = "Scale"',
                        'value_type = "number"',
                        'default_value = "nan"',
                        "",
                        "[distribution]",
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
            for diagnostic in (
                "plugin native_dynamic_fixture options[0].default_value "
                "bool value must be true or false",
                "plugin native_dynamic_fixture options[1].default_value "
                "integer value must parse as i64",
                "plugin native_dynamic_fixture options[2].default_value "
                "number value must be finite",
            ):
                self.assertIn(diagnostic, report["diagnostics"])
