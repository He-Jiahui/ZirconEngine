import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaGeometrySourcesTests(unittest.TestCase):
    def test_manifest_schema_rejects_geometry_source_descriptor_shape(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["geometry_sources"] = [
            {
                "id": 3,
                "token": "virtual geometry",
                "wgsl_include": "zr_geometry_virtual_geometry.txt",
                "vertex_attributes": ["position", "bad_attribute", "position"],
                "required_bindings": [
                    {
                        "kind": "bad_binding",
                        "slot_token": "Virtual Geometry.Pages",
                        "extra": True,
                    },
                    "bad",
                ],
                "shader_defines": [
                    {"kind": "bool", "name": " ZR_BAD", "value": "true"},
                    {"kind": "float", "name": "ZR_UNKNOWN", "value": 1},
                ],
                "legacy": True,
            }
        ]

        collect_manifest_schema_violations(
            "zircon_plugins/virtual_geometry/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/virtual_geometry/plugin.toml: "
                "geometry_sources[0].legacy is not a known geometry source field",
                "zircon_plugins/virtual_geometry/plugin.toml: "
                "geometry_sources[0].id 3 must be a plugin geometry source id >= 4",
                "zircon_plugins/virtual_geometry/plugin.toml: "
                "geometry_sources[0].token virtual geometry must use custom:<name>",
                "zircon_plugins/virtual_geometry/plugin.toml: "
                "geometry_sources[0].wgsl_include "
                "zr_geometry_virtual_geometry.txt must end with .wgsl",
                "zircon_plugins/virtual_geometry/plugin.toml: "
                'geometry_sources[0].vertex_attributes[1] "bad_attribute" is '
                "unsupported; expected one of position, normal, tangent, uv0, "
                "color0, joint_indices, joint_weights, morph_position_delta, "
                "morph_normal_delta",
                "zircon_plugins/virtual_geometry/plugin.toml: "
                "geometry_sources[0].vertex_attributes[2] position duplicates "
                "vertex_attributes[0]",
                "zircon_plugins/virtual_geometry/plugin.toml: "
                "geometry_sources[0].required_bindings[0].extra is not a known "
                "geometry source binding field",
                "zircon_plugins/virtual_geometry/plugin.toml: "
                'geometry_sources[0].required_bindings[0].kind "bad_binding" is '
                "unsupported; expected one of gpu_scene_instance, "
                "skinning_palette_storage, morph_weights_storage, "
                "morph_target_storage, virtual_geometry_pages, "
                "virtual_geometry_clusters",
                "zircon_plugins/virtual_geometry/plugin.toml: "
                "geometry_sources[0].required_bindings[0].slot_token "
                "Virtual Geometry.Pages should contain only lowercase ASCII "
                "letters, digits, underscores, and dots",
                "zircon_plugins/virtual_geometry/plugin.toml: "
                "geometry_sources[0].required_bindings[1] must be a table",
                "zircon_plugins/virtual_geometry/plugin.toml: "
                "geometry_sources[0].shader_defines[0].name must be a "
                "non-empty trimmed shader define name",
                "zircon_plugins/virtual_geometry/plugin.toml: "
                "geometry_sources[0].shader_defines[0].value must be a bool",
                "zircon_plugins/virtual_geometry/plugin.toml: "
                'geometry_sources[0].shader_defines[1].kind "float" is unsupported; '
                "expected one of bool, int, uint",
            ],
            violations,
        )

    def test_manifest_schema_rejects_shader_permutation_geometry_source_id_mismatch(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["geometry_sources"] = [
            geometry_source_descriptor(4, "custom:virtual_geometry"),
        ]
        manifest["shader_permutation"] = {
            "geometry_source_ids": [
                {"token": "custom:virtual_geometry", "id": 5},
                {"token": "custom:foliage", "id": 4},
            ]
        }

        collect_manifest_schema_violations(
            "zircon_plugins/virtual_geometry/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/virtual_geometry/plugin.toml: "
                "shader_permutation.geometry_source_ids[0].token "
                "custom:virtual_geometry was already assigned id 4 and cannot "
                "be reused by id 5",
                "zircon_plugins/virtual_geometry/plugin.toml: "
                "shader_permutation.geometry_source_ids[1].id 4 was already "
                "assigned to custom:virtual_geometry and cannot be reused by "
                "custom:foliage",
            ],
            violations,
        )


def plugin_manifest() -> dict[str, object]:
    return {
        "id": "virtual_geometry",
        "version": "0.1.0",
        "sdk_api_version": "0.1.0",
        "display_name": "Virtual Geometry",
        "category": "rendering",
        "description": "Virtual geometry plugin.",
        "supported_targets": ["client_runtime"],
        "supported_platforms": ["windows"],
        "capabilities": ["runtime.plugin.virtual_geometry"],
        "maturity": "experimental",
        "default_packaging": ["source_template", "library_embed", "native_dynamic"],
        "modules": [
            {
                "name": "virtual_geometry.runtime",
                "kind": "runtime",
                "crate_name": "zircon_plugin_virtual_geometry_runtime",
                "target_modes": ["client_runtime"],
                "capabilities": ["runtime.plugin.virtual_geometry"],
            }
        ],
    }


def geometry_source_descriptor(id_value: int, token: str) -> dict[str, object]:
    return {
        "id": id_value,
        "token": token,
        "wgsl_include": "zr_geometry_virtual_geometry.wgsl",
        "vertex_attributes": ["position", "normal", "tangent", "uv0"],
        "required_bindings": [
            {
                "kind": "virtual_geometry_pages",
                "slot_token": "virtual_geometry.pages",
            }
        ],
        "shader_defines": [
            {
                "kind": "bool",
                "name": "ZR_GEOMETRY_SOURCE_VIRTUAL_GEOMETRY",
                "value": True,
            }
        ],
    }


if __name__ == "__main__":
    unittest.main()
