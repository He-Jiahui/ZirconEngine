import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaOptionsTests(unittest.TestCase):
    def test_manifest_schema_rejects_malformed_option_row(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/options_plugin/plugin.toml",
            plugin_manifest(
                options=[
                    {
                        "key": "badkey",
                        "display_name": " Missing Gate ",
                        "value_type": "flag",
                        "default_value": "true",
                        "enum_values": ["on"],
                        "sidecar": "unexpected",
                    }
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/options_plugin/plugin.toml: "
                "options[0].sidecar is not a known option field",
                "zircon_plugins/options_plugin/plugin.toml: "
                "options[0].key must use at least two dot-separated "
                "namespace segments",
                "zircon_plugins/options_plugin/plugin.toml: "
                "options[0].display_name must be a non-empty trimmed string",
                "zircon_plugins/options_plugin/plugin.toml: "
                'options[0].value_type "flag" is unsupported; expected one of '
                "bool, integer, number, string, enum",
                "zircon_plugins/options_plugin/plugin.toml: "
                "options[0].enum_values must only be declared for enum options",
            ],
            violations,
        )

    def test_manifest_schema_rejects_enum_option_drift(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/options_plugin/plugin.toml",
            plugin_manifest(
                options=[
                    {
                        "key": "options_plugin.quality",
                        "display_name": "Quality",
                        "value_type": "enum",
                        "default_value": "high",
                        "enum_values": ["Low", "medium", "medium"],
                    }
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/options_plugin/plugin.toml: "
                "options[0].enum_values[0] must contain only lowercase ASCII "
                "letters, digits, underscores, or hyphens",
                "zircon_plugins/options_plugin/plugin.toml: "
                "options[0].enum_values[2] duplicates entry 1",
                "zircon_plugins/options_plugin/plugin.toml: "
                "options[0].default_value must be declared in enum_values",
            ],
            violations,
        )

    def test_manifest_schema_rejects_malformed_option_default_values(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/options_plugin/plugin.toml",
            plugin_manifest(
                options=[
                    option_row("options_plugin.enabled", "bool", "yes"),
                    option_row("options_plugin.count", "integer", "1.5"),
                    option_row("options_plugin.scale", "number", "nan"),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/options_plugin/plugin.toml: "
                "options[0].default_value bool value must be true or false",
                "zircon_plugins/options_plugin/plugin.toml: "
                "options[1].default_value integer value must parse as i64",
                "zircon_plugins/options_plugin/plugin.toml: "
                "options[2].default_value number value must be finite",
            ],
            violations,
        )


def plugin_manifest(
    *,
    options: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    manifest: dict[str, object] = {
        "id": "options_plugin",
        "version": "0.1.0",
        "sdk_api_version": "0.1.0",
        "display_name": "Options Plugin",
        "category": "runtime",
        "description": "Options plugin.",
        "supported_targets": ["client_runtime"],
        "supported_platforms": ["windows"],
        "capabilities": ["runtime.plugin.options_plugin"],
        "maturity": "stable",
        "default_packaging": ["source_template"],
        "modules": [
            {
                "name": "options_plugin.runtime",
                "kind": "runtime",
                "crate_name": "zircon_plugin_options_plugin_runtime",
                "target_modes": ["client_runtime"],
                "capabilities": ["runtime.plugin.options_plugin"],
            }
        ],
    }
    if options is not None:
        manifest["options"] = options
    return manifest


def option_row(key: str, value_type: str, default_value: str) -> dict[str, object]:
    return {
        "key": key,
        "display_name": "Option",
        "value_type": value_type,
        "default_value": default_value,
    }


if __name__ == "__main__":
    unittest.main()
