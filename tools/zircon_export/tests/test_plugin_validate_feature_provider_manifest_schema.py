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


class PluginValidateFeatureProviderManifestSchemaTests(unittest.TestCase):
    def test_plugin_validate_rejects_generated_feature_provider_manifest_fields(
        self,
    ) -> None:
        diagnostics = _validate_sound_feature_provider(
            "\n".join(
                [
                    'id = "sound_timeline_animation_track"',
                    'package_kind = "feature_extension"',
                    'preview_channel = "nightly"',
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
            )
        )

        self.assertIn(
            "plugin sound_timeline_animation_track generated "
            "manifest.preview_channel is not a known feature provider "
            "manifest field",
            diagnostics,
        )

    def test_plugin_validate_rejects_generated_feature_provider_manifest_metadata_schema(
        self,
    ) -> None:
        diagnostics = _validate_sound_feature_provider(
            "\n".join(
                [
                    'id = "sound_timeline_animation_track"',
                    "version = 3",
                    'package_kind = "feature_extension"',
                    'supported_targets = ["client_runtime", ""]',
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
            )
        )

        self.assertIn(
            "plugin sound_timeline_animation_track generated "
            "manifest.version must be a non-empty trimmed string",
            diagnostics,
        )
        self.assertIn(
            "plugin sound_timeline_animation_track generated "
            "manifest.supported_targets[1] must be a non-empty trimmed string",
            diagnostics,
        )

    def test_plugin_validate_rejects_generated_feature_provider_manifest_metadata_projection_drift(
        self,
    ) -> None:
        diagnostics = _validate_sound_feature_provider(
            "\n".join(
                [
                    'id = "sound_timeline_animation_track"',
                    'package_kind = "feature_extension"',
                    'capabilities = ["runtime.feature.sound.wrong"]',
                    'default_packaging = ["source_template"]',
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
                ]
            )
        )

        self.assertIn(
            "plugin sound_timeline_animation_track generated "
            "manifest.capabilities must match generated "
            "feature_extensions[0].capabilities",
            diagnostics,
        )
        self.assertIn(
            "plugin sound_timeline_animation_track generated "
            "manifest.default_packaging must match generated "
            "distribution.default_packaging",
            diagnostics,
        )


if __name__ == "__main__":
    unittest.main()
