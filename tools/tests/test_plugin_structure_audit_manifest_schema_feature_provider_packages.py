from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.plugin_structure_audits.manifest_schema import audit_plugin_manifest_schema


class PluginStructureAuditManifestSchemaFeatureProviderPackagesTests(unittest.TestCase):
    def test_manifest_schema_audits_feature_provider_generated_package_projection(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            write_sound_feature_provider_repo(repo_root)

            report = audit_plugin_manifest_schema(repo_root).to_json()

            self.assertEqual(1, report["feature_provider_package_projection_count"])
            self.assertEqual([], report["manifest_schema_violation_details"])


def write_sound_feature_provider_repo(repo_root: Path) -> None:
    plugin_root = repo_root / "zircon_plugins"
    sound_root = plugin_root / "sound"
    sound_root.mkdir(parents=True, exist_ok=True)
    (plugin_root / "Cargo.toml").write_text(
        "\n".join(
            [
                "[workspace]",
                'members = ["sound/runtime", "sound/features/timeline_animation_track/runtime", "sound/features/timeline_animation_track/dist"]',
                'resolver = "2"',
            ]
        ),
        encoding="utf-8",
    )
    write_member_crate(sound_root / "runtime", "zircon_plugin_sound_runtime")
    write_member_crate(
        sound_root / "features" / "timeline_animation_track" / "runtime",
        "zircon_plugin_sound_timeline_animation_runtime",
    )
    write_member_crate(
        sound_root / "features" / "timeline_animation_track" / "dist",
        "zircon_plugin_sound_timeline_animation_dist",
    )
    (sound_root / "plugin.toml").write_text(
        "\n".join(
            [
                "# @generated from Rust PluginDeclaration; do not edit by hand.",
                'id = "sound"',
                'version = "0.1.0"',
                'sdk_api_version = "0.1.0"',
                'display_name = "Sound"',
                'category = "runtime"',
                'description = "Sound runtime plugin."',
                'supported_targets = ["client_runtime", "editor_host"]',
                'supported_platforms = ["windows", "linux", "macos"]',
                'capabilities = ["runtime.plugin.sound"]',
                'maturity = "beta"',
                'default_packaging = ["source_template", "library_embed", "native_dynamic"]',
                "",
                "[[modules]]",
                'name = "sound.runtime"',
                'kind = "runtime"',
                'crate_name = "zircon_plugin_sound_runtime"',
                'target_modes = ["client_runtime", "editor_host"]',
                'capabilities = ["runtime.plugin.sound"]',
                "",
                "[[optional_features]]",
                'id = "sound.timeline_animation_track"',
                'display_name = "Sound Timeline Animation Track"',
                'owner_plugin_id = "sound"',
                'provider_package_id = "sound_timeline_animation_track"',
                'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                'default_packaging = ["source_template", "library_embed", "native_dynamic"]',
                "enabled_by_default = false",
                "",
                "[[optional_features.dependencies]]",
                'plugin_id = "sound"',
                'capability = "runtime.plugin.sound"',
                "primary = true",
                "",
                "[[optional_features.modules]]",
                'name = "sound.timeline_animation_track.runtime"',
                'kind = "runtime"',
                'crate_name = "zircon_plugin_sound_timeline_animation_runtime"',
                'target_modes = ["client_runtime", "editor_host"]',
                'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                "",
                "[[optional_features.modules]]",
                'name = "sound.timeline_animation_track.dist"',
                'kind = "native"',
                'crate_name = "zircon_plugin_sound_timeline_animation_dist"',
                'target_modes = ["client_runtime", "editor_host"]',
                'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                "",
                "[optional_features.distribution]",
                'forms = ["dist"]',
                'default_packaging = ["native_dynamic"]',
                "abi_version = 3",
                'engine_compat = ">=0.1, <0.2"',
                'dist_crate = "zircon_plugin_sound_timeline_animation_dist"',
                'descriptor_symbol = "zircon_native_plugin_descriptor_v3"',
                'runtime_entry = "zircon_plugin_sound_timeline_animation_runtime_entry_v3"',
            ]
        ),
        encoding="utf-8",
    )


def write_member_crate(crate_root: Path, crate_name: str) -> None:
    crate_root.mkdir(parents=True, exist_ok=True)
    (crate_root / "Cargo.toml").write_text(
        "\n".join(
            [
                "[package]",
                f'name = "{crate_name}"',
                'version = "0.1.0"',
                'edition = "2021"',
            ]
        ),
        encoding="utf-8",
    )


if __name__ == "__main__":
    unittest.main()
