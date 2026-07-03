import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaEventCatalogsTests(unittest.TestCase):
    def test_manifest_schema_rejects_malformed_event_catalog_row(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/audio_events/plugin.toml",
            plugin_manifest(
                event_catalogs=[
                    {
                        "namespace": "Audio.Events",
                        "version": "1",
                        "events": [
                            {
                                "id": "audio_events.other.started",
                                "display_name": " Started ",
                                "payload_schema": "external.started.v01",
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
                "zircon_plugins/audio_events/plugin.toml: "
                "event_catalogs[0].sidecar is not a known event catalog field",
                "zircon_plugins/audio_events/plugin.toml: "
                "event_catalogs[0].namespace Audio.Events should contain only "
                "lowercase ASCII letters, digits, underscores, and dots",
                "zircon_plugins/audio_events/plugin.toml: "
                "event_catalogs[0].namespace Audio.Events should stay under "
                "package namespace audio_events.",
                "zircon_plugins/audio_events/plugin.toml: "
                "event_catalogs[0].version must be an integer",
                "zircon_plugins/audio_events/plugin.toml: "
                "event_catalogs[0].events[0].sidecar is not a known event field",
                "zircon_plugins/audio_events/plugin.toml: "
                "event_catalogs[0].events[0].id audio_events.other.started "
                "should stay under namespace Audio.Events.",
                "zircon_plugins/audio_events/plugin.toml: "
                "event_catalogs[0].events[0].display_name must be a "
                "non-empty trimmed string",
                "zircon_plugins/audio_events/plugin.toml: "
                "event_catalogs[0].events[0].payload_schema external.started.v01 "
                "should stay under package namespace audio_events.",
                "zircon_plugins/audio_events/plugin.toml: "
                "event_catalogs[0].events[0].payload_schema external.started.v01 "
                "version segment should be a positive integer without leading "
                "zeroes",
            ],
            violations,
        )

    def test_manifest_schema_rejects_duplicate_event_catalog_namespace(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/audio_events/plugin.toml",
            plugin_manifest(
                event_catalogs=[
                    event_catalog_row(),
                    event_catalog_row(),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/audio_events/plugin.toml: "
                "event_catalogs[1].namespace audio_events.events duplicates "
                "event catalog namespace row 0"
            ],
            violations,
        )

    def test_manifest_schema_rejects_event_payload_schema_version(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/audio_events/plugin.toml",
            plugin_manifest(
                event_catalogs=[
                    {
                        "namespace": "audio_events.events",
                        "version": 0x1_0000_0000,
                        "events": [
                            {
                                "id": "audio_events.events.started",
                                "display_name": "Started",
                                "payload_schema": "audio_events.started.payload",
                            },
                            {
                                "id": "audio_events.events.started",
                                "display_name": "Started Again",
                                "payload_schema": "audio_events.started.vx",
                            },
                        ],
                    }
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/audio_events/plugin.toml: "
                "event_catalogs[0].version 4294967296 should be a positive u32",
                "zircon_plugins/audio_events/plugin.toml: "
                "event_catalogs[0].events[0].payload_schema "
                "audio_events.started.payload should end with a version "
                "segment like v1",
                "zircon_plugins/audio_events/plugin.toml: "
                "event_catalogs[0].events[1].id duplicates event row 0",
                "zircon_plugins/audio_events/plugin.toml: "
                "event_catalogs[0].events[1].payload_schema "
                "audio_events.started.vx version segment should contain only "
                "digits after v",
            ],
            violations,
        )


def plugin_manifest(
    *,
    event_catalogs: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    manifest: dict[str, object] = {
        "id": "audio_events",
        "version": "0.1.0",
        "sdk_api_version": "0.1.0",
        "display_name": "Audio Events",
        "category": "runtime",
        "description": "Audio event catalog.",
        "supported_targets": ["client_runtime"],
        "supported_platforms": ["windows"],
        "capabilities": ["runtime.plugin.audio_events"],
        "maturity": "stable",
        "default_packaging": ["source_template"],
        "modules": [
            {
                "name": "audio_events.runtime",
                "kind": "runtime",
                "crate_name": "zircon_plugin_audio_events_runtime",
                "target_modes": ["client_runtime"],
                "capabilities": ["runtime.plugin.audio_events"],
            }
        ],
    }
    if event_catalogs is not None:
        manifest["event_catalogs"] = event_catalogs
    return manifest


def event_catalog_row() -> dict[str, object]:
    return {
        "namespace": "audio_events.events",
        "version": 1,
        "events": [
            {
                "id": "audio_events.events.started",
                "display_name": "Started",
                "payload_schema": "audio_events.started.v1",
            }
        ],
    }


if __name__ == "__main__":
    unittest.main()
