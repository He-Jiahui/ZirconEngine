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


def _write_default_packaging_fixture(repo_root: Path) -> Path:
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


def _remove_manifest_line_if_present(manifest_path: Path, line: str) -> None:
    text = manifest_path.read_text(encoding="utf-8")
    manifest_path.write_text(text.replace(line + "\n", ""), encoding="utf-8")


class PluginValidateDefaultPackagingTests(unittest.TestCase):
    def test_plugin_validate_rejects_missing_root_default_packaging(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_default_packaging_fixture(repo_root)
            _remove_manifest_line_if_present(
                plugin_manifest_path,
                'default_packaging = ["source_template", "library_embed", "native_dynamic"]',
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin native_dynamic_fixture default_packaging is required",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_malformed_root_default_packaging(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_default_packaging_fixture(repo_root)
            _remove_manifest_line_if_present(
                plugin_manifest_path,
                'default_packaging = ["source_template", "library_embed", "native_dynamic"]',
            )
            _insert_before_distribution(
                plugin_manifest_path,
                [
                    'default_packaging = ["native_dynamic", "zip", "native_dynamic", ""]'
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                'plugin native_dynamic_fixture default_packaging[1] "zip" '
                "is unsupported; expected one of source_template, library_embed, native_dynamic",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture default_packaging[2] native_dynamic "
                "duplicates default_packaging[0]",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture default_packaging[3] "
                "must be a non-empty trimmed string",
                diagnostics,
            )

    def test_plugin_validate_rejects_malformed_optional_feature_default_packaging(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_default_packaging_fixture(repo_root)
            _insert_before_distribution(
                plugin_manifest_path,
                [
                    "[[optional_features]]",
                    'id = "native_dynamic_fixture.preview"',
                    'display_name = "Preview"',
                    'owner_plugin_id = "native_dynamic_fixture"',
                    'capabilities = ["runtime.feature.native_dynamic_fixture.preview"]',
                    'default_packaging = ["source_template", "zip", "source_template"]',
                    "enabled_by_default = false",
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
            self.assertIn(
                'plugin native_dynamic_fixture optional_features[0].default_packaging[1] "zip" '
                "is unsupported; expected one of source_template, library_embed, native_dynamic",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture optional_features[0].default_packaging[2] "
                "source_template duplicates default_packaging[0]",
                diagnostics,
            )

    def test_plugin_validate_rejects_malformed_feature_extension_default_packaging(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_default_packaging_fixture(repo_root)
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
                    'default_packaging = ["library_embed", "zip", "library_embed"]',
                    "enabled_by_default = false",
                    "",
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                'plugin native_dynamic_fixture feature_extensions[0].default_packaging[1] "zip" '
                "is unsupported; expected one of source_template, library_embed, native_dynamic",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture feature_extensions[0].default_packaging[2] "
                "library_embed duplicates default_packaging[0]",
                diagnostics,
            )


if __name__ == "__main__":
    unittest.main()
