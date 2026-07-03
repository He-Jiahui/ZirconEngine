from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.cli import main
from tools.zircon_export.plugin_validate_feature_provider import (
    validate_plugin_feature_provider_package_projection,
)
from tools.zircon_export.tests.plugin_validate_support import (
    _replace_manifest_line,
    _write_complete_native_dynamic_fixture_manifest,
    _write_complete_sound_manifest,
    _write_plugin_workspace_members,
    _write_sound_feature_dist_crate,
)
from tools.zircon_export.tests.test_plugin_build import (
    _write_dist_plugin_workspace,
    _write_feature_provider_workspace,
)


class PluginValidateFeatureProviderTests(unittest.TestCase):
    def test_plugin_validate_accepts_feature_provider_dist_package(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_sound_timeline_animation_dist"
            _write_feature_provider_workspace(repo_root, crate_name)
            _write_complete_sound_manifest(repo_root)

            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                exit_code = main(
                    [
                        "plugin",
                        "validate",
                        "sound_timeline_animation_track",
                        "--repo-root",
                        str(repo_root),
                        "--json",
                    ]
                )

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"])
            self.assertEqual(
                report["requested_plugin_id"],
                "sound_timeline_animation_track",
            )
            self.assertEqual(report["package_id"], "sound_timeline_animation_track")
            self.assertEqual(report["source_kind"], "feature_extension")
            self.assertEqual(report["dist_crate"], crate_name)
            self.assertEqual(report["diagnostics"], [])

    def test_plugin_validate_reports_feature_provider_owner_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_sound_timeline_animation_dist"
            _write_feature_provider_workspace(repo_root, crate_name)
            _write_complete_sound_manifest(repo_root)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "sound" / "plugin.toml",
                'owner_plugin_id = "sound"',
                'owner_plugin_id = "wrong_sound"',
            )

            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                exit_code = main(
                    [
                        "plugin",
                        "validate",
                        "sound_timeline_animation_track",
                        "--repo-root",
                        str(repo_root),
                        "--json",
                    ]
                )

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin sound_timeline_animation_track generated "
                "feature_extensions[0].owner_plugin_id must equal sound",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_feature_provider_dependency_malformed(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_sound_timeline_animation_dist"
            _write_feature_provider_workspace(repo_root, crate_name)
            _write_complete_sound_manifest(repo_root)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "sound" / "plugin.toml",
                'capability = "runtime.feature.animation.timeline_event_track"',
                'capability = ""',
            )

            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                exit_code = main(
                    [
                        "plugin",
                        "validate",
                        "sound_timeline_animation_track",
                        "--repo-root",
                        str(repo_root),
                        "--json",
                    ]
                )

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin sound_timeline_animation_track optional feature "
                "dependencies[1].capability must be a non-empty trimmed string",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_feature_provider_capability_malformed(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_sound_timeline_animation_dist"
            _write_feature_provider_workspace(repo_root, crate_name)
            _write_complete_sound_manifest(repo_root)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "sound" / "plugin.toml",
                'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                'capabilities = [""]',
            )

            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                exit_code = main(
                    [
                        "plugin",
                        "validate",
                        "sound_timeline_animation_track",
                        "--repo-root",
                        str(repo_root),
                        "--json",
                    ]
                )

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin sound_timeline_animation_track optional feature "
                "capabilities[0] must be a non-empty trimmed string",
                report["diagnostics"],
            )

    def test_plugin_validate_reports_feature_provider_distribution_projection_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            (repo_root / "zircon_plugins" / "sound").mkdir(parents=True)
            _write_complete_sound_manifest(repo_root)
            diagnostics: list[str] = []

            validate_plugin_feature_provider_package_projection(
                plugin_manifest_path=(
                    repo_root / "zircon_plugins" / "sound" / "plugin.toml"
                ),
                package_manifest_text="\n".join(
                    [
                        'id = "sound_timeline_animation_track"',
                        'package_kind = "feature_extension"',
                        "",
                        "[distribution]",
                        'forms = ["dist"]',
                        'default_packaging = ["native_dynamic"]',
                        "abi_version = 3",
                        'engine_compat = ">=0.1, <0.2"',
                        'dist_crate = "zircon_plugin_sound_wrong_dist"',
                        'descriptor_symbol = "zircon_native_plugin_descriptor_v3"',
                        'runtime_entry = "zircon_plugin_sound_timeline_animation_runtime_entry_v3"',
                        "",
                        "[[feature_extensions]]",
                        'id = "sound.timeline_animation_track"',
                        'owner_plugin_id = "sound"',
                        'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                        "",
                        "[[feature_extensions.dependencies]]",
                        'plugin_id = "sound"',
                        'capability = "runtime.plugin.sound"',
                        "primary = true",
                        "",
                        "[[feature_extensions.dependencies]]",
                        'plugin_id = "animation"',
                        'capability = "runtime.feature.animation.timeline_event_track"',
                        "primary = false",
                    ]
                ),
                requested_plugin_id="sound_timeline_animation_track",
                package_id="sound_timeline_animation_track",
                diagnostics=diagnostics,
            )

            self.assertIn(
                "plugin sound_timeline_animation_track generated "
                "distribution.dist_crate must equal owner optional feature "
                "distribution.dist_crate",
                diagnostics,
            )

    def test_plugin_validate_all_reports_root_and_feature_provider_packages(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            root_crate_name = "zircon_plugin_native_dynamic_fixture_native"
            feature_crate_name = "zircon_plugin_sound_timeline_animation_dist"
            _write_dist_plugin_workspace(repo_root, root_crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, root_crate_name)
            _write_sound_feature_dist_crate(repo_root, feature_crate_name)
            _write_complete_sound_manifest(repo_root)
            _write_plugin_workspace_members(
                repo_root,
                [
                    "native_dynamic_fixture/native",
                    "sound/features/timeline_animation_track/dist",
                ],
            )

            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                exit_code = main(
                    [
                        "plugin",
                        "validate",
                        "--all",
                        "--repo-root",
                        str(repo_root),
                        "--json",
                    ]
                )

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 0)
            self.assertEqual(report["command"], "plugin validate --all")
            self.assertFalse(report["fatal"])
            self.assertEqual(report["target_count"], 2)
            self.assertEqual(report["failed_count"], 0)
            self.assertEqual(report["diagnostics"], [])
            self.assertEqual(
                [item["package_id"] for item in report["items"]],
                ["native_dynamic_fixture", "sound_timeline_animation_track"],
            )
            self.assertEqual(
                [item["source_kind"] for item in report["items"]],
                ["plugin", "feature_extension"],
            )

    def test_plugin_validate_all_uses_default_feature_provider_package_id(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            root_crate_name = "zircon_plugin_native_dynamic_fixture_native"
            feature_crate_name = "zircon_plugin_sound_timeline_animation_dist"
            _write_dist_plugin_workspace(repo_root, root_crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, root_crate_name)
            _write_sound_feature_dist_crate(repo_root, feature_crate_name)
            _write_complete_sound_manifest(repo_root)
            _replace_manifest_line(
                repo_root / "zircon_plugins" / "sound" / "plugin.toml",
                'provider_package_id = "sound_timeline_animation_track"',
                "",
            )
            _write_plugin_workspace_members(
                repo_root,
                [
                    "native_dynamic_fixture/native",
                    "sound/features/timeline_animation_track/dist",
                ],
            )

            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                exit_code = main(
                    [
                        "plugin",
                        "validate",
                        "--all",
                        "--repo-root",
                        str(repo_root),
                        "--json",
                    ]
                )

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"])
            self.assertEqual(report["target_count"], 2)
            self.assertEqual(report["failed_count"], 0)
            self.assertEqual(report["diagnostics"], [])
            self.assertEqual(
                [item["package_id"] for item in report["items"]],
                ["native_dynamic_fixture", "sound_timeline_animation_track"],
            )

    def test_plugin_validate_all_reports_malformed_feature_distribution(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            root_crate_name = "zircon_plugin_native_dynamic_fixture_native"
            feature_crate_name = "zircon_plugin_sound_timeline_animation_dist"
            _write_dist_plugin_workspace(repo_root, root_crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, root_crate_name)
            _write_sound_feature_dist_crate(repo_root, feature_crate_name)
            _write_complete_sound_manifest(repo_root)
            sound_manifest_path = repo_root / "zircon_plugins" / "sound" / "plugin.toml"
            _replace_manifest_line(
                sound_manifest_path,
                'provider_package_id = "sound_timeline_animation_track"',
                'provider_package_id = "sound_timeline_animation_track"\ndistribution = "dist"',
            )
            _replace_manifest_line(
                sound_manifest_path,
                "[optional_features.distribution]",
                "# [optional_features.distribution]",
            )
            _write_plugin_workspace_members(
                repo_root,
                [
                    "native_dynamic_fixture/native",
                    "sound/features/timeline_animation_track/dist",
                ],
            )

            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                exit_code = main(
                    [
                        "plugin",
                        "validate",
                        "--all",
                        "--repo-root",
                        str(repo_root),
                        "--json",
                    ]
                )

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["target_count"], 1)
            self.assertEqual(report["failed_count"], 0)
            self.assertIn(
                f"{sound_manifest_path} optional_features[0].distribution must be a table",
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
