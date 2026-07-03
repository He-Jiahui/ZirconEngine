from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.cli import main
from tools.zircon_export.tests.plugin_validate_support import (
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


def _write_optional_feature_fixture(repo_root: Path) -> Path:
    crate_name = "zircon_plugin_native_dynamic_fixture_native"
    _write_dist_plugin_workspace(repo_root, crate_name)
    _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
    return repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml"


def _insert_before_distribution(manifest_path: Path, lines: list[str]) -> None:
    text = manifest_path.read_text(encoding="utf-8")
    marker = "\n[distribution]"
    if marker not in text:
        raise AssertionError("manifest fixture is missing [distribution]")
    manifest_path.write_text(
        text.replace(marker, "\n" + "\n".join(lines) + "\n" + marker, 1),
        encoding="utf-8",
    )


class PluginValidateOptionalFeatureTests(unittest.TestCase):
    def test_plugin_validate_rejects_malformed_optional_feature_schema(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_optional_feature_fixture(repo_root)
            _insert_before_distribution(
                plugin_manifest_path,
                [
                    "[[optional_features]]",
                    'id = "Native.Dynamic"',
                    'display_name = " Preview "',
                    'owner_plugin_id = "other_plugin"',
                    'capabilities = ["badcap", "runtime..feature", "runtime.feature.native_dynamic_fixture.preview", "runtime.feature.native_dynamic_fixture.preview"]',
                    'default_packaging = ["native_dynamic"]',
                    'enabled_by_default = "yes"',
                    'unknown_feature_field = "drift"',
                    "",
                    "[[optional_features.dependencies]]",
                    'plugin_id = "native_dynamic_fixture"',
                    'capability = "runtime.plugin.native_dynamic_fixture"',
                    "primary = true",
                    "",
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            for diagnostic in (
                "plugin native_dynamic_fixture optional_features[0].unknown_feature_field "
                "is not a known optional feature field",
                "plugin native_dynamic_fixture optional_features[0].id Native.Dynamic "
                "should contain only lowercase ASCII letters, digits, underscores, and dots",
                "plugin native_dynamic_fixture optional_features[0].id Native.Dynamic "
                "should stay under package namespace native_dynamic_fixture.",
                "plugin native_dynamic_fixture optional_features[0].display_name "
                "must be a non-empty trimmed string",
                "plugin native_dynamic_fixture optional_features[0].owner_plugin_id "
                "other_plugin should match package id native_dynamic_fixture",
                "plugin native_dynamic_fixture optional_features[0].capabilities[0] badcap "
                "should use at least two dot-separated namespace segments",
                "plugin native_dynamic_fixture optional_features[0].capabilities[1] "
                "runtime..feature should not contain empty namespace segments",
                "plugin native_dynamic_fixture optional_features[0].capabilities[3] "
                "runtime.feature.native_dynamic_fixture.preview duplicates capabilities "
                "capabilities[2]",
                "plugin native_dynamic_fixture optional_features[0].enabled_by_default "
                "must be a bool",
            ):
                self.assertIn(diagnostic, diagnostics)

    def test_plugin_validate_rejects_non_table_optional_feature_row(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_optional_feature_fixture(repo_root)
            _insert_before_distribution(
                plugin_manifest_path,
                [
                    'optional_features = ["native_dynamic_fixture.preview"]',
                    "",
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "plugin native_dynamic_fixture optional_features[0] must be a table",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_empty_optional_features_array(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_optional_feature_fixture(repo_root)
            _insert_before_distribution(
                plugin_manifest_path,
                [
                    "optional_features = []",
                    "",
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "plugin native_dynamic_fixture optional_features "
                "must not be empty when declared",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_optional_feature_provider_package_id_schema(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_optional_feature_fixture(repo_root)
            _insert_before_distribution(
                plugin_manifest_path,
                [
                    "[[optional_features]]",
                    'id = "native_dynamic_fixture.preview"',
                    'display_name = "Preview"',
                    'owner_plugin_id = "native_dynamic_fixture"',
                    'provider_package_id = " native_dynamic_preview "',
                    'capabilities = ["runtime.feature.native_dynamic_fixture.preview"]',
                    'default_packaging = ["native_dynamic"]',
                    'enabled_by_default = false',
                    "",
                    "[[optional_features.dependencies]]",
                    'plugin_id = "native_dynamic_fixture"',
                    'capability = "runtime.plugin.native_dynamic_fixture"',
                    "primary = true",
                    "",
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "plugin native_dynamic_fixture optional_features[0].provider_package_id "
                "must be a non-empty trimmed string",
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
