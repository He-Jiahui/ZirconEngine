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


class PluginValidateFeatureProviderManifestMetadataValueSchemaTests(
    unittest.TestCase
):
    def test_plugin_validate_rejects_generated_feature_provider_manifest_metadata_value_schema_drift(
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
                        'maturity = "beta"',
                        'supported_targets = ["client_runtime", "editor_host"]',
                        'supported_platforms = ["windows", "linux", "macos"]',
                        'capabilities = ["Runtime.Bad", "Runtime.Bad"]',
                        'default_packaging = ["nightly_dynamic", "nightly_dynamic"]',
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
                "manifest.capabilities[0] Runtime.Bad should contain only "
                "lowercase ASCII letters, digits, underscores, and dots",
                diagnostics,
            )
            self.assertIn(
                "plugin sound_timeline_animation_track generated "
                "manifest.capabilities[1] Runtime.Bad duplicates capabilities "
                "capabilities[0]",
                diagnostics,
            )
            self.assertIn(
                'plugin sound_timeline_animation_track generated '
                'manifest.default_packaging[0] "nightly_dynamic" is '
                "unsupported; expected one of source_template, library_embed, "
                "native_dynamic",
                diagnostics,
            )
            self.assertIn(
                "plugin sound_timeline_animation_track generated "
                "manifest.default_packaging[1] nightly_dynamic duplicates "
                "default_packaging[0]",
                diagnostics,
            )


if __name__ == "__main__":
    unittest.main()
