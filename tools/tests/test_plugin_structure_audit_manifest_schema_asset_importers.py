import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaAssetImportersTests(unittest.TestCase):
    def test_manifest_schema_rejects_asset_importer_retired_ui_suffixes(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/ui_document_importer/plugin.toml",
            plugin_manifest(
                asset_importers=[
                    asset_importer(
                        full_suffixes=[".ui.toml", ".v2.ui.toml"],
                    )
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/ui_document_importer/plugin.toml: "
                "asset_importers[0].full_suffixes[0] "
                "declares retired UI asset suffix .ui.toml; use .zui",
                "zircon_plugins/ui_document_importer/plugin.toml: "
                "asset_importers[0].full_suffixes[1] "
                "declares retired UI asset suffix .v2.ui.toml; use .zui",
            ],
            violations,
        )

    def test_manifest_schema_rejects_asset_importer_selector_and_suffix_format(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/ui_document_importer/plugin.toml",
            plugin_manifest(
                asset_importers=[
                    asset_importer(
                        source_extensions=[".zui", "ZUI"],
                        full_suffixes=["zui", ".ZUI", ".zui", ".zui"],
                    )
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/ui_document_importer/plugin.toml: "
                "asset_importers[0].source_extensions[0] must be a lowercase "
                "extension without dots; use full_suffixes for dotted suffixes",
                "zircon_plugins/ui_document_importer/plugin.toml: "
                "asset_importers[0].source_extensions[1] must be lowercase",
                "zircon_plugins/ui_document_importer/plugin.toml: "
                "asset_importers[0].full_suffixes[0] must be a dotted suffix",
                "zircon_plugins/ui_document_importer/plugin.toml: "
                "asset_importers[0].full_suffixes[1] must be lowercase",
                "zircon_plugins/ui_document_importer/plugin.toml: "
                "asset_importers[0].full_suffixes[3] duplicates entry 2",
            ],
            violations,
        )

    def test_manifest_schema_rejects_asset_importer_missing_source_selector(self):
        violations: list[str] = []
        importer = asset_importer()
        del importer["full_suffixes"]

        collect_manifest_schema_violations(
            "zircon_plugins/ui_document_importer/plugin.toml",
            plugin_manifest(asset_importers=[importer]),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/ui_document_importer/plugin.toml: "
                "asset_importers[0] must declare source_extensions or full_suffixes"
            ],
            violations,
        )

    def test_manifest_schema_rejects_unknown_asset_importer_field(self):
        violations: list[str] = []
        importer = asset_importer()
        importer["sidecar"] = "legacy"

        collect_manifest_schema_violations(
            "zircon_plugins/ui_document_importer/plugin.toml",
            plugin_manifest(asset_importers=[importer]),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/ui_document_importer/plugin.toml: "
                "asset_importers[0].sidecar is not a known asset_importer field"
            ],
            violations,
        )


def plugin_manifest(
    *,
    asset_importers: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    return {
        "id": "ui_document_importer",
        "version": "0.1.0",
        "sdk_api_version": "0.1.0",
        "display_name": "UI Document Importer",
        "category": "asset_importer",
        "description": "Runtime UI document importer package.",
        "supported_targets": ["client_runtime", "editor_host"],
        "supported_platforms": ["windows"],
        "capabilities": [
            "runtime.plugin.ui_document_importer",
            "runtime.asset.importer.ui_document",
        ],
        "maturity": "stable",
        "default_packaging": ["source_template", "library_embed", "native_dynamic"],
        "asset_importers": asset_importers or [asset_importer()],
        "modules": [
            {
                "name": "ui_document_importer.runtime",
                "kind": "runtime",
                "crate_name": "zircon_plugin_ui_document_importer_runtime",
                "target_modes": ["client_runtime", "editor_host"],
                "capabilities": [
                    "runtime.plugin.ui_document_importer",
                    "runtime.asset.importer.ui_document",
                ],
            }
        ],
    }


def asset_importer(
    *,
    source_extensions: list[str] | None = None,
    full_suffixes: list[str] | None = None,
) -> dict[str, object]:
    importer: dict[str, object] = {
        "id": "ui_document_importer.zui_document",
        "plugin_id": "ui_document_importer",
        "priority": 120,
        "full_suffixes": full_suffixes or [".zui"],
        "output_kind": "UiWidget",
        "additional_output_kinds": ["UiLayout", "UiStyle"],
        "importer_version": 2,
        "required_capabilities": ["runtime.asset.importer.ui_document"],
    }
    if source_extensions is not None:
        importer["source_extensions"] = source_extensions
    return importer


if __name__ == "__main__":
    unittest.main()
