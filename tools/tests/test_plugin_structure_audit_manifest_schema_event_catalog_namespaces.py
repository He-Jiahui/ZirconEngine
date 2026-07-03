import unittest

from tools.plugin_structure_audits.manifest_schema_event_catalog_namespaces import (
    collect_global_event_catalog_namespace_violations,
)


class PluginStructureAuditManifestSchemaEventCatalogNamespacesTests(unittest.TestCase):
    def test_manifest_schema_rejects_global_event_catalog_namespace_duplicates(self):
        violations: list[str] = []

        collect_global_event_catalog_namespace_violations(
            [
                (
                    "zircon_plugins/audio_events/plugin.toml",
                    plugin_manifest(
                        "audio_events",
                        event_catalogs=[event_catalog("audio_events.events")],
                    ),
                ),
                (
                    "zircon_plugins/audio_diagnostics/plugin.toml",
                    plugin_manifest(
                        "audio_diagnostics",
                        event_catalogs=[event_catalog("audio_events.events")],
                    ),
                ),
            ],
            violations,
        )

        self.assertEqual(
            [
                "plugin validate event_catalog namespace audio_events.events "
                "is duplicated by plugin audio_events event_catalogs[0].namespace "
                "and plugin audio_diagnostics event_catalogs[0].namespace",
            ],
            violations,
        )


def plugin_manifest(
    package_id: str,
    *,
    event_catalogs: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    manifest: dict[str, object] = {"id": package_id}
    if event_catalogs is not None:
        manifest["event_catalogs"] = event_catalogs
    return manifest


def event_catalog(namespace: str) -> dict[str, object]:
    return {"namespace": namespace}


if __name__ == "__main__":
    unittest.main()
