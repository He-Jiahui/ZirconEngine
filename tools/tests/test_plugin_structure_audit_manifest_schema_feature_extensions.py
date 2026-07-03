import unittest

from tools.tests.plugin_structure_audit_feature_extension_support import (
    collect_manifest_schema_violations,
    plugin_feature_extension,
    plugin_manifest,
)


class PluginStructureAuditManifestSchemaFeatureExtensionsTests(unittest.TestCase):
    def test_manifest_schema_rejects_unknown_feature_extension_field(self):
        violations: list[str] = []
        feature_extension = plugin_feature_extension()
        feature_extension["sidecar"] = "legacy"

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(feature_extensions=[feature_extension]),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].sidecar is not a known feature extension field"
            ],
            violations,
        )

    def test_manifest_schema_rejects_empty_feature_extensions_array(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(feature_extensions=[]),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: feature_extensions "
                "must not be empty when declared"
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_extension_non_table(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(feature_extensions=["sound.preview"]),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: feature_extensions[0] must be a table"
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_extension_missing_required_field(self):
        violations: list[str] = []
        feature_extension = plugin_feature_extension()
        del feature_extension["owner_plugin_id"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(feature_extensions=[feature_extension]),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: missing feature_extensions[0].owner_plugin_id"
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_extension_identity_semantics(self):
        violations: list[str] = []
        feature_extension = plugin_feature_extension(
            feature_id="Native.Dynamic",
            owner_plugin_id="1Sound__",
        )
        feature_extension["dependencies"] = [
            {
                "plugin_id": "1Sound__",
                "capability": "runtime.plugin.sound",
                "primary": True,
            }
        ]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(feature_extensions=[feature_extension]),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].owner_plugin_id 1Sound__ "
                "should start with a lowercase ASCII letter",
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].owner_plugin_id 1Sound__ "
                "should contain only lowercase ASCII letters, digits, and underscores",
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].owner_plugin_id 1Sound__ "
                "should not end with an underscore or contain repeated underscores",
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].id Native.Dynamic should contain only "
                "lowercase ASCII letters, digits, underscores, and dots",
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].id Native.Dynamic "
                "should stay under owner namespace 1Sound__.",
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_extension_id_namespace_segments(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                feature_extensions=[
                    plugin_feature_extension(feature_id="sound..preview"),
                ]
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].id sound..preview "
                "should not contain empty namespace segments"
            ],
            violations,
        )

    def test_manifest_schema_rejects_duplicate_feature_extension_id(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                feature_extensions=[
                    plugin_feature_extension(),
                    plugin_feature_extension(),
                ]
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[1].id sound.preview "
                "duplicates feature extension id feature_extensions[0]"
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_extension_capability_semantics(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                feature_extensions=[
                    plugin_feature_extension(
                        capabilities=[
                            "badcap",
                            "runtime..feature",
                            "runtime.feature.sound.preview",
                            "runtime.feature.sound.preview",
                        ],
                    )
                ]
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].capabilities[0] badcap "
                "should use at least two dot-separated namespace segments",
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].capabilities[1] runtime..feature "
                "should not contain empty namespace segments",
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].capabilities[3] "
                "runtime.feature.sound.preview duplicates capabilities "
                "capabilities[2]",
            ],
            violations,
        )


if __name__ == "__main__":
    unittest.main()
