import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaModulesTests(unittest.TestCase):
    def test_manifest_schema_accepts_module_descriptor_projection_fields(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["modules"][0]["description"] = "Sound runtime module"
        manifest["modules"][0]["init_level"] = "scene"
        manifest["modules"][0]["module_dependencies"] = [
            {"module_name": "foundation.runtime"}
        ]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual([], violations)

    def test_manifest_schema_accepts_services_and_rejects_retired_servers_level(self):
        services_violations: list[str] = []
        services_manifest = plugin_manifest()
        services_manifest["modules"][0]["init_level"] = "services"
        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            services_manifest,
            services_violations,
        )

        retired_violations: list[str] = []
        retired_manifest = plugin_manifest()
        retired_manifest["modules"][0]["init_level"] = "servers"
        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            retired_manifest,
            retired_violations,
        )

        self.assertEqual([], services_violations)
        self.assertEqual(
            [
                'zircon_plugins/sound/plugin.toml: modules[0].init_level "servers" is unsupported; expected one of kernel, services, scene, editor, post'
            ],
            retired_violations,
        )

    def test_manifest_schema_rejects_module_descriptor_projection_drift(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["modules"][0]["description"] = " Sound runtime "
        manifest["modules"][0]["init_level"] = "startup"
        manifest["modules"][0]["module_dependencies"] = [
            {"module_name": "foundation.runtime", "phase": "before"},
            {"module_name": "foundation.runtime"},
            {"module_name": "Runtime"},
        ]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: modules[0].description must be a non-empty trimmed string when declared",
                'zircon_plugins/sound/plugin.toml: modules[0].init_level "startup" is unsupported; expected one of kernel, services, scene, editor, post',
                "zircon_plugins/sound/plugin.toml: modules[0].module_dependencies[0].phase is not a known module dependency field",
                "zircon_plugins/sound/plugin.toml: modules[0].module_dependencies[1].module_name foundation.runtime duplicates module_dependencies[0]",
                "zircon_plugins/sound/plugin.toml: modules[0].module_dependencies[2].module_name Runtime should use package.module dot namespace form",
                "zircon_plugins/sound/plugin.toml: modules[0].module_dependencies[2].module_name Runtime should contain only lowercase ASCII letters, digits, underscores, and dots",
            ],
            violations,
        )

    def test_manifest_schema_rejects_module_identity_and_crate_drift(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["modules"][0] = {
            "name": "Sound..runtime",
            "kind": "runtime",
            "crate_name": "zircon_plugin_sound__runtime_",
            "target_modes": ["client_runtime"],
            "capabilities": ["runtime.plugin.sound"],
            "legacy_module": True,
        }

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "modules[0].legacy_module is not a known module field",
                "zircon_plugins/sound/plugin.toml: "
                "modules[0].name Sound..runtime should not contain empty namespace segments",
                "zircon_plugins/sound/plugin.toml: "
                "modules[0].name Sound..runtime should contain only lowercase ASCII "
                "letters, digits, underscores, and dots",
                "zircon_plugins/sound/plugin.toml: "
                "modules[0].name Sound..runtime should stay under namespace sound.",
                "zircon_plugins/sound/plugin.toml: "
                "modules[0].crate_name zircon_plugin_sound__runtime_ should not end "
                "with an underscore or contain repeated underscores",
            ],
            violations,
        )

    def test_manifest_schema_rejects_module_target_capability_and_system_contracts(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["modules"][0] = {
            "name": "sound.editor",
            "kind": "editor",
            "crate_name": "zircon_plugin_sound_editor",
            "target_modes": ["client_runtime", "server_runtime"],
            "capabilities": ["runtime.plugin.sound"],
            "system_sets": ["sound.editor.update"],
            "system_anchors": ["foreign.anchor", "foreign.anchor"],
        }

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "modules[0] is an editor module and should only target editor_host, "
                "got client_runtime",
                "zircon_plugins/sound/plugin.toml: "
                "modules[0].target_modes[1] server_runtime should be covered by "
                "package supported_targets",
                "zircon_plugins/sound/plugin.toml: "
                "modules[0] is an editor module and should only target editor_host, "
                "got server_runtime",
                "zircon_plugins/sound/plugin.toml: "
                "modules[0].capabilities[0] runtime.plugin.sound should start with editor.",
                "zircon_plugins/sound/plugin.toml: "
                "modules[0].system_sets may only be declared by runtime modules",
                "zircon_plugins/sound/plugin.toml: "
                "modules[0].system_anchors may only be declared by runtime modules",
                "zircon_plugins/sound/plugin.toml: "
                "modules[0].system_anchors[0] foreign.anchor should stay under namespace sound.",
                "zircon_plugins/sound/plugin.toml: "
                "modules[0].system_anchors[1] foreign.anchor should stay under namespace sound.",
                "zircon_plugins/sound/plugin.toml: "
                "modules[0].system_anchors[1] foreign.anchor duplicates system_anchors[0]",
            ],
            violations,
        )

    def test_manifest_schema_rejects_duplicate_module_names_across_feature_rows(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["optional_features"] = [
            plugin_feature(optional_modules=[feature_module(name="sound.runtime")]),
        ]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].modules[0].name sound.runtime "
                "should stay under namespace sound.preview.",
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].modules[0].name sound.runtime "
                "duplicates module name modules[0]",
            ],
            violations,
        )


    def test_manifest_schema_accepts_typed_editor_event_consumer(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["supported_targets"] = ["client_runtime", "editor_host"]
        manifest["modules"].append(
            {
                "name": "sound.editor",
                "kind": "editor",
                "crate_name": "zircon_plugin_sound_editor",
                "target_modes": ["editor_host"],
                "capabilities": ["editor.extension.sound"],
                "event_consumers": [
                    {
                        "consumer_id": "sound.editor.meter",
                        "event_id": "sound.events.meter",
                        "payload_schema": "sound.events.meter.v1",
                        "required_capability": "editor.extension.sound",
                    }
                ],
            }
        )

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml", manifest, violations
        )

        self.assertEqual([], violations)


def plugin_manifest() -> dict[str, object]:
    return {
        "id": "sound",
        "version": "0.1.0",
        "sdk_api_version": "0.1.0",
        "display_name": "Sound",
        "category": "runtime",
        "description": "Sound plugin.",
        "supported_targets": ["client_runtime"],
        "supported_platforms": ["windows"],
        "capabilities": ["runtime.plugin.sound"],
        "maturity": "experimental",
        "default_packaging": ["source_template", "library_embed", "native_dynamic"],
        "modules": [
            {
                "name": "sound.runtime",
                "kind": "runtime",
                "crate_name": "zircon_plugin_sound_runtime",
                "target_modes": ["client_runtime"],
                "capabilities": ["runtime.plugin.sound"],
            }
        ],
    }


def plugin_feature(
    *,
    optional_modules: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    feature: dict[str, object] = {
        "id": "sound.preview",
        "display_name": "Preview Sound",
        "owner_plugin_id": "sound",
        "capabilities": ["runtime.feature.sound.preview"],
        "default_packaging": ["source_template"],
        "enabled_by_default": False,
        "dependencies": [
            {
                "plugin_id": "sound",
                "capability": "runtime.plugin.sound",
                "primary": True,
            }
        ],
    }
    if optional_modules is not None:
        feature["modules"] = optional_modules
    return feature


def feature_module(*, name: str = "sound.preview.runtime") -> dict[str, object]:
    return {
        "name": name,
        "kind": "runtime",
        "crate_name": "zircon_plugin_sound_preview_runtime",
        "target_modes": ["client_runtime"],
        "capabilities": ["runtime.feature.sound.preview"],
    }


if __name__ == "__main__":
    unittest.main()
