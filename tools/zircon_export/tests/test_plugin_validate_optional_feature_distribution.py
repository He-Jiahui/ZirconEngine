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


def _write_optional_feature_distribution_fixture(repo_root: Path) -> Path:
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


class PluginValidateOptionalFeatureDistributionTests(unittest.TestCase):
    def test_plugin_validate_rejects_optional_feature_distribution_contract(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_optional_feature_distribution_fixture(
                repo_root
            )
            _insert_before_distribution(
                plugin_manifest_path,
                [
                    "[[optional_features]]",
                    'id = "native_dynamic_fixture.preview"',
                    'display_name = "Preview"',
                    'owner_plugin_id = "native_dynamic_fixture"',
                    'capabilities = ["runtime.feature.native_dynamic_fixture.preview"]',
                    'default_packaging = ["native_dynamic"]',
                    "enabled_by_default = false",
                    "",
                    "[[optional_features.dependencies]]",
                    'plugin_id = "native_dynamic_fixture"',
                    'capability = "runtime.plugin.native_dynamic_fixture"',
                    "primary = true",
                    "",
                    "[optional_features.distribution]",
                    'forms = ["embed", "sidecar"]',
                    'default_packaging = ["library_embed", "zip"]',
                    "abi_version = 2",
                    'engine_compat = ">=9.0, <10.0"',
                    'dist_crate = "zircon_plugin_native_dynamic_fixture_preview_dist"',
                    'descriptor_symbol = "zircon_native_plugin_descriptor_v2"',
                    'runtime_entry = " "',
                    'assets = "assets/**"',
                    "",
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            for diagnostic in (
                "plugin native_dynamic_fixture optional_features[0].distribution.forms "
                "must include dist",
                'plugin native_dynamic_fixture optional_features[0].distribution.forms[1] '
                '"sidecar" is unsupported; expected one of dist, embed',
                "plugin native_dynamic_fixture optional_features[0].distribution.default_packaging "
                "must include native_dynamic",
                "plugin native_dynamic_fixture optional_features[0].distribution.default_packaging[1] "
                '"zip" is unsupported; expected one of source_template, library_embed, native_dynamic',
                "plugin native_dynamic_fixture optional_features[0].distribution.abi_version "
                "must be 3",
                "plugin native_dynamic_fixture optional_features[0].distribution.engine_compat "
                '">=9.0, <10.0" does not include engine 0.1.0',
                "plugin native_dynamic_fixture optional_features[0].distribution.descriptor_symbol "
                "must equal zircon_native_plugin_descriptor_v3",
                "plugin native_dynamic_fixture optional_features[0].distribution.runtime_entry "
                "must be a non-empty trimmed string",
                "plugin native_dynamic_fixture optional_features[0].distribution "
                "must declare runtime_entry or editor_entry",
                "plugin native_dynamic_fixture optional_features[0].distribution.assets "
                "must be an array",
            ):
                self.assertIn(diagnostic, diagnostics)


if __name__ == "__main__":
    unittest.main()
