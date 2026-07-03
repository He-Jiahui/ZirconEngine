import unittest

from tools.plugin_structure_audits.manifest_schema_asset_importer_capability_gates import (
    collect_asset_importer_required_capability_gate_violations,
)


class PluginStructureAuditManifestSchemaAssetImporterCapabilityGatesTests(
    unittest.TestCase
):
    def test_manifest_schema_rejects_undeclared_asset_importer_required_capability(
        self,
    ):
        violations: list[str] = []

        collect_asset_importer_required_capability_gate_violations(
            [
                (
                    "zircon_plugins/native_dynamic_fixture/plugin.toml",
                    plugin_manifest(
                        "native_dynamic_fixture",
                        capabilities=["runtime.plugin.native_dynamic_fixture"],
                        asset_importers=[
                            asset_importer(
                                [
                                    "runtime.plugin.native_dynamic_fixture",
                                    "runtime.plugin.missing_feature",
                                    "runtime.capability.asset_registry",
                                ]
                            )
                        ],
                    ),
                )
            ],
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/native_dynamic_fixture/plugin.toml: "
                "asset_importers[0].required_capabilities[1] "
                "runtime.plugin.missing_feature should reference a declared "
                "static package/feature capability or an explicitly "
                "host-owned capability",
            ],
            violations,
        )

    def test_manifest_schema_accepts_declared_feature_required_capability(self):
        violations: list[str] = []

        collect_asset_importer_required_capability_gate_violations(
            [
                (
                    "zircon_plugins/sound/plugin.toml",
                    plugin_manifest(
                        "sound",
                        capabilities=["runtime.plugin.sound"],
                        optional_features=[
                            {
                                "id": "sound.preview",
                                "capabilities": [
                                    "runtime.feature.sound.preview_assets",
                                ],
                            }
                        ],
                        asset_importers=[
                            asset_importer(
                                [
                                    "runtime.feature.sound.preview_assets",
                                    "runtime.asset.importer.native",
                                ]
                            )
                        ],
                    ),
                )
            ],
            violations,
        )

        self.assertEqual([], violations)


def plugin_manifest(
    package_id: str,
    *,
    capabilities: list[str],
    asset_importers: list[dict[str, object]],
    optional_features: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    manifest: dict[str, object] = {
        "id": package_id,
        "capabilities": capabilities,
        "asset_importers": asset_importers,
    }
    if optional_features is not None:
        manifest["optional_features"] = optional_features
    return manifest


def asset_importer(required_capabilities: list[str]) -> dict[str, object]:
    return {
        "id": "native_dynamic_fixture.json",
        "required_capabilities": required_capabilities,
    }


if __name__ == "__main__":
    unittest.main()
