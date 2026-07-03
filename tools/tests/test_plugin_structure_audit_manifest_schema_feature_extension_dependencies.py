import unittest

from tools.tests.plugin_structure_audit_feature_extension_support import (
    collect_manifest_schema_violations,
    feature_dependency,
    plugin_feature_extension,
    plugin_manifest,
)


class PluginStructureAuditManifestSchemaFeatureExtensionDependencyTests(
    unittest.TestCase
):
    def test_manifest_schema_rejects_feature_extension_missing_dependencies(self):
        violations: list[str] = []
        feature_extension = plugin_feature_extension()
        del feature_extension["dependencies"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(feature_extensions=[feature_extension]),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "missing feature_extensions[0].dependencies"
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_extension_empty_dependencies(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                feature_extensions=[
                    plugin_feature_extension(dependencies=[]),
                ]
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: feature_extensions[0].dependencies "
                "should declare at least one dependency"
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_extension_dependency_non_table(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                feature_extensions=[
                    plugin_feature_extension(dependencies=["sound"]),
                ]
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].dependencies[0] must be a table"
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_extension_dependency_missing_plugin_id(
        self,
    ):
        violations: list[str] = []
        dependency = feature_dependency()
        del dependency["plugin_id"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                feature_extensions=[
                    plugin_feature_extension(dependencies=[dependency]),
                ]
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "missing feature_extensions[0].dependencies[0].plugin_id"
            ],
            violations,
        )

    def test_manifest_schema_rejects_unknown_feature_extension_dependency_field(self):
        violations: list[str] = []
        dependency = feature_dependency()
        dependency["sidecar"] = "legacy"

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                feature_extensions=[
                    plugin_feature_extension(dependencies=[dependency]),
                ]
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].dependencies[0].sidecar "
                "is not a known optional feature dependency field"
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_extension_without_primary_dependency(
        self,
    ):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                feature_extensions=[
                    plugin_feature_extension(
                        dependencies=[feature_dependency(primary=False)]
                    ),
                ]
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: feature_extensions[0].dependencies "
                "should declare exactly one primary dependency"
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_extension_multiple_primary_dependencies(
        self,
    ):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                feature_extensions=[
                    plugin_feature_extension(
                        dependencies=[
                            feature_dependency(),
                            feature_dependency(
                                capability="runtime.plugin.sound.secondary"
                            ),
                        ],
                    ),
                ]
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: feature_extensions[0].dependencies "
                "should declare exactly one primary dependency"
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_extension_duplicate_dependency_rows(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                feature_extensions=[
                    plugin_feature_extension(
                        dependencies=[
                            feature_dependency(),
                            feature_dependency(primary=False),
                        ],
                    ),
                ]
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].dependencies[1] duplicates dependency row 0"
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_extension_primary_owner_mismatch(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                feature_extensions=[
                    plugin_feature_extension(
                        dependencies=[feature_dependency(plugin_id="physics")],
                    ),
                ]
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].dependencies[0] "
                "primary dependency plugin_id must match owner plugin id sound"
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_extension_primary_capability_mismatch(
        self,
    ):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                feature_extensions=[
                    plugin_feature_extension(
                        dependencies=[
                            feature_dependency(
                                capability="runtime.plugin.sound.secondary"
                            ),
                        ],
                    ),
                ]
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].dependencies[0] "
                "primary dependency capability must be an owner plugin capability"
            ],
            violations,
        )


if __name__ == "__main__":
    unittest.main()
