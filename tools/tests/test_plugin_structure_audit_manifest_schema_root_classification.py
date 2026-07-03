import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaRootClassificationTests(unittest.TestCase):
    def test_manifest_schema_rejects_unknown_root_category(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["category"] = "tooling"

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                'zircon_plugins/sound/plugin.toml: category "tooling" '
                "is unsupported; expected one of asset_importer, authoring, "
                "diagnostics, platform, rendering, runtime, sdk"
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


if __name__ == "__main__":
    unittest.main()
