import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaRootDistributionTests(unittest.TestCase):
    def test_manifest_schema_rejects_root_distribution_non_table(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["distribution"] = "dist"

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            ["zircon_plugins/sound/plugin.toml: distribution must be a table"],
            violations,
        )

    def test_manifest_schema_rejects_root_distribution_packaging_contract(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["distribution"] = root_distribution(
            forms=["embed", "sidecar", "dist", "dist"],
            default_packaging=[
                "library_embed",
                "zip",
                "native_dynamic",
                "native_dynamic",
            ],
            preview_channel="nightly",
        )

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: distribution.preview_channel "
                "is not a known distribution field",
                'zircon_plugins/sound/plugin.toml: distribution.forms[1] "sidecar" '
                "is unsupported; expected one of dist, embed",
                "zircon_plugins/sound/plugin.toml: distribution.forms[3] dist "
                "duplicates distribution.forms[2]",
                "zircon_plugins/sound/plugin.toml: distribution.default_packaging[1] "
                '"zip" is unsupported; expected one of source_template, '
                "library_embed, native_dynamic",
                "zircon_plugins/sound/plugin.toml: distribution.default_packaging[3] "
                "native_dynamic duplicates distribution.default_packaging[2]",
            ],
            violations,
        )

    def test_manifest_schema_rejects_root_distribution_v3_contract(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        distribution = root_distribution()
        distribution["abi_version"] = 2
        distribution["engine_compat"] = ">=0"
        distribution["descriptor_symbol"] = "zircon_native_plugin_descriptor_v2"
        del distribution["runtime_entry"]
        manifest["distribution"] = distribution

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                'zircon_plugins/sound/plugin.toml: distribution.engine_compat ">=0" '
                'is invalid: version "0" must be major.minor[.patch]',
                "zircon_plugins/sound/plugin.toml: distribution.abi_version must be 3",
                "zircon_plugins/sound/plugin.toml: distribution.descriptor_symbol "
                "must equal zircon_native_plugin_descriptor_v3",
                "zircon_plugins/sound/plugin.toml: distribution must declare "
                "runtime_entry or editor_entry",
            ],
            violations,
        )

    def test_manifest_schema_rejects_root_distribution_assets_contract(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["distribution"] = root_distribution(
            assets=[
                " assets/**",
                "../outside/**",
                "editor/ui/panel.ui.toml",
                "editor/ui/panel.v2.ui.toml",
            ],
        )

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: distribution.assets[0] "
                "must be trimmed",
                "zircon_plugins/sound/plugin.toml: distribution.assets[1] "
                "must be a plugin-relative glob",
                "zircon_plugins/sound/plugin.toml: distribution.assets[2] targets "
                "retired UI asset suffix editor/ui/panel.ui.toml; use .zui",
                "zircon_plugins/sound/plugin.toml: distribution.assets[3] targets "
                "retired UI asset suffix editor/ui/panel.v2.ui.toml; use .zui",
            ],
            violations,
        )


def plugin_manifest() -> dict[str, object]:
    return {
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


def root_distribution(
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
        "dist_crate": "zircon_plugin_sound_dist",
        "descriptor_symbol": "zircon_native_plugin_descriptor_v3",
        "runtime_entry": "zircon_plugin_sound_runtime_entry_v3",
    }
    if assets is not None:
        distribution["assets"] = assets
    if preview_channel is not None:
        distribution["preview_channel"] = preview_channel
    return distribution


if __name__ == "__main__":
    unittest.main()
