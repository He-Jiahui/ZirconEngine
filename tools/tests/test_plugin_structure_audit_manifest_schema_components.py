import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaComponentsTests(unittest.TestCase):
    def test_manifest_schema_rejects_malformed_component_row(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/ui_plugin/plugin.toml",
            plugin_manifest(
                components=[
                    {
                        "type_id": "Ui.Plugin.Panel",
                        "plugin_id": "other_plugin",
                        "display_name": " Panel ",
                        "properties": [
                            {
                                "name": " speed ",
                                "value_type": "",
                                "editable": "yes",
                                "sidecar": "unexpected",
                            }
                        ],
                        "sidecar": "unexpected",
                    }
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/ui_plugin/plugin.toml: "
                "components[0].sidecar is not a known component field",
                "zircon_plugins/ui_plugin/plugin.toml: "
                "components[0].type_id Ui.Plugin.Panel should contain only "
                "lowercase ASCII letters, digits, underscores, and dots",
                "zircon_plugins/ui_plugin/plugin.toml: "
                "components[0].type_id Ui.Plugin.Panel should stay under "
                "package namespace ui_plugin.",
                "zircon_plugins/ui_plugin/plugin.toml: "
                "components[0].plugin_id other_plugin should match package id "
                "ui_plugin",
                "zircon_plugins/ui_plugin/plugin.toml: "
                "components[0].display_name must be a non-empty trimmed string",
                "zircon_plugins/ui_plugin/plugin.toml: "
                "components[0].properties[0].sidecar is not a known component "
                "property field",
                "zircon_plugins/ui_plugin/plugin.toml: "
                "components[0].properties[0].name must be a non-empty trimmed "
                "string",
                "zircon_plugins/ui_plugin/plugin.toml: "
                "components[0].properties[0].value_type must be a non-empty "
                "trimmed string",
                "zircon_plugins/ui_plugin/plugin.toml: "
                "components[0].properties[0].editable must be a bool",
            ],
            violations,
        )

    def test_manifest_schema_rejects_duplicate_component_identity(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/ui_plugin/plugin.toml",
            plugin_manifest(
                components=[
                    component_row(),
                    component_row(),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/ui_plugin/plugin.toml: "
                "components[1].type_id ui_plugin.transform duplicates "
                "component type_id row 0"
            ],
            violations,
        )

    def test_manifest_schema_rejects_ui_component_retired_document_path(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/ui_plugin/plugin.toml",
            plugin_manifest(
                ui_components=[
                    {
                        "component_id": "Ui.Plugin.Panel",
                        "plugin_id": "other_plugin",
                        "ui_document": "../panel.ui.toml",
                        "sidecar": "unexpected",
                    }
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/ui_plugin/plugin.toml: "
                "ui_components[0].sidecar is not a known ui_component field",
                "zircon_plugins/ui_plugin/plugin.toml: "
                "ui_components[0].component_id Ui.Plugin.Panel should contain "
                "only lowercase ASCII letters, digits, underscores, and dots",
                "zircon_plugins/ui_plugin/plugin.toml: "
                "ui_components[0].component_id Ui.Plugin.Panel should stay "
                "under package namespace ui_plugin.",
                "zircon_plugins/ui_plugin/plugin.toml: "
                "ui_components[0].plugin_id other_plugin should match package id "
                "ui_plugin",
                "zircon_plugins/ui_plugin/plugin.toml: "
                "ui_components[0].ui_document ../panel.ui.toml should reference "
                "a .zui component asset",
                "zircon_plugins/ui_plugin/plugin.toml: "
                "ui_components[0].ui_document ../panel.ui.toml should be a "
                "relative forward-slash package path",
            ],
            violations,
        )


def plugin_manifest(
    *,
    components: list[dict[str, object]] | None = None,
    ui_components: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    manifest: dict[str, object] = {
        "id": "ui_plugin",
        "version": "0.1.0",
        "sdk_api_version": "0.1.0",
        "display_name": "UI Plugin",
        "category": "authoring",
        "description": "UI plugin.",
        "supported_targets": ["editor_host"],
        "supported_platforms": ["windows"],
        "capabilities": ["editor.plugin.ui_plugin"],
        "maturity": "stable",
        "default_packaging": ["source_template"],
        "modules": [
            {
                "name": "ui_plugin.editor",
                "kind": "editor",
                "crate_name": "zircon_plugin_ui_plugin_editor",
                "target_modes": ["editor_host"],
                "capabilities": ["editor.plugin.ui_plugin"],
            }
        ],
    }
    if components is not None:
        manifest["components"] = components
    if ui_components is not None:
        manifest["ui_components"] = ui_components
    return manifest


def component_row() -> dict[str, object]:
    return {
        "type_id": "ui_plugin.transform",
        "plugin_id": "ui_plugin",
        "display_name": "Transform",
    }


if __name__ == "__main__":
    unittest.main()
