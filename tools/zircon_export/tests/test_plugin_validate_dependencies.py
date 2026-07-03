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


class PluginValidateDependencyTests(unittest.TestCase):
    def test_plugin_validate_rejects_malformed_dependencies(self) -> None:
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
                        "[[dependencies]]",
                        'id = ""',
                        'required = "yes"',
                        'capability = ""',
                        "",
                        "[[dependencies]]",
                        'id = "renderer"',
                        "required = true",
                        "interfaces = []",
                        "",
                        "[[dependencies]]",
                        'id = "audio"',
                        "required = true",
                        'interfaces = [" runtime.audio "]',
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
                "plugin native_dynamic_fixture dependencies[0].id "
                "must be a non-empty trimmed string",
                "plugin native_dynamic_fixture dependencies[0].required "
                "must be a bool",
                "plugin native_dynamic_fixture dependencies[0].capability "
                "must be a non-empty trimmed string",
                "plugin native_dynamic_fixture dependencies[1].interfaces "
                "must be a non-empty string array",
                "plugin native_dynamic_fixture dependencies[2].interfaces[0] "
                "must be a non-empty trimmed string",
            ):
                self.assertIn(diagnostic, report["diagnostics"])

    def test_plugin_validate_rejects_empty_dependencies_array(
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
                "dependencies = []\n\n[distribution]",
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
                "plugin native_dynamic_fixture dependencies "
                "must not be empty when declared",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_duplicate_dependency_rows(self) -> None:
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
                        "[[dependencies]]",
                        'id = "renderer"',
                        "required = true",
                        'capability = "runtime.capability.render"',
                        "",
                        "[[dependencies]]",
                        'id = "renderer"',
                        "required = false",
                        'capability = "runtime.capability.render"',
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
                "plugin native_dynamic_fixture dependencies[1] "
                "duplicates dependency row 0",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_unknown_dependency_fields(self) -> None:
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
                        "[[dependencies]]",
                        'id = "external_renderer"',
                        "required = true",
                        'capability = "runtime.capability.render"',
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
                "plugin native_dynamic_fixture dependencies[0].sidecar "
                "is not a known dependency field",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_dependency_capability_not_declared_by_package(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            animation_root = repo_root / "zircon_plugins" / "animation"
            animation_root.mkdir(parents=True)
            (animation_root / "plugin.toml").write_text(
                "\n".join(
                    [
                        'id = "animation"',
                        'capabilities = ["runtime.plugin.animation"]',
                    ]
                ),
                encoding="utf-8",
            )
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                "[distribution]",
                "\n".join(
                    [
                        "[[dependencies]]",
                        'id = "animation"',
                        "required = true",
                        'capability = "runtime.plugin.missing"',
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
                "plugin native_dynamic_fixture dependencies[0].capability "
                "runtime.plugin.missing should be declared by the referenced "
                "static plugin package or one of its feature rows",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_external_dependency_capability_namespace(
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
                        "[[dependencies]]",
                        'id = "external_editor"',
                        "required = false",
                        'capability = "editor.extension.timeline_sequence_authoring"',
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
                "plugin native_dynamic_fixture dependencies[0].capability "
                "editor.extension.timeline_sequence_authoring references no static "
                "plugin package and should use a runtime.module.* or "
                "runtime.capability.* host namespace",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_optional_feature_dependency_capability_not_declared_by_package(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            animation_root = repo_root / "zircon_plugins" / "animation"
            animation_root.mkdir(parents=True)
            (animation_root / "plugin.toml").write_text(
                "\n".join(
                    [
                        'id = "animation"',
                        'capabilities = ["runtime.plugin.animation"]',
                    ]
                ),
                encoding="utf-8",
            )
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml",
                "[distribution]",
                "\n".join(
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
                        "[[optional_features.dependencies]]",
                        'plugin_id = "animation"',
                        'capability = "runtime.plugin.missing"',
                        "primary = false",
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
                "plugin native_dynamic_fixture optional_features[0]."
                "dependencies[1].capability runtime.plugin.missing should be "
                "declared by the referenced static plugin package or one of "
                "its feature rows",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_optional_feature_external_dependency_capability_namespace(
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
                        "[[optional_features]]",
                        'id = "native_dynamic_fixture.preview"',
                        'capabilities = ["runtime.feature.native_dynamic_fixture.preview"]',
                        "",
                        "[[optional_features.dependencies]]",
                        'plugin_id = "native_dynamic_fixture"',
                        'capability = "runtime.plugin.native_dynamic_fixture"',
                        "primary = true",
                        "",
                        "[[optional_features.dependencies]]",
                        'plugin_id = "external_editor"',
                        'capability = "editor.extension.timeline_sequence_authoring"',
                        "primary = false",
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
                "plugin native_dynamic_fixture optional_features[0]."
                "dependencies[1].capability editor.extension.timeline_sequence_authoring "
                "references no static plugin package and should use a runtime.module.* "
                "or runtime.capability.* host namespace",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_optional_feature_without_dependencies(
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
                        "[[optional_features]]",
                        'id = "native_dynamic_fixture.preview"',
                        'capabilities = ["runtime.feature.native_dynamic_fixture.preview"]',
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
                "plugin native_dynamic_fixture optional_features[0].dependencies "
                "is required",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_malformed_optional_feature_dependencies(
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
                        "[[optional_features]]",
                        'id = "native_dynamic_fixture.preview"',
                        'capabilities = ["runtime.feature.native_dynamic_fixture.preview"]',
                        "",
                        "[[optional_features.dependencies]]",
                        'plugin_id = ""',
                        'capability = ""',
                        'primary = "yes"',
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
                "plugin native_dynamic_fixture optional_features[0].dependencies[0].plugin_id "
                "must be a non-empty trimmed string",
                "plugin native_dynamic_fixture optional_features[0].dependencies[0].capability "
                "must be a non-empty trimmed string",
                "plugin native_dynamic_fixture optional_features[0].dependencies[0].primary "
                "must be a bool",
            ):
                self.assertIn(diagnostic, report["diagnostics"])

    def test_plugin_validate_rejects_invalid_optional_feature_primary_dependency(
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
                        "[[optional_features]]",
                        'id = "native_dynamic_fixture.preview"',
                        'capabilities = ["runtime.feature.native_dynamic_fixture.preview"]',
                        "",
                        "[[optional_features.dependencies]]",
                        'plugin_id = "other_plugin"',
                        'capability = "runtime.plugin.native_dynamic_fixture"',
                        "primary = true",
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
                "plugin native_dynamic_fixture optional_features[0].dependencies[0] "
                "primary dependency plugin_id must match package id "
                "native_dynamic_fixture",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_duplicate_optional_feature_dependency_rows(
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
                        "[[optional_features]]",
                        'id = "native_dynamic_fixture.preview"',
                        'capabilities = ["runtime.feature.native_dynamic_fixture.preview"]',
                        "",
                        "[[optional_features.dependencies]]",
                        'plugin_id = "native_dynamic_fixture"',
                        'capability = "runtime.plugin.native_dynamic_fixture"',
                        "primary = true",
                        "",
                        "[[optional_features.dependencies]]",
                        'plugin_id = "native_dynamic_fixture"',
                        'capability = "runtime.plugin.native_dynamic_fixture"',
                        "primary = false",
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
                "plugin native_dynamic_fixture optional_features[0].dependencies[1] "
                "duplicates dependency row 0",
                report["diagnostics"],
            )

