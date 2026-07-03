import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaShadingModelsTests(unittest.TestCase):
    def test_manifest_schema_rejects_shading_model_descriptor_shape(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["shading_models"] = [
            {
                "id": 15,
                "token": "Toon Model",
                "forward_include": "zr_shading_toon_forward.txt",
                "gbuffer_encode_include": "zr_shading_toon_gbuffer.txt",
                "deferred_include": "zr_shading_toon_deferred.txt",
                "required_channels": -1,
                "legacy": True,
            }
        ]

        collect_manifest_schema_violations(
            "zircon_plugins/rendering/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/rendering/plugin.toml: "
                "shading_models[0].legacy is not a known shading model field",
                "zircon_plugins/rendering/plugin.toml: "
                "shading_models[0].id 15 must be a plugin shading model id >= 16",
                "zircon_plugins/rendering/plugin.toml: "
                "shading_models[0].token Toon Model must use custom:<name>",
                "zircon_plugins/rendering/plugin.toml: "
                "shading_models[0].forward_include "
                "zr_shading_toon_forward.txt must end with .wgsl",
                "zircon_plugins/rendering/plugin.toml: "
                "shading_models[0].gbuffer_encode_include "
                "zr_shading_toon_gbuffer.txt must end with .wgsl",
                "zircon_plugins/rendering/plugin.toml: "
                "shading_models[0].deferred_include "
                "zr_shading_toon_deferred.txt must end with .wgsl",
                "zircon_plugins/rendering/plugin.toml: "
                "shading_models[0].required_channels must be a u16 integer",
            ],
            violations,
        )

    def test_manifest_schema_rejects_shader_permutation_shading_model_id_mismatch(
        self,
    ):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["shading_models"] = [
            shading_model_descriptor(16, "custom:toon"),
        ]
        manifest["shader_permutation"] = {
            "shading_model_ids": [
                {"token": "custom:toon", "id": 17},
                {"token": "custom:subsurface", "id": 16},
            ]
        }

        collect_manifest_schema_violations(
            "zircon_plugins/rendering/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/rendering/plugin.toml: "
                "shader_permutation.shading_model_ids[0].token custom:toon "
                "was already assigned id 16 and cannot be reused by id 17",
                "zircon_plugins/rendering/plugin.toml: "
                "shader_permutation.shading_model_ids[1].id 16 was already "
                "assigned to custom:toon and cannot be reused by "
                "custom:subsurface",
            ],
            violations,
        )


def plugin_manifest() -> dict[str, object]:
    return {
        "id": "rendering",
        "version": "0.1.0",
        "sdk_api_version": "0.1.0",
        "display_name": "Rendering",
        "category": "rendering",
        "description": "Rendering plugin.",
        "supported_targets": ["client_runtime"],
        "supported_platforms": ["windows"],
        "capabilities": ["runtime.plugin.rendering"],
        "maturity": "experimental",
        "default_packaging": ["source_template", "library_embed", "native_dynamic"],
        "modules": [
            {
                "name": "rendering.runtime",
                "kind": "runtime",
                "crate_name": "zircon_plugin_rendering_runtime",
                "target_modes": ["client_runtime"],
                "capabilities": ["runtime.plugin.rendering"],
            }
        ],
    }


def shading_model_descriptor(id_value: int, token: str) -> dict[str, object]:
    return {
        "id": id_value,
        "token": token,
        "forward_include": "zr_shading_toon_forward.wgsl",
        "gbuffer_encode_include": "zr_shading_toon_gbuffer.wgsl",
        "deferred_include": "zr_shading_toon_deferred.wgsl",
        "required_channels": 7,
    }


if __name__ == "__main__":
    unittest.main()
