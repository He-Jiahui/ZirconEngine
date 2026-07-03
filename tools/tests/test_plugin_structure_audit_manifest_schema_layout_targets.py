import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaLayoutTargetsTests(unittest.TestCase):
    def test_manifest_schema_rejects_duplicate_supported_targets(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                supported_targets=[
                    "client_runtime",
                    "client_runtime",
                    "server_runtime",
                    "server_runtime",
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: supported_targets[1] "
                "client_runtime duplicates supported_targets[0]",
                "zircon_plugins/sound/plugin.toml: supported_targets[3] "
                "server_runtime duplicates supported_targets[2]",
            ],
            violations,
        )


def plugin_manifest(*, supported_targets: list[str]) -> dict[str, object]:
    return {
        "id": "sound",
        "version": "0.1.0",
        "sdk_api_version": "0.1.0",
        "display_name": "Sound",
        "category": "runtime",
        "description": "Sound plugin.",
        "supported_targets": supported_targets,
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
