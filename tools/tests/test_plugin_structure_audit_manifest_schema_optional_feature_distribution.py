import unittest

from tools.tests.plugin_structure_audit_optional_feature_support import (
    collect_manifest_schema_violations,
    feature_distribution,
    plugin_feature,
    plugin_manifest,
)


class PluginStructureAuditManifestSchemaOptionalFeatureDistributionTests(
    unittest.TestCase
):
    def test_manifest_schema_rejects_optional_feature_distribution_non_table(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[plugin_feature(distribution="dist")],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: optional_features[0].distribution must be a table"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_distribution_missing_abi(self):
        violations: list[str] = []
        distribution = feature_distribution()
        del distribution["abi_version"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[plugin_feature(distribution=distribution)],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: missing optional_features[0].distribution.abi_version"
            ],
            violations,
        )

    def test_manifest_schema_rejects_unknown_optional_feature_distribution_form(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(
                        distribution=feature_distribution(forms=["dist", "zip"]),
                    ),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                'zircon_plugins/sound/plugin.toml: optional_features[0].distribution.forms[1] "zip" is unsupported; expected one of dist, embed'
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_distribution_missing_entry(
        self,
    ):
        violations: list[str] = []
        distribution = feature_distribution()
        del distribution["runtime_entry"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[plugin_feature(distribution=distribution)],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: optional_features[0].distribution must declare runtime_entry or editor_entry"
            ],
            violations,
        )


if __name__ == "__main__":
    unittest.main()
