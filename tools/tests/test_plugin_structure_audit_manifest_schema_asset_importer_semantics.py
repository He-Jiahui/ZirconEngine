import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaAssetImporterSemanticsTests(unittest.TestCase):
    def test_manifest_schema_rejects_asset_importer_id_and_resource_kind_drift(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["asset_importers"] = [
            asset_importer(
                id_value="shader.Bad",
                output_kind="ShaderDoc",
                additional_output_kinds=["Mesh", "ShaderDoc", "Mesh"],
            )
        ]

        collect_manifest_schema_violations(
            "zircon_plugins/shader_wgsl_importer/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/shader_wgsl_importer/plugin.toml: "
                "asset_importers[0].id shader.Bad must contain only "
                "lowercase ASCII letters, digits, underscores, and dots",
                "zircon_plugins/shader_wgsl_importer/plugin.toml: "
                "asset_importers[0].output_kind must be a known ResourceKind",
                "zircon_plugins/shader_wgsl_importer/plugin.toml: "
                "asset_importers[0].additional_output_kinds[1] must be a "
                "known ResourceKind",
                "zircon_plugins/shader_wgsl_importer/plugin.toml: "
                "asset_importers[0].additional_output_kinds[2] duplicates entry 0",
            ],
            violations,
        )

    def test_manifest_schema_rejects_asset_importer_capability_and_number_drift(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["asset_importers"] = [
            asset_importer(
                priority=2**31,
                importer_version=2**32 + 1,
                required_capabilities=[
                    "bad",
                    "Runtime.Asset.Import",
                    "runtime.asset.importer.native",
                    "runtime.asset.importer.native",
                ],
            )
        ]

        collect_manifest_schema_violations(
            "zircon_plugins/shader_wgsl_importer/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/shader_wgsl_importer/plugin.toml: "
                "asset_importers[0].priority must fit i32",
                "zircon_plugins/shader_wgsl_importer/plugin.toml: "
                "asset_importers[0].importer_version must be a positive u32",
                "zircon_plugins/shader_wgsl_importer/plugin.toml: "
                "asset_importers[0].required_capabilities[0] must use at least "
                "two dot-separated namespace segments",
                "zircon_plugins/shader_wgsl_importer/plugin.toml: "
                "asset_importers[0].required_capabilities[1] must contain only "
                "lowercase ASCII letters, digits, underscores, and dots",
                "zircon_plugins/shader_wgsl_importer/plugin.toml: "
                "asset_importers[0].required_capabilities[3] duplicates entry 2",
            ],
            violations,
        )


def plugin_manifest() -> dict[str, object]:
    return {
        "id": "shader_wgsl_importer",
        "version": "0.1.0",
        "sdk_api_version": "0.1.0",
        "display_name": "Shader WGSL Importer",
        "category": "asset_importer",
        "description": "Shader importer plugin.",
        "supported_targets": ["client_runtime"],
        "supported_platforms": ["windows"],
        "capabilities": ["runtime.asset.importer.shader"],
        "maturity": "experimental",
        "default_packaging": ["source_template", "library_embed", "native_dynamic"],
        "modules": [
            {
                "name": "shader_wgsl_importer.runtime",
                "kind": "runtime",
                "crate_name": "zircon_plugin_shader_wgsl_importer_runtime",
                "target_modes": ["client_runtime"],
                "capabilities": ["runtime.asset.importer.shader"],
            }
        ],
    }


def asset_importer(
    *,
    id_value: str = "shader.wgsl",
    priority: int = 100,
    output_kind: str = "Shader",
    additional_output_kinds: list[str] | None = None,
    importer_version: int = 1,
    required_capabilities: list[str] | None = None,
) -> dict[str, object]:
    importer: dict[str, object] = {
        "id": id_value,
        "plugin_id": "shader_wgsl_importer",
        "priority": priority,
        "source_extensions": ["wgsl"],
        "output_kind": output_kind,
        "importer_version": importer_version,
    }
    if additional_output_kinds is not None:
        importer["additional_output_kinds"] = additional_output_kinds
    if required_capabilities is not None:
        importer["required_capabilities"] = required_capabilities
    return importer


if __name__ == "__main__":
    unittest.main()
