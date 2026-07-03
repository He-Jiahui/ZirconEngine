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


def _append_manifest(manifest_path: Path, lines: list[str]) -> None:
    manifest_path.write_text(
        manifest_path.read_text(encoding="utf-8")
        + "\n"
        + "\n".join(lines)
        + "\n",
        encoding="utf-8",
    )


class PluginValidateComponentTests(unittest.TestCase):
    def test_plugin_validate_rejects_malformed_component_row(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[components]]",
                    'type_id = "Native.Dynamic"',
                    'plugin_id = "other_plugin"',
                    'display_name = " Fixture Component "',
                    "",
                    "[[components.properties]]",
                    'name = " speed "',
                    'value_type = ""',
                    'editable = "yes"',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture components[0].type_id "
                "Native.Dynamic should contain only lowercase ASCII letters, "
                "digits, underscores, and dots",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture components[0].type_id "
                "Native.Dynamic should stay under package namespace "
                "native_dynamic_fixture.",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture components[0].plugin_id "
                "other_plugin should match package id native_dynamic_fixture",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture components[0].display_name "
                "must be a non-empty trimmed string",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture components[0].properties[0].name "
                "must be a non-empty trimmed string",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture components[0].properties[0].value_type "
                "must be a non-empty trimmed string",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture components[0].properties[0].editable "
                "must be a bool",
                diagnostics,
            )

    def test_plugin_validate_rejects_duplicate_component_type_id(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            current_component = [
                "[[components]]",
                'type_id = "native_dynamic_fixture.transform"',
                'plugin_id = "native_dynamic_fixture"',
                'display_name = "Transform"',
            ]
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                current_component,
            )
            other_manifest = repo_root / "zircon_plugins/other/plugin.toml"
            other_manifest.parent.mkdir(parents=True, exist_ok=True)
            other_manifest.write_text(
                "\n".join(
                    [
                        'id = "other"',
                        'capabilities = ["runtime.plugin.other"]',
                        *current_component,
                    ]
                ),
                encoding="utf-8",
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "plugin native_dynamic_fixture components[0].type_id "
                "native_dynamic_fixture.transform duplicates component type_id "
                "declared by other",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_ui_component_retired_document_path(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[ui_components]]",
                    'component_id = "Native.Dynamic.Panel"',
                    'plugin_id = "other_plugin"',
                    'ui_document = "../panel.ui.toml"',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture ui_components[0].component_id "
                "Native.Dynamic.Panel should contain only lowercase ASCII letters, "
                "digits, underscores, and dots",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture ui_components[0].plugin_id "
                "other_plugin should match package id native_dynamic_fixture",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture ui_components[0].ui_document "
                "../panel.ui.toml should reference a .zui component asset",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture ui_components[0].ui_document "
                "../panel.ui.toml should be a relative forward-slash package path",
                diagnostics,
            )

    def test_plugin_validate_rejects_unknown_component_fields(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[components]]",
                    'type_id = "native_dynamic_fixture.transform"',
                    'plugin_id = "native_dynamic_fixture"',
                    'display_name = "Transform"',
                    'sidecar = "unexpected"',
                    "",
                    "[[components.properties]]",
                    'name = "speed"',
                    'value_type = "f32"',
                    "editable = true",
                    'sidecar = "unexpected"',
                    "",
                    "[[ui_components]]",
                    'component_id = "native_dynamic_fixture.panel"',
                    'plugin_id = "native_dynamic_fixture"',
                    'ui_document = "ui/panel.zui"',
                    'sidecar = "unexpected"',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture components[0].sidecar "
                "is not a known component field",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture components[0].properties[0].sidecar "
                "is not a known component property field",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture ui_components[0].sidecar "
                "is not a known ui_component field",
                diagnostics,
            )


if __name__ == "__main__":
    unittest.main()
