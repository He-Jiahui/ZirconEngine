import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_FEATURE_PROVIDER = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider_extension.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_MODULES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider_modules.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_MODULE_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/plugin_validate_feature_provider_module_schema.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_MODULE_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_plugin_validate_feature_provider_module_schema.py"
)


class PluginValidateFeatureProviderModuleOwnerBoundaryTests(unittest.TestCase):
    def test_feature_provider_modules_lives_in_modules_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_FEATURE_PROVIDER_MODULES.exists(),
            "feature-provider module projection belongs in plugin_validate_feature_provider_modules.py",
        )
        parent_text = PLUGIN_VALIDATE_FEATURE_PROVIDER.read_text(encoding="utf-8")
        extension_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION.read_text(
            encoding="utf-8"
        )
        modules_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_MODULES.read_text(
            encoding="utf-8"
        )
        test_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_MODULE_SCHEMA_TEST.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "PLUGIN_VALIDATE_FEATURE_PROVIDER_MODULE_FIELDS",
            "validate_plugin_feature_provider_modules",
            "plugin_validate_feature_provider_single_module",
            "plugin_validate_feature_provider_expected_runtime_module",
            "must equal generated distribution.dist_crate",
            "is not a known feature provider module field",
        ):
            self.assertIn(symbol, modules_text)
            self.assertNotIn(symbol, parent_text)
        self.assertIn(
            "from .plugin_validate_feature_provider_modules import",
            extension_text,
            "extension owner should dispatch generated module projection to the module leaf",
        )
        self.assertNotIn(
            "from .plugin_validate_feature_provider_modules import",
            parent_text,
            "feature-provider parent should dispatch modules through the extension owner",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_feature_provider import",
        ):
            self.assertNotIn(
                forbidden_import,
                modules_text,
                "feature-provider module owner must not borrow build, validate entry, or parent owners",
            )
        self.assertIn(
            "def test_plugin_validate_rejects_generated_feature_provider_module_projection_drift(",
            test_text,
        )
        self.assertLessEqual(
            len(modules_text.splitlines()),
            160,
            "feature-provider module projection owner should stay focused",
        )
        self.assertLessEqual(
            len(extension_text.splitlines()),
            130,
            "feature-provider extension owner should stay below its split budget",
        )

    def test_feature_provider_module_schema_stays_in_schema_leaf(self):
        self.assertTrue(
            PLUGIN_VALIDATE_FEATURE_PROVIDER_MODULE_SCHEMA.exists(),
            "feature-provider generated module schema checks belong in a focused leaf",
        )
        parent_text = PLUGIN_VALIDATE_FEATURE_PROVIDER.read_text(encoding="utf-8")
        extension_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION.read_text(
            encoding="utf-8"
        )
        modules_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_MODULES.read_text(
            encoding="utf-8"
        )
        schema_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_MODULE_SCHEMA.read_text(
            encoding="utf-8"
        )
        test_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_MODULE_SCHEMA_TEST.read_text(
            encoding="utf-8"
        )

        self.assertIn(
            "def validate_plugin_feature_provider_module_schema(",
            schema_text,
        )
        for root_helper in (
            "validate_plugin_module_name",
            "validate_plugin_module_kind",
            "validate_plugin_module_crate_name",
            "validate_plugin_module_target_modes",
            "validate_plugin_module_capabilities",
            "PLUGIN_VALIDATE_TARGET_MODES",
        ):
            self.assertIn(root_helper, schema_text)
            self.assertNotIn(root_helper, parent_text)
        self.assertIn(
            "from .plugin_validate_feature_provider_module_schema import",
            modules_text,
        )
        self.assertNotIn(
            "from .plugin_validate_feature_provider_module_schema import",
            extension_text,
        )
        self.assertNotIn(
            "from .plugin_validate_feature_provider_module_schema import",
            parent_text,
        )
        self.assertIn(
            "test_plugin_validate_rejects_generated_feature_provider_module_schema_drift",
            test_text,
        )
        self.assertLessEqual(len(modules_text.splitlines()), 175)
        self.assertLessEqual(len(schema_text.splitlines()), 90)


if __name__ == "__main__":
    unittest.main()
