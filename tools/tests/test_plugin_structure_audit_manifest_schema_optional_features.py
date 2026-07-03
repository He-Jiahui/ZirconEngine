import unittest

from tools.tests.plugin_structure_audit_optional_feature_support import (
    collect_manifest_schema_violations,
    plugin_feature,
    plugin_manifest,
)


class PluginStructureAuditManifestSchemaOptionalFeaturesTests(unittest.TestCase):
    def test_manifest_schema_rejects_empty_optional_features_array(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(optional_features=[]),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: optional_features "
                "must not be empty when declared"
            ],
            violations,
        )

    def test_manifest_schema_rejects_unknown_optional_feature_field(self):
        violations: list[str] = []
        feature = plugin_feature()
        feature["sidecar"] = "legacy"

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(optional_features=[feature]),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].sidecar is not a known optional feature field"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_identity_semantics(self):
        violations: list[str] = []
        manifest = plugin_manifest(
            optional_features=[
                plugin_feature(
                    feature_id="Native.Dynamic",
                    owner_plugin_id="other_plugin",
                )
            ],
        )

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].id Native.Dynamic should contain only "
                "lowercase ASCII letters, digits, underscores, and dots",
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].id Native.Dynamic "
                "should stay under package namespace sound.",
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].owner_plugin_id other_plugin "
                "should match package id sound",
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_id_namespace_segments(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[plugin_feature(feature_id="sound..preview")],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].id sound..preview "
                "should not contain empty namespace segments"
            ],
            violations,
        )

    def test_manifest_schema_rejects_duplicate_optional_feature_id(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(optional_features=[plugin_feature(), plugin_feature()]),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[1].id sound.preview "
                "duplicates optional feature id optional_features[0]"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_capability_semantics(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(
                        capabilities=[
                            "badcap",
                            "runtime..feature",
                            "runtime.feature.sound.preview",
                            "runtime.feature.sound.preview",
                        ],
                    )
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].capabilities[0] badcap "
                "should use at least two dot-separated namespace segments",
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].capabilities[1] runtime..feature "
                "should not contain empty namespace segments",
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].capabilities[3] "
                "runtime.feature.sound.preview duplicates capabilities "
                "capabilities[2]",
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_missing_owner(self):
        violations: list[str] = []
        feature = plugin_feature()
        del feature["owner_plugin_id"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(optional_features=[feature]),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "missing optional_features[0].owner_plugin_id"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_enabled_by_default_type(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[plugin_feature(enabled_by_default="false")],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].enabled_by_default must be a bool"
            ],
            violations,
        )

    def test_manifest_schema_rejects_unknown_optional_feature_default_packaging(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(default_packaging=["source_template", "sidecar"]),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                'zircon_plugins/sound/plugin.toml: optional_features[0].default_packaging[1] "sidecar" is unsupported; expected one of source_template, library_embed, native_dynamic'
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_provider_package_id_type(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[plugin_feature(provider_package_id=42)],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: optional_features[0].provider_package_id must be a non-empty trimmed string"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_provider_package_id_untrimmed(
        self,
    ):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(provider_package_id=" sound_provider "),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: optional_features[0].provider_package_id must be a non-empty trimmed string"
            ],
            violations,
        )


if __name__ == "__main__":
    unittest.main()
