import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaDependenciesTests(unittest.TestCase):
    def test_manifest_schema_rejects_malformed_dependency_row(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/bridge_plugin/plugin.toml",
            plugin_manifest(
                dependencies=[
                    {
                        "id": "",
                        "required": "yes",
                        "capability": "",
                        "sidecar": "unexpected",
                    },
                    {
                        "id": "renderer",
                        "required": True,
                        "interfaces": [
                            "Renderer.Query",
                            "renderer.query",
                            "renderer.query",
                        ],
                    },
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "dependencies[0].sidecar is not a known dependency field",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "dependencies[0].id must be a non-empty trimmed string",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "dependencies[0].required must be a bool",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "dependencies[0].capability must be a non-empty trimmed string",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "dependencies[1].interfaces[0] Renderer.Query should contain "
                "only lowercase ASCII letters, digits, underscores, and dots",
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "dependencies[1].interfaces[2] renderer.query duplicates "
                "dependency interface interfaces[1]",
            ],
            violations,
        )

    def test_manifest_schema_rejects_missing_dependency_interfaces(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/bridge_plugin/plugin.toml",
            plugin_manifest(
                dependencies=[
                    {
                        "id": "renderer",
                        "required": True,
                    }
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "dependencies[0].interfaces must be a non-empty string array"
            ],
            violations,
        )

    def test_manifest_schema_rejects_duplicate_dependency_rows(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/bridge_plugin/plugin.toml",
            plugin_manifest(
                dependencies=[
                    dependency_row(),
                    dependency_row(required=False),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/bridge_plugin/plugin.toml: "
                "dependencies[1] duplicates dependency row 0"
            ],
            violations,
        )


def plugin_manifest(
    *,
    dependencies: list[dict[str, object]] | None = None,
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
    if dependencies is not None:
        manifest["dependencies"] = dependencies
    return manifest


def dependency_row(*, required: bool = True) -> dict[str, object]:
    return {
        "id": "renderer",
        "required": required,
        "capability": "runtime.capability.render",
    }


if __name__ == "__main__":
    unittest.main()
