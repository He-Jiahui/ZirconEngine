import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaRootShapeTests(unittest.TestCase):
    def test_manifest_schema_rejects_root_id_semantics(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["id"] = "1physics__"
        manifest["modules"][0]["name"] = "1physics__.runtime"

        collect_manifest_schema_violations(
            "zircon_plugins/physics/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/physics/plugin.toml: id 1physics__ "
                "must start with a lowercase ASCII letter",
                "zircon_plugins/physics/plugin.toml: id 1physics__ "
                "segments must not end with an underscore or contain repeated "
                "underscores",
            ],
            violations,
        )

    def test_manifest_schema_rejects_root_version_shape(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["version"] = "1.2"
        manifest["sdk_api_version"] = "1.two.3"

        collect_manifest_schema_violations(
            "zircon_plugins/physics/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/physics/plugin.toml: version 1.2 must use "
                "MAJOR.MINOR.PATCH form",
                "zircon_plugins/physics/plugin.toml: sdk_api_version 1.two.3 "
                "minor component two must contain ASCII digits",
            ],
            violations,
        )

    def test_manifest_schema_rejects_root_version_numeric_boundaries(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["version"] = "01.2.3"
        manifest["sdk_api_version"] = "4294967296.0.0"

        collect_manifest_schema_violations(
            "zircon_plugins/physics/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/physics/plugin.toml: version 01.2.3 major "
                "component 01 must not use leading zeroes",
                "zircon_plugins/physics/plugin.toml: sdk_api_version "
                "4294967296.0.0 major component 4294967296 must fit in u32",
            ],
            violations,
        )


def plugin_manifest() -> dict[str, object]:
    return {
        "id": "physics",
        "version": "0.1.0",
        "sdk_api_version": "0.1.0",
        "display_name": "Physics",
        "category": "runtime",
        "description": "Physics plugin.",
        "supported_targets": ["client_runtime"],
        "supported_platforms": ["windows"],
        "capabilities": ["runtime.plugin.physics"],
        "maturity": "experimental",
        "default_packaging": ["source_template", "library_embed", "native_dynamic"],
        "modules": [
            {
                "name": "physics.runtime",
                "kind": "runtime",
                "crate_name": "zircon_plugin_physics_runtime",
                "target_modes": ["client_runtime"],
                "capabilities": ["runtime.plugin.physics"],
            }
        ],
    }


if __name__ == "__main__":
    unittest.main()
