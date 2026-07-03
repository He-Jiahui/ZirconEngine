import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaPackageKindTests(unittest.TestCase):
    def test_manifest_schema_rejects_unknown_package_kind(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["package_kind"] = "preview"

        collect_manifest_schema_violations(
            "zircon_plugins/physics/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/physics/plugin.toml: package_kind preview should be standard or feature_extension"
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_extension_package_kind_without_rows(
        self,
    ):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["package_kind"] = "feature_extension"

        collect_manifest_schema_violations(
            "zircon_plugins/physics/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/physics/plugin.toml: package_kind feature_extension should declare at least one feature_extensions row"
            ],
            violations,
        )

    def test_manifest_schema_rejects_package_kind_row_mismatch(self):
        violations: list[str] = []
        standard_manifest = plugin_manifest()
        standard_manifest["package_kind"] = "standard"
        standard_manifest["feature_extensions"] = [plugin_feature_extension()]
        feature_manifest = plugin_manifest()
        feature_manifest["package_kind"] = "feature_extension"
        feature_manifest["feature_extensions"] = [plugin_feature_extension()]
        feature_manifest["optional_features"] = [plugin_feature_extension()]

        collect_manifest_schema_violations(
            "zircon_plugins/standard/plugin.toml",
            standard_manifest,
            violations,
        )
        collect_manifest_schema_violations(
            "zircon_plugins/feature/plugin.toml",
            feature_manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/standard/plugin.toml: standard package_kind should not declare feature_extensions rows",
                "zircon_plugins/feature/plugin.toml: package_kind feature_extension should not declare optional_features rows",
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


def plugin_feature_extension() -> dict[str, object]:
    return {
        "id": "sound.preview",
        "display_name": "Sound Preview",
        "owner_plugin_id": "sound",
        "capabilities": ["runtime.feature.sound.preview"],
        "default_packaging": ["source_template"],
        "enabled_by_default": False,
        "dependencies": [
            {
                "plugin_id": "sound",
                "capability": "runtime.plugin.sound",
                "primary": True,
            }
        ],
    }


if __name__ == "__main__":
    unittest.main()
