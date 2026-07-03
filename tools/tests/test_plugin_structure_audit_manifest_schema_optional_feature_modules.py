import unittest

from tools.tests.plugin_structure_audit_optional_feature_support import (
    collect_manifest_schema_violations,
    feature_module,
    plugin_feature,
    plugin_manifest,
)


class PluginStructureAuditManifestSchemaOptionalFeatureModulesTests(unittest.TestCase):
    def test_manifest_schema_rejects_optional_feature_module_missing_field(self):
        violations: list[str] = []
        optional_module = feature_module()
        del optional_module["crate_name"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(optional_modules=[optional_module]),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: missing optional_features[0].modules[0].crate_name"
            ],
            violations,
        )

    def test_manifest_schema_rejects_unknown_optional_feature_module_kind(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(optional_modules=[feature_module(kind="sidecar")]),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                'zircon_plugins/sound/plugin.toml: optional_features[0].modules[0].kind "sidecar" is unsupported; expected one of runtime, editor, native, vm'
            ],
            violations,
        )

    def test_manifest_schema_rejects_unknown_optional_feature_module_target_mode(
        self,
    ):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(
                        optional_modules=[
                            feature_module(target_modes=["client_runtime", "desktop"]),
                        ],
                    ),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                'zircon_plugins/sound/plugin.toml: optional_features[0].modules[0].target_modes[1] "desktop" is unsupported; expected one of client_runtime, server_runtime, editor_host'
            ],
            violations,
        )


if __name__ == "__main__":
    unittest.main()
