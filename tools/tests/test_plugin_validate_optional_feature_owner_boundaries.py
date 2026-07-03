import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST = (
    REPO_ROOT / "tools/tests/test_plugin_validate_owner_boundaries.py"
)
PLUGIN_VALIDATE = REPO_ROOT / "tools/zircon_export/plugin_validate.py"
PLUGIN_VALIDATE_SINGLE_TARGET = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_single_target.py"
)
PLUGIN_VALIDATE_DEPENDENCIES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_dependencies.py"
)
PLUGIN_VALIDATE_DEFAULT_PACKAGING_METADATA = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_default_packaging.py"
)
PLUGIN_VALIDATE_MODULES = REPO_ROOT / "tools/zircon_export/plugin_validate_modules.py"
PLUGIN_VALIDATE_OPTIONAL_FEATURE_DEPENDENCIES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_optional_feature_dependencies.py"
)
PLUGIN_VALIDATE_OPTIONAL_FEATURE_DISTRIBUTION = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_optional_feature_distribution.py"
)
PLUGIN_VALIDATE_OPTIONAL_FEATURE_DEPENDENCY_CAPABILITIES = (
    REPO_ROOT
    / "tools/zircon_export/plugin_validate_optional_feature_dependency_capabilities.py"
)
PLUGIN_VALIDATE_OPTIONAL_FEATURES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_optional_features.py"
)
PLUGIN_VALIDATE_FEATURE_EXTENSIONS = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_extensions.py"
)
PLUGIN_VALIDATE_PACKAGE_KIND = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_package_kind.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_DEPENDENCIES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider_dependencies.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_DISTRIBUTION = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider_distribution.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider_extension.py"
)
PLUGIN_VALIDATE_TEST = REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate.py"
PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_distribution_contract.py"
)
PLUGIN_VALIDATE_OPTIONAL_FEATURES_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_optional_features.py"
)
PLUGIN_VALIDATE_OPTIONAL_FEATURE_DEPENDENCIES_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_plugin_validate_optional_feature_dependencies.py"
)
PLUGIN_VALIDATE_OPTIONAL_FEATURE_DISTRIBUTION_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_plugin_validate_optional_feature_distribution.py"
)
PLUGIN_VALIDATE_FEATURE_EXTENSIONS_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_feature_extensions.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_feature_provider.py"
)

OPTIONAL_FEATURE_BOUNDARY_METHODS = (
    "test_optional_feature_dependencies_lives_in_optional_feature_owner",
    "test_optional_feature_distribution_lives_in_optional_feature_distribution_owner",
    "test_optional_features_schema_lives_in_optional_features_owner",
    "test_feature_extensions_schema_lives_in_feature_extensions_owner",
    "test_optional_feature_dependency_capabilities_lives_in_capability_owner",
)


class PluginValidateOptionalFeatureOwnerBoundaryTests(unittest.TestCase):
    def test_optional_feature_boundaries_leave_general_owner_file(self):
        general_owner_text = PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in OPTIONAL_FEATURE_BOUNDARY_METHODS:
            self.assertNotIn(
                f"def {method_name}(",
                general_owner_text,
                f"{method_name} belongs in test_plugin_validate_optional_feature_owner_boundaries.py",
            )

        self.assertLessEqual(
            len(general_owner_text.splitlines()),
            700,
            "general PluginValidate owner boundary tests should shrink below budget after optional-feature split",
        )
        self.assertLessEqual(
            len(Path(__file__).read_text(encoding="utf-8").splitlines()),
            560,
            "focused PluginValidate optional-feature owner boundary file should stay narrow",
        )

    def test_optional_feature_dependencies_lives_in_optional_feature_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_OPTIONAL_FEATURE_DEPENDENCIES.exists(),
            "optional feature dependency rows belong in a focused root owner",
        )
        self.assertTrue(
            PLUGIN_VALIDATE_OPTIONAL_FEATURE_DEPENDENCIES_TEST.exists(),
            "optional feature dependency behavior tests should stay focused",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")
        dependencies_text = PLUGIN_VALIDATE_DEPENDENCIES.read_text(encoding="utf-8")
        feature_provider_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_DEPENDENCIES.read_text(
            encoding="utf-8"
        )
        validate_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        feature_extension_test_text = PLUGIN_VALIDATE_FEATURE_EXTENSIONS_TEST.read_text(
            encoding="utf-8"
        )
        optional_test_text = PLUGIN_VALIDATE_OPTIONAL_FEATURE_DEPENDENCIES_TEST.read_text(
            encoding="utf-8"
        )
        optional_text = PLUGIN_VALIDATE_OPTIONAL_FEATURE_DEPENDENCIES.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "validate_plugin_optional_feature_dependencies",
            "validate_plugin_feature_extension_dependencies",
            "PLUGIN_VALIDATE_OPTIONAL_FEATURE_DEPENDENCY_FIELDS",
            "validate_plugin_optional_feature_dependency_known_fields",
            "validate_plugin_optional_feature_dependency_row",
            "plugin_validate_optional_feature_dependency_identity",
            "is not a known optional feature dependency field",
            "exactly one primary dependency",
            "primary dependency plugin_id must match package id",
            "primary dependency plugin_id must match owner plugin id",
            "duplicates dependency row",
        ):
            self.assertIn(symbol, optional_text)
        self.assertIn(
            "from .plugin_validate_optional_feature_dependencies import",
            single_target_text,
            "single-target owner should dispatch optional feature dependency checks",
        )
        self.assertIn(
            "validate_plugin_optional_feature_dependencies(",
            single_target_text,
        )
        for parent_text, parent_name in (
            (validate_text, "plugin_validate.py"),
            (dependencies_text, "plugin_validate_dependencies.py"),
            (
                feature_provider_text,
                "plugin_validate_feature_provider_dependencies.py",
            ),
        ):
            self.assertNotIn(
                "def validate_plugin_optional_feature_dependencies(",
                parent_text,
                f"{parent_name} must not own root optional feature dependencies",
            )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_dependencies import",
            "from .plugin_validate_feature_provider_dependencies import",
        ):
            self.assertNotIn(
                forbidden_import,
                optional_text,
                "optional feature dependency owner must stay independent",
            )
        self.assertLessEqual(
            len(optional_text.splitlines()),
            200,
            "optional feature dependency owner should stay focused",
        )
        for test_name in (
            "test_plugin_validate_rejects_malformed_feature_extension_dependencies",
            "test_plugin_validate_rejects_feature_extension_primary_owner_mismatch",
            "test_plugin_validate_rejects_unknown_feature_dependency_fields",
        ):
            self.assertIn(f"def {test_name}(", optional_test_text)
            for source_text, source_name in (
                (validate_test_text, "test_plugin_validate.py"),
                (
                    feature_extension_test_text,
                    "test_plugin_validate_feature_extensions.py",
                ),
            ):
                self.assertNotIn(
                    f"def {test_name}(",
                    source_text,
                    f"{test_name} belongs in optional feature dependency tests, "
                    f"not {source_name}",
                )

    def test_optional_feature_distribution_lives_in_optional_feature_distribution_owner(
        self,
    ):
        self.assertTrue(
            PLUGIN_VALIDATE_OPTIONAL_FEATURE_DISTRIBUTION.exists(),
            "optional feature distribution schema belongs in a focused root owner",
        )
        self.assertTrue(
            PLUGIN_VALIDATE_OPTIONAL_FEATURE_DISTRIBUTION_TEST.exists(),
            "optional feature distribution behavior tests should stay focused",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")
        optional_features_text = PLUGIN_VALIDATE_OPTIONAL_FEATURES.read_text(
            encoding="utf-8"
        )
        optional_dependencies_text = (
            PLUGIN_VALIDATE_OPTIONAL_FEATURE_DEPENDENCIES.read_text(encoding="utf-8")
        )
        default_packaging_text = PLUGIN_VALIDATE_DEFAULT_PACKAGING_METADATA.read_text(
            encoding="utf-8"
        )
        feature_provider_distribution_text = (
            PLUGIN_VALIDATE_FEATURE_PROVIDER_DISTRIBUTION.read_text(encoding="utf-8")
        )
        validate_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        distribution_test_text = PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT_TEST.read_text(
            encoding="utf-8"
        )
        feature_provider_test_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_TEST.read_text(
            encoding="utf-8"
        )
        optional_features_test_text = PLUGIN_VALIDATE_OPTIONAL_FEATURES_TEST.read_text(
            encoding="utf-8"
        )
        optional_test_text = PLUGIN_VALIDATE_OPTIONAL_FEATURE_DISTRIBUTION_TEST.read_text(
            encoding="utf-8"
        )
        distribution_text = PLUGIN_VALIDATE_OPTIONAL_FEATURE_DISTRIBUTION.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "validate_plugin_optional_feature_distribution",
            "validate_plugin_optional_feature_distribution_row",
            "validate_plugin_distribution(",
            "distribution_label=",
            "optional_features[{index}].distribution",
        ):
            self.assertIn(symbol, distribution_text)
        self.assertIn(
            "from .plugin_validate_optional_feature_distribution import",
            single_target_text,
            "single-target owner should dispatch optional feature distribution checks",
        )
        self.assertIn(
            "validate_plugin_optional_feature_distribution(",
            single_target_text,
        )
        for parent_text, parent_name in (
            (validate_text, "plugin_validate.py"),
            (optional_features_text, "plugin_validate_optional_features.py"),
            (
                optional_dependencies_text,
                "plugin_validate_optional_feature_dependencies.py",
            ),
            (default_packaging_text, "plugin_validate_default_packaging.py"),
            (
                feature_provider_distribution_text,
                "plugin_validate_feature_provider_distribution.py",
            ),
        ):
            self.assertNotIn(
                "def validate_plugin_optional_feature_distribution(",
                parent_text,
                f"{parent_name} must not own optional feature distribution schema",
            )
        self.assertIn(
            "def test_plugin_validate_rejects_optional_feature_distribution_contract(",
            optional_test_text,
        )
        for source_name, source_text in (
            ("test_plugin_validate.py", validate_test_text),
            (
                "test_plugin_validate_distribution_contract.py",
                distribution_test_text,
            ),
            (
                "test_plugin_validate_feature_provider.py",
                feature_provider_test_text,
            ),
            (
                "test_plugin_validate_optional_features.py",
                optional_features_test_text,
            ),
        ):
            self.assertNotIn(
                "def test_plugin_validate_rejects_optional_feature_distribution_contract(",
                source_text,
                "optional feature distribution contract test belongs in the "
                f"focused owner, not {source_name}",
            )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_feature_provider_distribution import",
        ):
            self.assertNotIn(
                forbidden_import,
                distribution_text,
                "optional feature distribution owner must stay independent",
            )
        self.assertLessEqual(
            len(distribution_text.splitlines()),
            110,
            "optional feature distribution owner should stay focused",
        )

    def test_optional_features_schema_lives_in_optional_features_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_OPTIONAL_FEATURES.exists(),
            "optional feature row schema belongs in a focused root owner",
        )
        self.assertTrue(
            PLUGIN_VALIDATE_OPTIONAL_FEATURES_TEST.exists(),
            "optional feature behavior tests should stay in their focused test owner",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")
        dependencies_text = PLUGIN_VALIDATE_OPTIONAL_FEATURE_DEPENDENCIES.read_text(
            encoding="utf-8"
        )
        default_packaging_text = PLUGIN_VALIDATE_DEFAULT_PACKAGING_METADATA.read_text(
            encoding="utf-8"
        )
        modules_text = PLUGIN_VALIDATE_MODULES.read_text(encoding="utf-8")
        optional_text = PLUGIN_VALIDATE_OPTIONAL_FEATURES.read_text(encoding="utf-8")
        validate_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        optional_test_text = PLUGIN_VALIDATE_OPTIONAL_FEATURES_TEST.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "validate_plugin_optional_features",
            "validate_plugin_optional_feature_row",
            "PLUGIN_VALIDATE_OPTIONAL_FEATURE_FIELDS",
            "validate_plugin_optional_feature_id",
            "plugin_validate_optional_trimmed_string",
            "provider_package_id",
            ".provider_package_id",
            "enabled_by_default must be a bool",
            "is not a known optional feature field",
        ):
            self.assertIn(symbol, optional_text)
        self.assertIn(
            "from .plugin_validate_optional_features import",
            single_target_text,
            "single-target owner should dispatch optional feature schema checks",
        )
        self.assertIn(
            "validate_plugin_optional_features(",
            single_target_text,
        )
        for parent_text, parent_name in (
            (validate_text, "plugin_validate.py"),
            (
                dependencies_text,
                "plugin_validate_optional_feature_dependencies.py",
            ),
            (
                default_packaging_text,
                "plugin_validate_default_packaging.py",
            ),
            (modules_text, "plugin_validate_modules.py"),
        ):
            self.assertNotIn(
                "def validate_plugin_optional_features(",
                parent_text,
                f"{parent_name} must not own optional feature row schema",
            )
        for test_name in (
            "test_plugin_validate_rejects_malformed_optional_feature_schema",
            "test_plugin_validate_rejects_non_table_optional_feature_row",
            "test_plugin_validate_rejects_optional_feature_provider_package_id_schema",
        ):
            self.assertIn(f"def {test_name}(", optional_test_text)
            self.assertNotIn(
                f"def {test_name}(",
                validate_test_text,
                f"{test_name} belongs in the optional feature test owner",
            )
        self.assertLessEqual(
            len(optional_text.splitlines()),
            180,
            "optional feature schema owner should stay focused",
        )

    def test_feature_extensions_schema_lives_in_feature_extensions_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_FEATURE_EXTENSIONS.exists(),
            "feature extension row schema belongs in a focused root owner",
        )
        self.assertTrue(
            PLUGIN_VALIDATE_FEATURE_EXTENSIONS_TEST.exists(),
            "feature extension behavior tests should stay in their focused test owner",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")
        package_kind_text = PLUGIN_VALIDATE_PACKAGE_KIND.read_text(encoding="utf-8")
        provider_text = PLUGIN_VALIDATE_FEATURE_PROVIDER.read_text(encoding="utf-8")
        provider_extension_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION.read_text(
            encoding="utf-8"
        )
        dependencies_text = PLUGIN_VALIDATE_OPTIONAL_FEATURE_DEPENDENCIES.read_text(
            encoding="utf-8"
        )
        optional_text = PLUGIN_VALIDATE_OPTIONAL_FEATURES.read_text(encoding="utf-8")
        feature_extension_text = PLUGIN_VALIDATE_FEATURE_EXTENSIONS.read_text(
            encoding="utf-8"
        )
        validate_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        provider_test_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_TEST.read_text(
            encoding="utf-8"
        )
        feature_extension_test_text = PLUGIN_VALIDATE_FEATURE_EXTENSIONS_TEST.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "validate_plugin_feature_extensions",
            "validate_plugin_feature_extension_row",
            "PLUGIN_VALIDATE_FEATURE_EXTENSION_FIELDS",
            "validate_plugin_feature_extension_id",
            "validate_plugin_feature_extension_owner_package_token",
            "plugin_validate_optional_trimmed_string",
            "provider_package_id",
            ".provider_package_id",
            "enabled_by_default must be a bool",
            "is not a known feature extension field",
        ):
            self.assertIn(symbol, feature_extension_text)
        self.assertIn(
            "from .plugin_validate_feature_extensions import",
            single_target_text,
            "single-target owner should dispatch feature extension schema checks",
        )
        self.assertIn("validate_plugin_feature_extensions(", single_target_text)
        for parent_text, parent_name in (
            (validate_text, "plugin_validate.py"),
            (package_kind_text, "plugin_validate_package_kind.py"),
            (provider_text, "plugin_validate_feature_provider.py"),
            (
                provider_extension_text,
                "plugin_validate_feature_provider_extension.py",
            ),
            (
                dependencies_text,
                "plugin_validate_optional_feature_dependencies.py",
            ),
            (optional_text, "plugin_validate_optional_features.py"),
        ):
            self.assertNotIn(
                "def validate_plugin_feature_extensions(",
                parent_text,
                f"{parent_name} must not own feature extension row schema",
            )
        for test_name in (
            "test_plugin_validate_rejects_malformed_feature_extension_schema",
            "test_plugin_validate_rejects_feature_extension_provider_package_id_schema",
        ):
            self.assertIn(f"def {test_name}(", feature_extension_test_text)
            for source_name, source_text in (
                ("test_plugin_validate.py", validate_test_text),
                ("test_plugin_validate_feature_provider.py", provider_test_text),
            ):
                self.assertNotIn(
                    f"def {test_name}(",
                    source_text,
                    f"{test_name} belongs in the feature extension test owner, not {source_name}",
                )
        self.assertLessEqual(
            len(feature_extension_text.splitlines()),
            190,
            "feature extension schema owner should stay focused",
        )

    def test_optional_feature_dependency_capabilities_lives_in_capability_owner(
        self,
    ):
        self.assertTrue(
            PLUGIN_VALIDATE_OPTIONAL_FEATURE_DEPENDENCY_CAPABILITIES.exists(),
            "optional feature dependency capability resolution belongs in a focused owner",
        )
        optional_text = PLUGIN_VALIDATE_OPTIONAL_FEATURE_DEPENDENCIES.read_text(
            encoding="utf-8"
        )
        capability_text = (
            PLUGIN_VALIDATE_OPTIONAL_FEATURE_DEPENDENCY_CAPABILITIES.read_text(
                encoding="utf-8"
            )
        )

        for symbol in (
            "validate_plugin_optional_feature_dependency_capability_gate",
            "referenced static plugin package",
            "runtime.module.* or runtime.capability.*",
        ):
            self.assertIn(symbol, capability_text)
        self.assertIn(
            "from .plugin_validate_optional_feature_dependency_capabilities import",
            optional_text,
            "optional feature dependency owner should dispatch capability resolution to the leaf owner",
        )
        self.assertIn(
            "validate_plugin_optional_feature_dependency_capability_gate(",
            optional_text,
        )
        self.assertNotIn(
            "def validate_plugin_optional_feature_dependency_capability_gate(",
            optional_text,
            "capability resolution policy belongs in the optional feature capability owner",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_optional_feature_dependencies import",
        ):
            self.assertNotIn(
                forbidden_import,
                capability_text,
                "optional feature dependency capability owner must stay independent",
            )
        self.assertLessEqual(
            len(capability_text.splitlines()),
            90,
            "optional feature dependency capability owner should stay small",
        )


if __name__ == "__main__":
    unittest.main()
