import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaFeatureDistributionContractTests(
    unittest.TestCase
):
    def test_manifest_schema_rejects_optional_feature_distribution_contract(self):
        violations: list[str] = []
        distribution = feature_distribution(
            forms=["embed", "sidecar"],
            default_packaging=["library_embed", "zip"],
            assets="assets/**",
            preview_channel="nightly",
        )
        distribution["abi_version"] = 2
        distribution["engine_compat"] = ">=0"
        distribution["descriptor_symbol"] = "zircon_native_plugin_descriptor_v2"
        distribution["runtime_entry"] = " entry "

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(distribution=distribution),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].distribution.preview_channel "
                "is not a known distribution field",
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].distribution.runtime_entry "
                "must be a non-empty trimmed string",
                'zircon_plugins/sound/plugin.toml: optional_features[0].'
                'distribution.engine_compat ">=0" is invalid: version "0" '
                "must be major.minor[.patch]",
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].distribution.abi_version must be 3",
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].distribution.descriptor_symbol must equal "
                "zircon_native_plugin_descriptor_v3",
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].distribution must declare runtime_entry "
                "or editor_entry",
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].distribution.forms must include dist",
                'zircon_plugins/sound/plugin.toml: optional_features[0].'
                'distribution.forms[1] "sidecar" is unsupported; '
                "expected one of dist, embed",
                "zircon_plugins/sound/plugin.toml: optional_features[0]."
                "distribution.default_packaging must include native_dynamic",
                'zircon_plugins/sound/plugin.toml: optional_features[0].'
                'distribution.default_packaging[1] "zip" is unsupported; '
                "expected one of source_template, library_embed, native_dynamic",
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].distribution.assets must be an array",
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_extension_distribution_contract(self):
        violations: list[str] = []
        distribution = feature_distribution(
            forms=["embed"],
            default_packaging=["source_template"],
            assets=["editor/ui/panel.ui.toml"],
        )
        distribution["abi_version"] = 2
        distribution["descriptor_symbol"] = "zircon_native_plugin_descriptor_v2"
        del distribution["runtime_entry"]

        manifest = plugin_manifest()
        manifest["package_kind"] = "feature_extension"
        manifest["feature_extensions"] = [
            plugin_feature(distribution=distribution),
        ]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].distribution.abi_version must be 3",
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].distribution.descriptor_symbol must equal "
                "zircon_native_plugin_descriptor_v3",
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].distribution must declare runtime_entry "
                "or editor_entry",
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].distribution.forms must include dist",
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].distribution.default_packaging "
                "must include native_dynamic",
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].distribution.assets[0] targets retired "
                "UI asset suffix editor/ui/panel.ui.toml; use .zui",
            ],
            violations,
        )


def plugin_manifest(
    *,
    optional_features: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    manifest: dict[str, object] = {
        "id": "sound",
        "version": "0.1.0",
        "sdk_api_version": "0.1.0",
        "display_name": "Sound",
        "category": "runtime",
        "description": "Sound plugin.",
        "supported_targets": ["client_runtime"],
        "supported_platforms": ["windows"],
        "capabilities": ["runtime.plugin.sound"],
        "maturity": "experimental",
        "default_packaging": ["source_template", "library_embed", "native_dynamic"],
        "modules": [
            {
                "name": "sound.runtime",
                "kind": "runtime",
                "crate_name": "zircon_plugin_sound_runtime",
                "target_modes": ["client_runtime"],
                "capabilities": ["runtime.plugin.sound"],
            }
        ],
    }
    if optional_features is not None:
        manifest["optional_features"] = optional_features
    return manifest


def plugin_feature(
    *,
    distribution: object | None = None,
) -> dict[str, object]:
    feature = {
        "id": "sound.preview",
        "display_name": "Preview",
        "owner_plugin_id": "sound",
        "capabilities": ["runtime.feature.sound.preview"],
        "default_packaging": ["native_dynamic"],
        "enabled_by_default": False,
        "dependencies": [
            {
                "plugin_id": "sound",
                "capability": "runtime.plugin.sound",
                "primary": True,
            }
        ],
    }
    if distribution is not None:
        feature["distribution"] = distribution
    return feature


def feature_distribution(
    *,
    forms: list[str] | None = None,
    default_packaging: list[str] | None = None,
    assets: object | None = None,
    preview_channel: str | None = None,
) -> dict[str, object]:
    distribution: dict[str, object] = {
        "forms": forms or ["dist"],
        "default_packaging": default_packaging or ["native_dynamic"],
        "abi_version": 3,
        "engine_compat": ">=0.1, <0.2",
        "dist_crate": "zircon_plugin_sound_preview_dist",
        "descriptor_symbol": "zircon_native_plugin_descriptor_v3",
        "runtime_entry": "zircon_plugin_sound_preview_runtime_entry_v3",
    }
    if assets is not None:
        distribution["assets"] = assets
    if preview_channel is not None:
        distribution["preview_channel"] = preview_channel
    return distribution


if __name__ == "__main__":
    unittest.main()
