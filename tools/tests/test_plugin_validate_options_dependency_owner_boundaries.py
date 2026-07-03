import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE = REPO_ROOT / "tools/zircon_export/plugin_validate.py"
PLUGIN_VALIDATE_SINGLE_TARGET = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_single_target.py"
)
PLUGIN_VALIDATE_ASSET_IMPORTERS = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_asset_importers.py"
)
PLUGIN_VALIDATE_OPTIONS = REPO_ROOT / "tools/zircon_export/plugin_validate_options.py"
PLUGIN_VALIDATE_OPTION_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_option_schema.py"
)
PLUGIN_VALIDATE_OPTION_GLOBAL_KEYS = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_option_global_keys.py"
)
PLUGIN_VALIDATE_DEPENDENCIES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_dependencies.py"
)


class PluginValidateOptionsDependencyOwnerBoundaryTests(unittest.TestCase):
    def test_options_required_capability_gates_lives_in_options_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_OPTIONS.exists(),
            "plugin option required capability gates belong in plugin_validate_options.py",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")
        asset_importers_text = PLUGIN_VALIDATE_ASSET_IMPORTERS.read_text(
            encoding="utf-8"
        )
        options_text = PLUGIN_VALIDATE_OPTIONS.read_text(encoding="utf-8")

        for symbol in (
            "validate_plugin_options",
            "validate_plugin_option_required_capability",
            "options",
            "required_capability",
            "PLUGIN_VALIDATE_REQUIRED_CAPABILITY_DECLARED_GATE_DIAGNOSTIC",
            "plugin_validate_required_capability_is_host_owned",
        ):
            self.assertIn(symbol, options_text)
            self.assertNotIn(
                symbol,
                validate_text,
                "option required capability gates must not live in plugin_validate.py",
            )
        for symbol in (
            "validate_plugin_options",
            "validate_plugin_option_required_capability",
            "plugin option",
            "options[",
        ):
            self.assertNotIn(
                symbol,
                asset_importers_text,
                "option required capability gates must not live in asset_importers owner",
            )
        self.assertIn(
            "from .plugin_validate_options import validate_plugin_options",
            single_target_text,
            "single-target orchestration should dispatch option validation through the options owner",
        )
        self.assertIn("validate_plugin_options(", single_target_text)
        self.assertIn(
            "from .plugin_validate_asset_importer_required_capability_gates import",
            options_text,
            "options owner should reuse the shared static declared capability gate helpers",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_asset_importers import",
        ):
            self.assertNotIn(
                forbidden_import,
                options_text,
                "options owner must stay independent from entry, build, and sibling owners",
            )
        self.assertLessEqual(
            len(options_text.splitlines()),
            120,
            "plugin options owner should stay small and focused",
        )

    def test_dependencies_lives_in_dependencies_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DEPENDENCIES.exists(),
            "top-level plugin dependencies validation belongs in a focused owner",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")
        options_text = PLUGIN_VALIDATE_OPTIONS.read_text(encoding="utf-8")
        asset_importers_text = PLUGIN_VALIDATE_ASSET_IMPORTERS.read_text(
            encoding="utf-8"
        )
        dependencies_text = PLUGIN_VALIDATE_DEPENDENCIES.read_text(encoding="utf-8")

        for symbol in (
            "validate_plugin_dependencies",
            "validate_plugin_dependency_known_fields",
            "validate_plugin_dependency_row",
            "plugin_validate_dependency_row_identity",
            "PLUGIN_VALIDATE_DEPENDENCY_FIELDS",
            "is not a known dependency field",
            "must not be empty when declared",
            "duplicates dependency row",
        ):
            self.assertIn(symbol, dependencies_text)
        self.assertIn(
            "from .plugin_validate_dependencies import validate_plugin_dependencies",
            single_target_text,
            "single-target orchestration should dispatch dependencies validation",
        )
        self.assertIn("validate_plugin_dependencies(", single_target_text)
        for parent_text, parent_name in (
            (validate_text, "plugin_validate.py"),
            (options_text, "plugin_validate_options.py"),
            (asset_importers_text, "plugin_validate_asset_importers.py"),
        ):
            self.assertNotIn(
                "def validate_plugin_dependencies(",
                parent_text,
                f"{parent_name} must not own top-level dependency validation",
            )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_options import",
            "from .plugin_validate_asset_importers import",
            "from .plugin_validate_feature_provider_dependencies import",
        ):
            self.assertNotIn(
                forbidden_import,
                dependencies_text,
                "dependencies owner must stay independent from entry and sibling owners",
            )
        self.assertLessEqual(
            len(dependencies_text.splitlines()),
            120,
            "top-level dependencies owner should stay small and focused",
        )

    def test_option_schema_lives_in_schema_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_OPTION_SCHEMA.exists(),
            "plugin option row schema belongs in plugin_validate_option_schema.py",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        options_text = PLUGIN_VALIDATE_OPTIONS.read_text(encoding="utf-8")
        schema_text = PLUGIN_VALIDATE_OPTION_SCHEMA.read_text(encoding="utf-8")

        for symbol in (
            "validate_plugin_option_schema",
            "validate_plugin_option_known_fields",
            "plugin_validate_option_key",
            "plugin_validate_option_default_value",
            "plugin_validate_option_enum_values",
            "PLUGIN_VALIDATE_OPTION_FIELDS",
            "dot-separated namespace segments",
            "is not a known option field",
            "bool value must be true or false",
            "integer value must parse as i64",
            "number value must be finite",
            "must only be declared for enum options",
        ):
            self.assertIn(symbol, schema_text)
            self.assertNotIn(
                symbol,
                validate_text,
                "option schema policy must not live in plugin_validate.py",
            )
        self.assertIn(
            "from .plugin_validate_option_schema import validate_plugin_option_schema",
            options_text,
            "options owner should dispatch row schema checks to the schema owner",
        )
        self.assertIn("validate_plugin_option_schema(", options_text)
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_options import",
        ):
            self.assertNotIn(
                forbidden_import,
                schema_text,
                "option schema owner must stay independent from entry, build, and parent owners",
            )
        self.assertLessEqual(
            len(schema_text.splitlines()),
            180,
            "plugin option schema owner should stay focused",
        )

    def test_option_global_keys_lives_in_global_key_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_OPTION_GLOBAL_KEYS.exists(),
            "plugin option key global uniqueness checks belong in a focused --all owner",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")
        options_text = PLUGIN_VALIDATE_OPTIONS.read_text(encoding="utf-8")
        global_keys_text = PLUGIN_VALIDATE_OPTION_GLOBAL_KEYS.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "validate_plugin_option_global_keys",
            "validate_plugin_option_manifest_global_keys",
            "plugin_validate_option_global_package_label",
            "plugin validate options key",
            "is duplicated by",
        ):
            self.assertIn(symbol, global_keys_text)
        self.assertIn(
            "from .plugin_validate_option_global_keys import",
            validate_text,
            "plugin validate --all should dispatch global option key checks",
        )
        self.assertIn("validate_plugin_option_global_keys(", validate_text)
        for parent_text, parent_name in (
            (single_target_text, "plugin_validate_single_target.py"),
            (options_text, "plugin_validate_options.py"),
        ):
            self.assertNotIn(
                "validate_plugin_option_global_keys(",
                parent_text,
                f"{parent_name} must not own cross-manifest option key checks",
            )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_options import",
            "from .plugin_validate_option_schema import",
        ):
            self.assertNotIn(
                forbidden_import,
                global_keys_text,
                "global option key owner must stay independent",
            )
        self.assertLessEqual(
            len(global_keys_text.splitlines()),
            110,
            "global option key owner should stay a small leaf module",
        )


if __name__ == "__main__":
    unittest.main()
