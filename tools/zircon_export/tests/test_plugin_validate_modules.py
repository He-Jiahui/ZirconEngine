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


def _insert_before_distribution(manifest_path: Path, lines: list[str]) -> None:
    text = manifest_path.read_text(encoding="utf-8")
    marker = "\n[distribution]"
    if marker not in text:
        raise AssertionError("manifest fixture is missing [distribution]")
    manifest_path.write_text(
        text.replace(marker, "\n" + "\n".join(lines) + "\n" + marker, 1),
        encoding="utf-8",
    )


class PluginValidateModuleTests(unittest.TestCase):
    def test_plugin_validate_rejects_malformed_module_row(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[modules]]",
                    'name = "Native.Dynamic"',
                    'kind = "unknown"',
                    'crate_name = "bad__crate_"',
                    'target_modes = ["mobile_runtime"]',
                    'capabilities = ["editor.extension.bad"]',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture modules[1].name Native.Dynamic "
                "should contain only lowercase ASCII letters, digits, underscores, and dots",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture modules[1].name Native.Dynamic "
                "should stay under namespace native_dynamic_fixture.",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture modules[1].kind unknown "
                "should be one of runtime, editor, native, vm",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture modules[1].crate_name bad__crate_ "
                "should use the zircon_plugin_ prefix",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture modules[1].crate_name bad__crate_ "
                "should not end with an underscore or contain repeated underscores",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture modules[1].target_modes[0] "
                "mobile_runtime should be covered by package supported_targets",
                diagnostics,
            )

    def test_plugin_validate_rejects_duplicate_module_name(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[modules]]",
                    'name = "native_dynamic_fixture.runtime"',
                    'kind = "runtime"',
                    'crate_name = "zircon_plugin_native_dynamic_fixture_native"',
                    'target_modes = ["client_runtime"]',
                    'capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "plugin native_dynamic_fixture modules[1].name "
                "native_dynamic_fixture.runtime duplicates module name row 0",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_optional_feature_module_namespace(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[optional_features]]",
                    'id = "native_dynamic_fixture.preview"',
                    'capabilities = ["runtime.feature.native_dynamic_fixture.preview"]',
                    "",
                    "[[optional_features.dependencies]]",
                    'plugin_id = "native_dynamic_fixture"',
                    'capability = "runtime.plugin.native_dynamic_fixture"',
                    "primary = true",
                    "",
                    "[[optional_features.modules]]",
                    'name = "native_dynamic_fixture.runtime"',
                    'kind = "editor"',
                    'crate_name = "zircon_plugin_native_dynamic_fixture_native"',
                    'target_modes = ["client_runtime"]',
                    'capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture optional_features[0].modules[0].name "
                "native_dynamic_fixture.runtime should stay under namespace "
                "native_dynamic_fixture.preview.",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture optional_features[0].modules[0].name "
                "native_dynamic_fixture.runtime with kind editor should end with .editor",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture optional_features[0].modules[0] "
                "is an editor module and should only target editor_host, got client_runtime",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture optional_features[0].modules[0].capabilities[0] "
                "runtime.plugin.native_dynamic_fixture should start with editor.",
                diagnostics,
            )

    def test_plugin_validate_rejects_feature_extension_module_namespace(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _insert_before_distribution(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
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
                    "[[feature_extensions.modules]]",
                    'name = "native_dynamic_fixture.runtime"',
                    'kind = "runtime"',
                    'crate_name = "zircon_plugin_native_dynamic_fixture_native"',
                    'target_modes = ["client_runtime"]',
                    'capabilities = ["runtime.feature.native_dynamic_fixture.preview"]',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "plugin native_dynamic_fixture feature_extensions[0].modules[0].name "
                "native_dynamic_fixture.runtime should stay under namespace "
                "native_dynamic_fixture.preview.",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_module_crate_missing_workspace_member(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[modules]]",
                    'name = "native_dynamic_fixture.extra.runtime"',
                    'kind = "editor"',
                    'crate_name = "zircon_plugin_native_dynamic_fixture_editor"',
                    'target_modes = ["editor_host"]',
                    'capabilities = ["editor.extension.native_dynamic_fixture"]',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "plugin native_dynamic_fixture modules[1].crate_name "
                "zircon_plugin_native_dynamic_fixture_editor must be a "
                "zircon_plugins workspace member",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_module_crate_outside_package_root(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            workspace_manifest = repo_root / "zircon_plugins/Cargo.toml"
            workspace_manifest.write_text(
                workspace_manifest.read_text(encoding="utf-8").replace(
                    'members = ["native_dynamic_fixture/native"]',
                    'members = ["native_dynamic_fixture/native", "other_plugin/runtime"]',
                ),
                encoding="utf-8",
            )
            other_crate_root = repo_root / "zircon_plugins/other_plugin/runtime"
            other_crate_root.mkdir(parents=True)
            (other_crate_root / "Cargo.toml").write_text(
                "\n".join(
                    [
                        "[package]",
                        'name = "zircon_plugin_other_runtime"',
                        'version = "0.1.0"',
                        'edition = "2021"',
                    ]
                ),
                encoding="utf-8",
            )
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[modules]]",
                    'name = "native_dynamic_fixture.extra.runtime"',
                    'kind = "editor"',
                    'crate_name = "zircon_plugin_other_runtime"',
                    'target_modes = ["editor_host"]',
                    'capabilities = ["editor.extension.native_dynamic_fixture"]',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "plugin native_dynamic_fixture modules[1].crate_name "
                "zircon_plugin_other_runtime workspace member other_plugin/runtime "
                "must stay under native_dynamic_fixture",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_malformed_module_system_names(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[modules]]",
                    'name = "native_dynamic_fixture.editor"',
                    'kind = "runtime"',
                    'crate_name = "zircon_plugin_native_dynamic_fixture_native"',
                    'target_modes = ["client_runtime"]',
                    'capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                    'system_sets = ["Native.Dynamic", "other_plugin.systems"]',
                    'system_anchors = ["tick"]',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture modules[1].system_sets[0] "
                "Native.Dynamic should contain only lowercase ASCII letters, "
                "digits, underscores, and dots",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture modules[1].system_sets[1] "
                "other_plugin.systems should stay under namespace "
                "native_dynamic_fixture.",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture modules[1].system_anchors[0] "
                "tick should use package.module dot namespace form",
                diagnostics,
            )

    def test_plugin_validate_rejects_duplicate_module_system_names(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[modules]]",
                    'name = "native_dynamic_fixture.editor"',
                    'kind = "runtime"',
                    'crate_name = "zircon_plugin_native_dynamic_fixture_native"',
                    'target_modes = ["client_runtime"]',
                    'capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                    'system_sets = ["native_dynamic_fixture.systems", "native_dynamic_fixture.systems"]',
                    'system_anchors = ["native_dynamic_fixture.tick", "native_dynamic_fixture.tick"]',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture modules[1].system_sets[1] "
                "native_dynamic_fixture.systems duplicates system_sets[0]",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture modules[1].system_anchors[1] "
                "native_dynamic_fixture.tick duplicates system_anchors[0]",
                diagnostics,
            )

    def test_plugin_validate_rejects_non_runtime_module_system_names(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[modules]]",
                    'name = "native_dynamic_fixture.editor"',
                    'kind = "editor"',
                    'crate_name = "zircon_plugin_native_dynamic_fixture_native"',
                    'target_modes = ["editor_host"]',
                    'capabilities = ["editor.extension.native_dynamic_fixture"]',
                    'system_anchors = ["native_dynamic_fixture.editor_tick"]',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "plugin native_dynamic_fixture modules[1].system_anchors "
                "may only be declared by runtime modules",
                report["diagnostics"],
            )

    def test_plugin_validate_accepts_module_description_descriptor_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[modules]]",
                    'name = "native_dynamic_fixture.tools.runtime"',
                    'description = "Runtime plugin module native_dynamic_fixture.tools.runtime"',
                    'kind = "runtime"',
                    'crate_name = "zircon_plugin_native_dynamic_fixture_native"',
                    'target_modes = ["client_runtime"]',
                    'capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"])
            self.assertNotIn(
                "plugin native_dynamic_fixture modules[1].description "
                "is not a known module field",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_malformed_module_description(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[modules]]",
                    'name = "native_dynamic_fixture.tools.runtime"',
                    'description = " Runtime plugin module native_dynamic_fixture.tools.runtime "',
                    'kind = "runtime"',
                    'crate_name = "zircon_plugin_native_dynamic_fixture_native"',
                    'target_modes = ["client_runtime"]',
                    'capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "plugin native_dynamic_fixture modules[1].description "
                "must be a non-empty trimmed string",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_unknown_module_fields(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[modules]]",
                    'name = "native_dynamic_fixture.tools.runtime"',
                    'kind = "runtime"',
                    'crate_name = "zircon_plugin_native_dynamic_fixture_native"',
                    'target_modes = ["client_runtime"]',
                    'capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                    'sidecar = "unexpected"',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture modules[1].sidecar "
                "is not a known module field",
                diagnostics,
            )


    def test_plugin_validate_accepts_typed_editor_event_consumer(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            editor_root = repo_root / "zircon_plugins/native_dynamic_fixture/editor"
            editor_root.mkdir(parents=True)
            (editor_root / "Cargo.toml").write_text(
                "\n".join(
                    [
                        "[package]",
                        'name = "zircon_plugin_native_dynamic_fixture_editor"',
                        'version = "0.1.0"',
                        'edition = "2021"',
                    ]
                ),
                encoding="utf-8",
            )
            workspace = repo_root / "zircon_plugins/Cargo.toml"
            workspace.write_text(
                workspace.read_text(encoding="utf-8").replace(
                    'members = ["native_dynamic_fixture/native"]',
                    'members = ["native_dynamic_fixture/native", "native_dynamic_fixture/editor"]',
                ),
                encoding="utf-8",
            )
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[modules]]",
                    'name = "native_dynamic_fixture.editor"',
                    'kind = "editor"',
                    'crate_name = "zircon_plugin_native_dynamic_fixture_editor"',
                    'target_modes = ["editor_host"]',
                    'capabilities = ["editor.extension.native_dynamic_fixture"]',
                    "",
                    "[[modules.event_consumers]]",
                    'consumer_id = "native_dynamic_fixture.editor.meter"',
                    'event_id = "native_dynamic_fixture.events.meter"',
                    'payload_schema = "native_dynamic_fixture.events.meter.v1"',
                    'required_capability = "editor.extension.native_dynamic_fixture"',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(0, exit_code, report["diagnostics"])


if __name__ == "__main__":
    unittest.main()
