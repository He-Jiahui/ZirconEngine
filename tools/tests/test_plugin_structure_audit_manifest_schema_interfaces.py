import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaInterfacesTests(unittest.TestCase):
    def test_manifest_schema_rejects_malformed_provided_interface_row(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/bridge_plugin/plugin.toml",
            plugin_manifest(
                provides_interfaces=[
                    {
                        "id": "Bridge.Plugin.Runtime",
                        "methods": [
                            {
                                "name": "Tick",
                                "method_slot": "0",
                                "return_value_kind": "vector",
                                "documentation": " method docs ",
                                "required_capabilities": ["Runtime.Bad"],
                                "parameters": [
                                    {
                                        "name": "Payload",
                                        "value_kind": "Vec3",
                                        "type_ref": {
                                            "value_kind": "BadKind",
                                            "type_name": "",
                                            "sidecar": "drift",
                                        },
                                        "sidecar": "drift",
                                    }
                                ],
                                "sidecar": "drift",
                            }
                        ],
                        "sidecar": "drift",
                    }
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[0].sidecar is not a known provided "
                "interface field",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[0].methods[0].sidecar is not a known "
                "provided interface method field",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[0].methods[0].name Tick should contain "
                "only lowercase ASCII letters, digits, and underscores",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[0].methods[0].method_slot must be a "
                "non-negative integer",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[0].methods[0].return_value_kind vector "
                "is unsupported; expected one of null, bool, int, float, "
                "string, bytes, host_handle",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[0].methods[0].parameters[0].sidecar is "
                "not a known interface method parameter field",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[0].methods[0].parameters[0].value_kind "
                "Vec3 is unsupported; expected one of null, bool, int, "
                "float, string, bytes, host_handle",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[0].methods[0].parameters[0].type_ref."
                "sidecar is not a known interface method type_ref field",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[0].methods[0].parameters[0].type_ref."
                "value_kind BadKind is unsupported; expected one of null, "
                "bool, int, float, string, bytes, host_handle",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[0].methods[0].parameters[0].type_ref."
                "type_name must be a non-empty trimmed string",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[0].methods[0].parameters[0].name "
                "Payload should contain only lowercase ASCII letters, digits, "
                "and underscores",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[0].methods[0].required_capabilities[0] "
                "Runtime.Bad should contain only lowercase ASCII letters, "
                "digits, underscores, and dots",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[0].methods[0].documentation must be a "
                "non-empty trimmed string",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[0].id Bridge.Plugin.Runtime should "
                "contain only lowercase ASCII letters, digits, underscores, "
                "and dots",
            ],
            violations,
        )

    def test_manifest_schema_rejects_duplicate_provided_interface_members(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/bridge_plugin/plugin.toml",
            plugin_manifest(
                provides_interfaces=[
                    {
                        "id": "bridge_plugin.runtime",
                        "methods": [
                            {
                                "name": "tick",
                                "method_slot": 0,
                                "parameters": [
                                    {"name": "payload", "value_kind": "bytes"},
                                    {"name": "payload", "value_kind": "string"},
                                ],
                                "required_capabilities": [
                                    "runtime.plugin.bridge_plugin",
                                    "runtime.plugin.bridge_plugin",
                                ],
                            },
                            {"name": "tick", "method_slot": 0},
                        ],
                    },
                    {
                        "id": "bridge_plugin.runtime",
                    },
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[0].methods[0].parameters[1].name payload "
                "duplicates interface method parameter parameters[0]",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[0].methods[0].required_capabilities[1] "
                "runtime.plugin.bridge_plugin duplicates required capability "
                "required_capabilities[0]",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[0].methods[1].name tick duplicates "
                "provided interface method methods[0]",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[0].methods[1].method_slot 0 duplicates "
                "provided interface method_slot methods[0]",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[1].id bridge_plugin.runtime duplicates "
                "provided interface id provides_interfaces[0]",
            ],
            violations,
        )

    def test_manifest_schema_rejects_empty_provided_interface_methods(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/bridge_plugin/plugin.toml",
            plugin_manifest(
                provides_interfaces=[
                    {
                        "id": "bridge_plugin.runtime",
                        "methods": [],
                    }
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "provides_interfaces[0].methods must not be empty when declared"
            ],
            violations,
        )


def plugin_manifest(
    *,
    provides_interfaces: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    manifest: dict[str, object] = {
        "id": "bridge_plugin",
        "version": "0.1.0",
        "sdk_api_version": "0.1.0",
        "display_name": "Bridge Plugin",
        "category": "runtime",
        "description": "Bridge plugin.",
        "supported_targets": ["client_runtime"],
        "supported_platforms": ["windows"],
        "capabilities": ["runtime.plugin.bridge_plugin"],
        "maturity": "stable",
        "default_packaging": ["source_template"],
        "modules": [
            {
                "name": "bridge_plugin.runtime",
                "kind": "runtime",
                "crate_name": "zircon_plugin_bridge_plugin_runtime",
                "target_modes": ["client_runtime"],
                "capabilities": ["runtime.plugin.bridge_plugin"],
            }
        ],
    }
    if provides_interfaces is not None:
        manifest["provides_interfaces"] = provides_interfaces
    return manifest


if __name__ == "__main__":
    unittest.main()
