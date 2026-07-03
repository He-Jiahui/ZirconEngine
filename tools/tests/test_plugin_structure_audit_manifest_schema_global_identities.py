import unittest

from tools.plugin_structure_audits.manifest_schema_global_identities import (
    collect_global_manifest_identity_violations,
)


class PluginStructureAuditManifestSchemaGlobalIdentitiesTests(unittest.TestCase):
    def test_manifest_schema_rejects_global_asset_importer_id_duplicates(self):
        violations: list[str] = []

        collect_global_manifest_identity_violations(
            [
                (
                    "zircon_plugins/texture_importer/plugin.toml",
                    plugin_manifest(
                        "texture_importer",
                        asset_importers=[asset_importer("texture.png")],
                    ),
                ),
                (
                    "zircon_plugins/asset_importers/texture/plugin.toml",
                    plugin_manifest(
                        "asset_importer.texture",
                        asset_importers=[asset_importer("texture.png")],
                    ),
                ),
            ],
            violations,
        )

        self.assertEqual(
            [
                "plugin validate asset_importers id texture.png is duplicated by "
                "plugin texture_importer asset_importers[0].id and plugin "
                "asset_importer.texture asset_importers[0].id",
            ],
            violations,
        )

    def test_manifest_schema_rejects_global_option_key_duplicates(self):
        violations: list[str] = []

        collect_global_manifest_identity_violations(
            [
                (
                    "zircon_plugins/rendering/plugin.toml",
                    plugin_manifest(
                        "rendering",
                        options=[plugin_option("render.debug")],
                    ),
                ),
                (
                    "zircon_plugins/texture/plugin.toml",
                    plugin_manifest(
                        "texture",
                        options=[plugin_option("render.debug")],
                    ),
                ),
            ],
            violations,
        )

        self.assertEqual(
            [
                "plugin validate options key render.debug is duplicated by "
                "plugin rendering options[0].key and plugin texture options[0].key",
            ],
            violations,
        )


def plugin_manifest(
    package_id: str,
    *,
    asset_importers: list[dict[str, object]] | None = None,
    options: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    manifest: dict[str, object] = {"id": package_id}
    if asset_importers is not None:
        manifest["asset_importers"] = asset_importers
    if options is not None:
        manifest["options"] = options
    return manifest


def asset_importer(importer_id: str) -> dict[str, object]:
    return {"id": importer_id}


def plugin_option(key: str) -> dict[str, object]:
    return {"key": key}


if __name__ == "__main__":
    unittest.main()
