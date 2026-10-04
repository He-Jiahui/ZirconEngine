import unittest

from tools.tests.plugin_structure_audit_feature_extension_support import (
    collect_manifest_schema_violations,
    feature_module,
    plugin_feature_extension,
    plugin_manifest,
)

# 用插件清单夹具验证特性扩展模块：拒绝特性扩展空模块，并拒绝特性扩展模块缺失字段。
class PluginStructureAuditManifestSchemaFeatureExtensionModuleTests(unittest.TestCase):
    def test_manifest_schema_rejects_feature_extension_empty_modules(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                feature_extensions=[
                    plugin_feature_extension(modules=[]),
                ]
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: feature_extensions[0].modules "
                "must not be empty when declared"
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_extension_module_missing_field(self):
        violations: list[str] = []
        module = feature_module()
        del module["crate_name"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                feature_extensions=[
                    plugin_feature_extension(modules=[module]),
                ]
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "missing feature_extensions[0].modules[0].crate_name"
            ],
            violations,
        )


if __name__ == "__main__":
    unittest.main()
