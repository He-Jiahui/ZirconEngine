import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaLayoutCoordinatesTests(unittest.TestCase):
    def test_manifest_schema_rejects_partial_package_coordinate_set(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["package_prefix"] = "com..Example"
        manifest["package_company"] = "BadCompany"

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: package coordinates must declare "
                "package_prefix, package_company, and package_name together or leave all empty",
                "zircon_plugins/sound/plugin.toml: package_prefix com..Example "
                "must contain only non-empty lowercase coordinate segments",
                "zircon_plugins/sound/plugin.toml: package_company BadCompany "
                "must be a non-empty lowercase coordinate segment",
                "zircon_plugins/sound/plugin.toml: package_name  "
                "must be a non-empty lowercase coordinate segment",
            ],
            violations,
        )

    def test_manifest_schema_rejects_package_coordinate_segment_drift(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["package_prefix"] = " com.example "
        manifest["package_company"] = "zircon-engine"
        manifest["package_name"] = 42

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: package coordinates must declare "
                "package_prefix, package_company, and package_name together or leave all empty",
                "zircon_plugins/sound/plugin.toml: package_prefix  com.example  "
                "must contain only non-empty lowercase coordinate segments",
                "zircon_plugins/sound/plugin.toml: package_company zircon-engine "
                "must be a non-empty lowercase coordinate segment",
                "zircon_plugins/sound/plugin.toml: package_name 42 "
                "must be a non-empty lowercase coordinate segment",
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
