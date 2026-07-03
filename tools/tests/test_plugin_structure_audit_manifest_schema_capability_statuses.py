import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaCapabilityStatusesTests(unittest.TestCase):
    def test_manifest_schema_rejects_malformed_capability_status_row(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                capability_statuses=[
                    capability_status(
                        capability="Runtime.Plugin.Bad",
                        status="done",
                        note=" ",
                        sidecar="unexpected",
                    ),
                    {"status": "partial"},
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "capability_statuses[0].sidecar "
                "is not a known capability_status field",
                "zircon_plugins/sound/plugin.toml: "
                "capability_statuses[0].capability Runtime.Plugin.Bad "
                "should contain only lowercase ASCII letters, digits, "
                "underscores, and dots",
                "zircon_plugins/sound/plugin.toml: "
                "capability_statuses[0].capability Runtime.Plugin.Bad "
                "must reference a package or optional feature capability "
                "declared by the same package",
                "zircon_plugins/sound/plugin.toml: "
                "capability_statuses[0].status done should be one of complete, "
                "partial, stub, externalized, unsupported",
                "zircon_plugins/sound/plugin.toml: "
                "capability_statuses[0].note must be a non-empty trimmed string",
                "zircon_plugins/sound/plugin.toml: "
                "capability_statuses[1].capability is required",
            ],
            violations,
        )

    def test_manifest_schema_rejects_duplicate_capability_status_row(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                capability_statuses=[
                    capability_status(),
                    capability_status(status="complete"),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "capability_statuses[1].capability runtime.plugin.sound "
                "duplicates capability_status capability_statuses[0]"
            ],
            violations,
        )

    def test_manifest_schema_rejects_capability_status_target_modes(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                supported_targets=["client_runtime"],
                capability_statuses=[
                    capability_status(
                        target_modes=[
                            "client_runtime",
                            "desktop",
                            "client_runtime",
                            "server_runtime",
                        ]
                    )
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                'zircon_plugins/sound/plugin.toml: '
                'capability_statuses[0].target_modes[1] "desktop" '
                "is unsupported; expected one of client_runtime, server_runtime, "
                "editor_host",
                "zircon_plugins/sound/plugin.toml: "
                "capability_statuses[0].target_modes[2] client_runtime "
                "duplicates capability_status target_modes target_modes[0]",
                "zircon_plugins/sound/plugin.toml: "
                "capability_statuses[0].target_modes[3] server_runtime "
                "should be covered by package supported_targets",
            ],
            violations,
        )

    def test_manifest_schema_rejects_capability_status_bevy_references(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                capability_statuses=[
                    capability_status(
                        bevy_references=[
                            "dev/bevy/crates/bevy_app/src/app.rs",
                            "bevy/crates/bevy_ecs/src/world/mod.rs",
                            "dev/bevy/../bad.rs",
                            "dev\\bevy\\bad.rs",
                            "dev/bevy/crates/bevy_app/src/app.rs",
                        ]
                    )
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "capability_statuses[0].bevy_references[1] "
                "bevy/crates/bevy_ecs/src/world/mod.rs should start with dev/bevy/",
                "zircon_plugins/sound/plugin.toml: "
                "capability_statuses[0].bevy_references[2] "
                "dev/bevy/../bad.rs should not contain empty, current, or "
                "parent path segments",
                "zircon_plugins/sound/plugin.toml: "
                "capability_statuses[0].bevy_references[3] dev\\bevy\\bad.rs "
                "should use repository-relative forward-slash paths",
                "zircon_plugins/sound/plugin.toml: "
                "capability_statuses[0].bevy_references[4] "
                "dev/bevy/crates/bevy_app/src/app.rs duplicates capability_status "
                "bevy_references bevy_references[0]",
            ],
            violations,
        )


def plugin_manifest(
    *,
    supported_targets: list[str] | None = None,
    capability_statuses: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    return {
        "id": "sound",
        "version": "0.1.0",
        "sdk_api_version": "0.1.0",
        "display_name": "Sound",
        "category": "runtime",
        "description": "Sound plugin.",
        "supported_targets": supported_targets or ["client_runtime", "editor_host"],
        "supported_platforms": ["windows"],
        "capabilities": [
            "runtime.plugin.sound",
            "runtime.capability.sound.mixer",
        ],
        "maturity": "stable",
        "default_packaging": ["source_template", "library_embed", "native_dynamic"],
        "capability_statuses": capability_statuses or [capability_status()],
        "modules": [
            {
                "name": "sound.runtime",
                "kind": "runtime",
                "crate_name": "zircon_plugin_sound_runtime",
                "target_modes": ["client_runtime"],
                "capabilities": [
                    "runtime.plugin.sound",
                    "runtime.capability.sound.mixer",
                ],
            }
        ],
    }


def capability_status(
    *,
    capability: str = "runtime.plugin.sound",
    status: str = "partial",
    note: str | None = None,
    target_modes: list[str] | None = None,
    bevy_references: list[str] | None = None,
    sidecar: str | None = None,
) -> dict[str, object]:
    row: dict[str, object] = {
        "capability": capability,
        "status": status,
    }
    if note is not None:
        row["note"] = note
    if target_modes is not None:
        row["target_modes"] = target_modes
    if bevy_references is not None:
        row["bevy_references"] = bevy_references
    if sidecar is not None:
        row["sidecar"] = sidecar
    return row


if __name__ == "__main__":
    unittest.main()
