import unittest

from tools.tests.plugin_structure_audit_optional_feature_support import (
    collect_manifest_schema_violations,
    feature_dependency,
    plugin_feature,
    plugin_manifest,
)


class PluginStructureAuditManifestSchemaOptionalFeatureDependenciesTests(
    unittest.TestCase
):
    def test_manifest_schema_rejects_optional_feature_dependency_non_table(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(dependencies=["runtime.plugin.sound"]),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: optional_features[0].dependencies[0] must be a table"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_missing_dependencies(self):
        violations: list[str] = []
        feature = plugin_feature()
        del feature["dependencies"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(optional_features=[feature]),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "missing optional_features[0].dependencies"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_empty_dependencies(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[plugin_feature(dependencies=[])],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].dependencies should declare at least one dependency"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_dependency_missing_plugin_id(
        self,
    ):
        violations: list[str] = []
        dependency = feature_dependency()
        del dependency["plugin_id"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[plugin_feature(dependencies=[dependency])],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: missing optional_features[0].dependencies[0].plugin_id"
            ],
            violations,
        )

    def test_manifest_schema_rejects_unknown_optional_feature_dependency_field(self):
        violations: list[str] = []
        dependency = feature_dependency()
        dependency["sidecar"] = "legacy"

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[plugin_feature(dependencies=[dependency])],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].dependencies[0].sidecar "
                "is not a known optional feature dependency field"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_dependency_missing_primary(self):
        violations: list[str] = []
        dependency = feature_dependency()
        del dependency["primary"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[plugin_feature(dependencies=[dependency])],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: missing optional_features[0].dependencies[0].primary"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_dependency_primary_type(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(dependencies=[feature_dependency(primary="true")]),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: optional_features[0].dependencies[0].primary must be a bool"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_without_primary_dependency(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(dependencies=[feature_dependency(primary=False)]),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: optional_features[0].dependencies "
                "should declare exactly one primary dependency"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_multiple_primary_dependencies(
        self,
    ):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(
                        dependencies=[
                            feature_dependency(),
                            feature_dependency(
                                capability="runtime.plugin.sound.secondary",
                            ),
                        ],
                    ),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: optional_features[0].dependencies "
                "should declare exactly one primary dependency"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_duplicate_dependency_rows(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(
                        dependencies=[
                            feature_dependency(),
                            feature_dependency(primary=False),
                        ],
                    ),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].dependencies[1] duplicates dependency row 0"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_primary_plugin_mismatch(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(
                        dependencies=[feature_dependency(plugin_id="physics")],
                    ),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].dependencies[0] "
                "primary dependency plugin_id must match package id sound"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_primary_capability_mismatch(
        self,
    ):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(
                        dependencies=[
                            feature_dependency(
                                capability="runtime.plugin.sound.secondary",
                            ),
                        ],
                    ),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].dependencies[0] "
                "primary dependency capability must be a package capability"
            ],
            violations,
        )


if __name__ == "__main__":
    unittest.main()
