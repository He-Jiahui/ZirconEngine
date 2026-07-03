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


def _write_dependency_fixture(repo_root: Path) -> Path:
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


class PluginValidateOptionalFeatureDependencyTests(unittest.TestCase):
    def test_plugin_validate_rejects_malformed_feature_extension_dependencies(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_dependency_fixture(repo_root)
            _insert_before_distribution(
                plugin_manifest_path,
                [
                    'package_kind = "feature_extension"',
                    "",
                    "[[feature_extensions]]",
                    'id = "native_dynamic_fixture.preview"',
                    'display_name = "Preview"',
                    'owner_plugin_id = "native_dynamic_fixture"',
                    'capabilities = ["runtime.feature.native_dynamic_fixture.preview"]',
                    'default_packaging = ["source_template"]',
                    "enabled_by_default = false",
                    "",
                    "[[feature_extensions.dependencies]]",
                    'plugin_id = ""',
                    'capability = ""',
                    'primary = "yes"',
                    "",
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            for diagnostic in (
                "plugin native_dynamic_fixture feature_extensions[0].dependencies[0].plugin_id "
                "must be a non-empty trimmed string",
                "plugin native_dynamic_fixture feature_extensions[0].dependencies[0].capability "
                "must be a non-empty trimmed string",
                "plugin native_dynamic_fixture feature_extensions[0].dependencies[0].primary "
                "must be a bool",
            ):
                self.assertIn(diagnostic, diagnostics)

    def test_plugin_validate_rejects_feature_extension_primary_owner_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_dependency_fixture(repo_root)
            _insert_before_distribution(
                plugin_manifest_path,
                [
                    'package_kind = "feature_extension"',
                    "",
                    "[[feature_extensions]]",
                    'id = "native_dynamic_fixture.preview"',
                    'display_name = "Preview"',
                    'owner_plugin_id = "native_dynamic_fixture"',
                    'capabilities = ["runtime.feature.native_dynamic_fixture.preview"]',
                    'default_packaging = ["source_template"]',
                    "enabled_by_default = false",
                    "",
                    "[[feature_extensions.dependencies]]",
                    'plugin_id = "other_plugin"',
                    'capability = "runtime.plugin.native_dynamic_fixture"',
                    "primary = true",
                    "",
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "plugin native_dynamic_fixture feature_extensions[0].dependencies[0] "
                "primary dependency plugin_id must match owner plugin id "
                "native_dynamic_fixture",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_unknown_feature_dependency_fields(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_dependency_fixture(repo_root)
            _insert_before_distribution(
                plugin_manifest_path,
                [
                    "[[optional_features]]",
                    'id = "native_dynamic_fixture.preview"',
                    'display_name = "Preview"',
                    'owner_plugin_id = "native_dynamic_fixture"',
                    'capabilities = ["runtime.feature.native_dynamic_fixture.preview"]',
                    'default_packaging = ["source_template"]',
                    "enabled_by_default = false",
                    "",
                    "[[optional_features.dependencies]]",
                    'plugin_id = "native_dynamic_fixture"',
                    'capability = "runtime.plugin.native_dynamic_fixture"',
                    "primary = true",
                    'sidecar = "unexpected"',
                    "",
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture optional_features[0].dependencies[0].sidecar "
                "is not a known optional feature dependency field",
                diagnostics,
            )

        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_dependency_fixture(repo_root)
            _insert_before_distribution(
                plugin_manifest_path,
                [
                    'package_kind = "feature_extension"',
                    "",
                    "[[feature_extensions]]",
                    'id = "native_dynamic_fixture.preview_extension"',
                    'display_name = "Preview Extension"',
                    'owner_plugin_id = "native_dynamic_fixture"',
                    'capabilities = ["runtime.feature.native_dynamic_fixture.preview_extension"]',
                    'default_packaging = ["source_template"]',
                    "enabled_by_default = false",
                    "",
                    "[[feature_extensions.dependencies]]",
                    'plugin_id = "native_dynamic_fixture"',
                    'capability = "runtime.plugin.native_dynamic_fixture"',
                    "primary = true",
                    'sidecar = "unexpected"',
                    "",
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture feature_extensions[0].dependencies[0].sidecar "
                "is not a known optional feature dependency field",
                diagnostics,
            )


if __name__ == "__main__":
    unittest.main()
