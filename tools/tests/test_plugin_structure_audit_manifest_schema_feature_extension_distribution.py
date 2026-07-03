import unittest

from tools.tests.plugin_structure_audit_feature_extension_support import (
    collect_manifest_schema_violations,
    feature_distribution,
    plugin_feature_extension,
    plugin_manifest,
)


class PluginStructureAuditManifestSchemaFeatureExtensionDistributionTests(
    unittest.TestCase
):
    def test_manifest_schema_rejects_feature_extension_distribution_non_table(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                feature_extensions=[
                    plugin_feature_extension(distribution="native_dynamic"),
                ]
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].distribution must be a table"
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_extension_distribution_missing_abi(
        self,
    ):
        violations: list[str] = []
        distribution = feature_distribution()
        del distribution["abi_version"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                feature_extensions=[
                    plugin_feature_extension(distribution=distribution),
                ]
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "missing feature_extensions[0].distribution.abi_version"
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_extension_distribution_missing_entry(
        self,
    ):
        violations: list[str] = []
        distribution = feature_distribution()
        del distribution["runtime_entry"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                feature_extensions=[
                    plugin_feature_extension(distribution=distribution),
                ]
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].distribution must declare runtime_entry "
                "or editor_entry"
            ],
            violations,
        )


if __name__ == "__main__":
    unittest.main()
