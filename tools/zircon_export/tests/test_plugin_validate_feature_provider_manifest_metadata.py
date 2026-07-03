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


class PluginValidateFeatureProviderManifestMetadataTests(unittest.TestCase):
    def test_plugin_validate_rejects_generated_feature_provider_manifest_missing_metadata(
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
                        'dist_crate = "zircon_plugin_sound_timeline_animation_dist"',
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
                        'name = "sound.timeline_animation_track.runtime"',
                        'kind = "runtime"',
                        'crate_name = "zircon_plugin_sound_timeline_animation_dist"',
                        'target_modes = ["client_runtime", "editor_host"]',
                        'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                    ]
                ),
                requested_plugin_id="sound_timeline_animation_track",
                package_id="sound_timeline_animation_track",
                diagnostics=diagnostics,
            )

            for field in (
                "version",
                "display_name",
                "description",
                "sdk_api_version",
                "category",
                "maturity",
                "supported_targets",
                "supported_platforms",
                "capabilities",
                "default_packaging",
            ):
                self.assertIn(
                    "plugin sound_timeline_animation_track generated "
                    f"manifest.{field} is required",
                    diagnostics,
                )

    def test_plugin_validate_rejects_generated_feature_provider_manifest_owner_metadata_drift(
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
                        'version = "9.9.9"',
                        'package_kind = "feature_extension"',
                        'display_name = "Sound Timeline Animation Track Provider"',
                        'description = "Native dynamic provider for optional feature sound.timeline_animation_track."',
                        'sdk_api_version = "9.9.9"',
                        'category = "diagnostics"',
                        'maturity = "stable"',
                        'supported_targets = ["client_runtime", "editor_host"]',
                        'supported_platforms = ["headless"]',
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
                        'name = "sound.timeline_animation_track.runtime"',
                        'kind = "runtime"',
                        'crate_name = "zircon_plugin_sound_timeline_animation_dist"',
                        'target_modes = ["client_runtime", "editor_host"]',
                        'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                    ]
                ),
                requested_plugin_id="sound_timeline_animation_track",
                package_id="sound_timeline_animation_track",
                diagnostics=diagnostics,
            )

            for field in ("version", "sdk_api_version", "category", "maturity"):
                self.assertIn(
                    "plugin sound_timeline_animation_track generated "
                    f"manifest.{field} must equal owner manifest.{field}",
                    diagnostics,
                )
            self.assertIn(
                "plugin sound_timeline_animation_track generated "
                "manifest.supported_platforms must match "
                "owner manifest.supported_platforms",
                diagnostics,
            )

    def test_plugin_validate_rejects_generated_feature_provider_manifest_description_drift(
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
                        'display_name = "Sound Timeline Animation Track Provider"',
                        'description = "Drifted feature provider description."',
                        'supported_targets = ["client_runtime", "editor_host"]',
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
                        'name = "sound.timeline_animation_track.runtime"',
                        'kind = "runtime"',
                        'crate_name = "zircon_plugin_sound_timeline_animation_dist"',
                        'target_modes = ["client_runtime", "editor_host"]',
                        'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                    ]
                ),
                requested_plugin_id="sound_timeline_animation_track",
                package_id="sound_timeline_animation_track",
                diagnostics=diagnostics,
            )

            self.assertIn(
                "plugin sound_timeline_animation_track generated "
                "manifest.description must equal Native dynamic provider for "
                "optional feature feature_extensions[0].id",
                diagnostics,
            )

    def test_plugin_validate_rejects_generated_feature_provider_manifest_supported_targets_drift(
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
                        'display_name = "Sound Timeline Animation Track Provider"',
                        'supported_targets = ["editor_host"]',
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
                        'name = "sound.timeline_animation_track.runtime"',
                        'kind = "runtime"',
                        'crate_name = "zircon_plugin_sound_timeline_animation_dist"',
                        'target_modes = ["client_runtime", "editor_host"]',
                        'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                    ]
                ),
                requested_plugin_id="sound_timeline_animation_track",
                package_id="sound_timeline_animation_track",
                diagnostics=diagnostics,
            )

            self.assertIn(
                "plugin sound_timeline_animation_track generated "
                "manifest.supported_targets must match generated "
                "feature_extensions[0].modules[0].target_modes",
                diagnostics,
            )

    def test_plugin_validate_rejects_generated_feature_provider_manifest_display_name_drift(
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
                        'display_name = "Timeline Drift Package"',
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
                    ]
                ),
                requested_plugin_id="sound_timeline_animation_track",
                package_id="sound_timeline_animation_track",
                diagnostics=diagnostics,
            )

            self.assertIn(
                "plugin sound_timeline_animation_track generated "
                "manifest.display_name must equal generated "
                "feature_extensions[0].display_name + Provider",
                diagnostics,
            )


if __name__ == "__main__":
    unittest.main()
