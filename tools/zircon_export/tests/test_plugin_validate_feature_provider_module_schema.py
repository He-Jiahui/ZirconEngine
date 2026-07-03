from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.plugin_validate_feature_provider import (
    validate_plugin_feature_provider_package_projection,
)
from tools.zircon_export.tests.plugin_validate_support import (
    _write_complete_sound_manifest,
)


def _validate_sound_feature_provider(package_manifest_text: str) -> list[str]:
    with tempfile.TemporaryDirectory() as temp_dir:
        repo_root = Path(temp_dir) / "repo"
        (repo_root / "zircon_plugins" / "sound").mkdir(parents=True)
        _write_complete_sound_manifest(repo_root)
        diagnostics: list[str] = []

        validate_plugin_feature_provider_package_projection(
            plugin_manifest_path=(
                repo_root / "zircon_plugins" / "sound" / "plugin.toml"
            ),
            package_manifest_text=package_manifest_text,
            requested_plugin_id="sound_timeline_animation_track",
            package_id="sound_timeline_animation_track",
            diagnostics=diagnostics,
        )
        return diagnostics


class PluginValidateFeatureProviderModuleSchemaTests(unittest.TestCase):
    def test_plugin_validate_rejects_generated_feature_provider_module_projection_drift(
        self,
    ) -> None:
        diagnostics = _validate_sound_feature_provider(
            "\n".join(
                [
                    'id = "sound_timeline_animation_track"',
                    'package_kind = "feature_extension"',
                    'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                    'default_packaging = ["native_dynamic"]',
                    "",
                    "[distribution]",
                    'forms = ["dist"]',
                    'default_packaging = ["native_dynamic"]',
                    "abi_version = 3",
                    'engine_compat = ">=0.1, <0.2"',
                    'dist_crate = "zircon_plugin_sound_timeline_animation_dist"',
                    'descriptor_symbol = "zircon_native_plugin_descriptor_v3"',
                    'runtime_entry = "zircon_plugin_sound_timeline_animation_runtime_entry_v3"',
                    "",
                    "[[feature_extensions]]",
                    'id = "sound.timeline_animation_track"',
                    'owner_plugin_id = "sound"',
                    'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                    'default_packaging = ["native_dynamic"]',
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
                    "",
                    "[[feature_extensions.modules]]",
                    'name = "sound.timeline_animation_track.runtime"',
                    'kind = "runtime"',
                    'crate_name = "zircon_plugin_sound_wrong_dist"',
                    'target_modes = ["client_runtime", "editor_host"]',
                    'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                ]
            )
        )

        self.assertIn(
            "plugin sound_timeline_animation_track generated "
            "feature_extensions[0].modules[0].crate_name must equal "
            "generated distribution.dist_crate",
            diagnostics,
        )

    def test_plugin_validate_rejects_generated_feature_provider_module_schema_drift(
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
                        'version = "0.1.0"',
                        'package_kind = "feature_extension"',
                        'display_name = "Sound Timeline Animation Track Provider"',
                        'description = "Native dynamic provider for optional feature sound.timeline_animation_track."',
                        'sdk_api_version = "0.1.0"',
                        'category = "runtime"',
                        'maturity = "experimental"',
                        'supported_targets = ["client_runtime", "editor_host"]',
                        'supported_platforms = ["windows", "linux", "macos"]',
                        'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                        'default_packaging = ["native_dynamic"]',
                        "",
                        "[distribution]",
                        'forms = ["dist"]',
                        'default_packaging = ["native_dynamic"]',
                        "abi_version = 3",
                        'engine_compat = ">=0.1, <0.2"',
                        'dist_crate = "Bad_Crate"',
                        'descriptor_symbol = "zircon_native_plugin_descriptor_v3"',
                        'runtime_entry = "zircon_plugin_sound_timeline_animation_runtime_entry_v3"',
                        "",
                        "[[feature_extensions]]",
                        'id = "sound.timeline_animation_track"',
                        'display_name = "Sound Timeline Animation Track"',
                        'owner_plugin_id = "sound"',
                        'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                        'default_packaging = ["native_dynamic"]',
                        "enabled_by_default = false",
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
                        "",
                        "[[feature_extensions.modules]]",
                        'name = "Sound.Timeline.Runtime"',
                        'kind = "tooling"',
                        'crate_name = "Bad_Crate"',
                        'target_modes = ["nightly_runtime"]',
                        'capabilities = ["editor.extension.sound.timeline"]',
                    ]
                ),
                requested_plugin_id="sound_timeline_animation_track",
                package_id="sound_timeline_animation_track",
                diagnostics=diagnostics,
            )

            self.assertIn(
                "plugin sound_timeline_animation_track generated "
                "feature_extensions[0].modules[0].name Sound.Timeline.Runtime "
                "should contain only lowercase ASCII letters, digits, "
                "underscores, and dots",
                diagnostics,
            )
            self.assertIn(
                "plugin sound_timeline_animation_track generated "
                "feature_extensions[0].modules[0].kind tooling should be one of "
                "runtime, editor, native, vm",
                diagnostics,
            )
            self.assertIn(
                "plugin sound_timeline_animation_track generated "
                "feature_extensions[0].modules[0].crate_name Bad_Crate "
                "should use the zircon_plugin_ prefix",
                diagnostics,
            )
            self.assertIn(
                "plugin sound_timeline_animation_track generated "
                "feature_extensions[0].modules[0].target_modes[0] "
                '"nightly_runtime" is unsupported; expected one of '
                "client_runtime, server_runtime, editor_host",
                diagnostics,
            )


if __name__ == "__main__":
    unittest.main()
