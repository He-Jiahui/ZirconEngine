import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST = (
    REPO_ROOT / "tools/tests/test_plugin_validate_owner_boundaries.py"
)
PLUGIN_VALIDATE = REPO_ROOT / "tools/zircon_export/plugin_validate.py"
PLUGIN_BUILD = REPO_ROOT / "tools/zircon_export/plugin_build.py"
PLUGIN_VALIDATE_COMMON = REPO_ROOT / "tools/zircon_export/plugin_validate_common.py"
PLUGIN_VALIDATE_TARGET_DISCOVERY = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_target_discovery.py"
)
PLUGIN_PACKAGE_IDENTITY = REPO_ROOT / "tools/zircon_export/plugin_package_identity.py"
PLUGIN_PACKAGE_SOURCE = REPO_ROOT / "tools/zircon_export/plugin_package_source.py"
PLUGIN_PACKAGE_TEMPLATE = REPO_ROOT / "tools/zircon_export/plugin_package_template.py"
PLUGIN_VALIDATE_FEATURE_PROVIDER = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_SCHEMA = REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider_manifest_schema.py"
PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_PARSE = (
    REPO_ROOT
    / "tools/zircon_export/plugin_validate_feature_provider_manifest_parse.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_PROJECTION_COMPARE = (
    REPO_ROOT
    / "tools/zircon_export/plugin_validate_feature_provider_projection_compare.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_PROJECTION_OPTIONAL = (
    REPO_ROOT
    / "tools/zircon_export/plugin_validate_feature_provider_projection_optional.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_DEPENDENCIES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider_dependencies.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_CAPABILITIES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider_capabilities.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_DISTRIBUTION = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider_distribution.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider_extension.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_feature_provider.py"
)

FEATURE_PROVIDER_BOUNDARY_METHODS = (
    "test_feature_provider_package_id_lives_in_identity_owner",
    "test_plugin_package_source_resolution_lives_in_source_owner",
    "test_feature_provider_package_template_lives_in_template_owner",
    "test_feature_provider_projection_tests_import_projection_owner",
    "test_feature_provider_projection_compare_helpers_live_in_compare_owner",
    "test_feature_provider_projection_optional_helpers_live_in_optional_owner",
    "test_feature_provider_dependencies_lives_in_dependencies_owner",
    "test_feature_provider_capabilities_lives_in_capabilities_owner",
    "test_feature_provider_distribution_lives_in_distribution_owner",
    "test_feature_provider_extension_lives_in_extension_owner",
)


class PluginValidateFeatureProviderOwnerBoundaryTests(unittest.TestCase):
    def test_feature_provider_boundaries_leave_general_owner_file(self):
        general_owner_text = PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in FEATURE_PROVIDER_BOUNDARY_METHODS:
            self.assertNotIn(
                f"def {method_name}(",
                general_owner_text,
                f"{method_name} belongs in test_plugin_validate_feature_provider_owner_boundaries.py",
            )

        self.assertLessEqual(
            len(general_owner_text.splitlines()),
            4000,
            "general PluginValidate owner boundary tests should shrink after feature-provider split",
        )
        self.assertLessEqual(
            len(Path(__file__).read_text(encoding="utf-8").splitlines()),
            720,
            "feature-provider boundary file should stay below the next split budget",
        )

    def test_feature_provider_package_id_lives_in_identity_owner(self):
        self.assertTrue(
            PLUGIN_PACKAGE_IDENTITY.exists(),
            "feature-provider package id derivation belongs in plugin_package_identity.py",
        )
        common_text = PLUGIN_VALIDATE_COMMON.read_text(encoding="utf-8")
        discovery_text = PLUGIN_VALIDATE_TARGET_DISCOVERY.read_text(encoding="utf-8")
        build_text = PLUGIN_BUILD.read_text(encoding="utf-8")
        identity_text = PLUGIN_PACKAGE_IDENTITY.read_text(encoding="utf-8")
        source_text = (
            PLUGIN_PACKAGE_SOURCE.read_text(encoding="utf-8")
            if PLUGIN_PACKAGE_SOURCE.exists()
            else ""
        )

        self.assertIn(
            "def feature_provider_package_id(",
            identity_text,
        )
        self.assertNotIn(
            "def feature_provider_package_id(",
            build_text,
            "plugin_build.py must consume feature-provider package id derivation from plugin_package_identity.py",
        )
        self.assertIn(
            "from .plugin_package_identity import feature_provider_package_id",
            source_text,
            "plugin package source owner consumes feature-provider package id derivation",
        )
        self.assertNotIn(
            "from .plugin_package_identity import feature_provider_package_id",
            build_text,
            "plugin_build.py must consume identity derivation through plugin_package_source.py",
        )
        for validate_owner, text in (
            ("plugin_validate_common.py", common_text),
            ("plugin_validate_target_discovery.py", discovery_text),
        ):
            self.assertNotIn(
                "from .plugin_build import feature_provider_package_id",
                text,
                f"{validate_owner} must not borrow feature-provider package id derivation from plugin_build.py",
            )

    def test_plugin_package_source_resolution_lives_in_source_owner(self):
        self.assertTrue(
            PLUGIN_PACKAGE_SOURCE.exists(),
            "root and feature-provider package source resolution belongs in plugin_package_source.py",
        )
        source_text = PLUGIN_PACKAGE_SOURCE.read_text(encoding="utf-8")
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        build_text = PLUGIN_BUILD.read_text(encoding="utf-8")

        for function_name in (
            "default_repo_root",
            "resolve_plugin_package_path",
            "resolve_plugin_package_source",
            "root_plugin_package_source",
            "feature_provider_plugin_package_source",
        ):
            self.assertIn(
                f"def {function_name}(",
                source_text,
                f"{function_name} belongs in plugin_package_source.py",
            )

        self.assertIn("class PluginPackageSource", source_text)
        for function_name in (
            "resolve_plugin_build_source",
            "root_plugin_build_source",
            "feature_provider_plugin_build_source",
            "feature_provider_package_manifest_template",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                build_text,
                f"{function_name} must not remain owned by plugin_build.py",
            )
        self.assertNotIn(
            "from .plugin_build import",
            validate_text,
            "plugin validate must not import package source resolution from plugin_build.py",
        )
        self.assertIn(
            "from .plugin_package_source import",
            validate_text,
        )

    def test_feature_provider_package_template_lives_in_template_owner(self):
        self.assertTrue(
            PLUGIN_PACKAGE_TEMPLATE.exists(),
            "feature-provider generated package manifest templating belongs in plugin_package_template.py",
        )
        source_text = PLUGIN_PACKAGE_SOURCE.read_text(encoding="utf-8")
        template_text = PLUGIN_PACKAGE_TEMPLATE.read_text(encoding="utf-8")

        self.assertIn(
            "from .plugin_package_template import feature_provider_package_manifest_template",
            source_text,
            "plugin_package_source.py should consume generated manifest templating through the template owner",
        )
        for function_name in (
            "feature_provider_package_manifest_template",
            "feature_provider_supported_targets",
            "feature_provider_runtime_module",
            "first_feature_module",
            "feature_provider_dependencies",
            "feature_string",
            "feature_string_array",
            "toml_string_array",
            "toml_bool",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                source_text,
                f"{function_name} belongs in plugin_package_template.py",
            )
            self.assertIn(
                f"def {function_name}(",
                template_text,
            )

        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
        ):
            self.assertNotIn(
                forbidden_import,
                template_text,
                "generated package templating must stay independent from build and validate entry owners",
            )

    def test_feature_provider_projection_tests_import_projection_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_FEATURE_PROVIDER.exists(),
            "feature-provider projection validation belongs in plugin_validate_feature_provider.py",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        feature_provider_text = PLUGIN_VALIDATE_FEATURE_PROVIDER.read_text(
            encoding="utf-8"
        )
        manifest_schema_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_SCHEMA.read_text(encoding="utf-8")
        manifest_parse_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_PARSE.read_text(
            encoding="utf-8"
        )
        feature_provider_test_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_TEST.read_text(
            encoding="utf-8"
        )

        self.assertIn(
            "def validate_plugin_feature_provider_package_projection(",
            feature_provider_text,
        )
        for symbol in (
            "PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_FIELDS",
            "plugin_validate_feature_provider_manifest_known_fields",
            "is not a known feature provider manifest field",
        ):
            self.assertIn(symbol, feature_provider_text)
        for symbol in (
            "PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_STRING_FIELDS",
            "PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_ARRAY_FIELDS",
            "plugin_validate_feature_provider_manifest_metadata_schema",
            "plugin_validate_feature_provider_manifest_projection_consistency",
        ):
            self.assertIn(symbol, manifest_schema_text)
            self.assertNotIn(f"def {symbol}(", feature_provider_text)
        self.assertIn("from .plugin_validate_feature_provider_manifest_schema import", feature_provider_text)
        self.assertIn(
            "from .plugin_validate_feature_provider_manifest_parse import",
            feature_provider_text,
        )
        self.assertIn(
            "def plugin_validate_generated_package_manifest(", manifest_parse_text
        )
        self.assertNotIn(
            "def plugin_validate_generated_package_manifest(", feature_provider_text
        )
        self.assertLessEqual(
            len(manifest_parse_text.splitlines()),
            45,
            "generated package TOML parsing should stay in a focused leaf",
        )
        self.assertIn(
            "from tools.zircon_export.plugin_validate_feature_provider import",
            feature_provider_test_text,
            "feature-provider tests must call the projection owner directly",
        )
        self.assertNotIn(
            "from tools.zircon_export.plugin_validate import",
            feature_provider_test_text,
            "feature-provider tests must not borrow projection helpers from the validate entry owner",
        )
        self.assertNotIn(
            "validate_plugin_feature_provider_package_projection(",
            validate_text,
            "plugin_validate.py should not expose the projection helper name as an entry-owner API",
        )

    def test_feature_provider_projection_compare_helpers_live_in_compare_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_FEATURE_PROVIDER_PROJECTION_COMPARE.exists(),
            "feature-provider projection field comparison belongs in a focused compare owner",
        )
        feature_provider_text = PLUGIN_VALIDATE_FEATURE_PROVIDER.read_text(
            encoding="utf-8"
        )
        distribution_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_DISTRIBUTION.read_text(
            encoding="utf-8"
        )
        compare_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_PROJECTION_COMPARE.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "plugin_validate_compare_string_array_projection",
            "plugin_validate_compare_int_projection",
            "plugin_validate_compare_required_string_projection",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                feature_provider_text,
                f"{function_name} belongs in the projection compare owner",
            )
            self.assertIn(
                f"def {function_name}(",
                compare_text,
            )

        self.assertIn(
            "from .plugin_validate_feature_provider_projection_compare import",
            distribution_text,
            "feature-provider distribution owner should consume required field comparison through the compare owner",
        )
        self.assertNotIn(
            "from .plugin_validate_feature_provider_projection_compare import",
            feature_provider_text,
            "feature-provider parent should dispatch distribution checks instead of consuming compare helpers directly",
        )
        for forbidden_import in (
            "from .plugin_validate import",
            "from .plugin_build import",
            "from .plugin_validate_feature_provider import",
        ):
            self.assertNotIn(
                forbidden_import,
                compare_text,
                "projection compare owner must not borrow entry or parent orchestration owners",
            )
        self.assertLessEqual(
            len(feature_provider_text.splitlines()),
            340,
            "feature-provider projection orchestration owner should stay below the split budget",
        )
        self.assertLessEqual(
            len(compare_text.splitlines()),
            150,
            "feature-provider projection compare owner should stay below the split budget",
        )

    def test_feature_provider_projection_optional_helpers_live_in_optional_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_FEATURE_PROVIDER_PROJECTION_OPTIONAL.exists(),
            "feature-provider optional projection field comparison belongs in a focused optional owner",
        )
        distribution_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_DISTRIBUTION.read_text(
            encoding="utf-8"
        )
        compare_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_PROJECTION_COMPARE.read_text(
            encoding="utf-8"
        )
        optional_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_PROJECTION_OPTIONAL.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "plugin_validate_compare_optional_string_projection",
            "plugin_validate_compare_optional_string_array_projection",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                compare_text,
                f"{function_name} belongs in the projection optional owner",
            )
            self.assertIn(
                f"def {function_name}(",
                optional_text,
            )

        self.assertIn(
            "from .plugin_validate_feature_provider_projection_optional import",
            distribution_text,
            "feature-provider distribution owner should consume optional field comparison through the optional owner",
        )
        self.assertNotIn(
            "plugin_validate_compare_optional_string_projection",
            compare_text,
            "optional string comparison should not remain in the required compare owner",
        )
        self.assertNotIn(
            "plugin_validate_compare_optional_string_array_projection",
            compare_text,
            "optional string-array comparison should not remain in the required compare owner",
        )
        self.assertNotIn(
            "plugin_validate_optional_trimmed_string",
            compare_text,
            "optional string parsing belongs in the optional projection owner",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_feature_provider import",
            "from .plugin_validate_feature_provider_projection_compare import",
        ):
            self.assertNotIn(
                forbidden_import,
                optional_text,
                "projection optional owner must not borrow entry, parent, or sibling orchestration owners",
            )
        self.assertLessEqual(
            len(compare_text.splitlines()),
            150,
            "projection compare owner should shrink after optional owner split",
        )
        self.assertLessEqual(
            len(optional_text.splitlines()),
            120,
            "projection optional owner should stay a focused leaf module",
        )

    def test_feature_provider_dependencies_lives_in_dependencies_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_FEATURE_PROVIDER_DEPENDENCIES.exists(),
            "feature-provider dependency projection belongs in plugin_validate_feature_provider_dependencies.py",
        )
        feature_provider_text = PLUGIN_VALIDATE_FEATURE_PROVIDER.read_text(
            encoding="utf-8"
        )
        extension_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION.read_text(
            encoding="utf-8"
        )
        dependencies_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_DEPENDENCIES.read_text(
            encoding="utf-8"
        )

        self.assertIn(
            "def plugin_validate_feature_dependencies(",
            dependencies_text,
            "dependency array parsing belongs in the feature-provider dependencies owner",
        )
        for symbol in (
            "PLUGIN_VALIDATE_FEATURE_PROVIDER_DEPENDENCY_FIELDS",
            "plugin_validate_feature_dependency_known_fields",
            "is not a known feature provider dependency field",
        ):
            self.assertIn(symbol, dependencies_text)
        self.assertIn(
            "def validate_plugin_feature_provider_dependencies(",
            dependencies_text,
            "dependency projection comparison belongs in the feature-provider dependencies owner",
        )
        for function_name in (
            "plugin_validate_feature_dependencies",
            "validate_plugin_feature_provider_dependencies",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                feature_provider_text,
                f"{function_name} must not remain in plugin_validate_feature_provider.py",
            )
        self.assertIn(
            "from .plugin_validate_feature_provider_dependencies import",
            extension_text,
            "feature-provider extension owner should dispatch dependency checks to the leaf owner",
        )
        self.assertNotIn(
            "from .plugin_validate_feature_provider_dependencies import",
            feature_provider_text,
            "feature-provider parent should dispatch through the extension owner",
        )
        self.assertNotIn(
            "plugin_validate_trimmed_string",
            feature_provider_text,
            "trimmed dependency field parsing belongs in the dependencies leaf owner",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
        ):
            self.assertNotIn(
                forbidden_import,
                dependencies_text,
                "feature-provider dependencies owner must stay independent from build and validate entry owners",
            )
        self.assertLessEqual(
            len(feature_provider_text.splitlines()),
            260,
            "feature-provider projection owner must stay below the next split threshold",
        )
        self.assertLessEqual(
            len(dependencies_text.splitlines()),
            90,
            "feature-provider dependencies owner must stay a focused leaf module",
        )

    def test_feature_provider_capabilities_lives_in_capabilities_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_FEATURE_PROVIDER_CAPABILITIES.exists(),
            "feature-provider capability projection belongs in plugin_validate_feature_provider_capabilities.py",
        )
        feature_provider_text = PLUGIN_VALIDATE_FEATURE_PROVIDER.read_text(
            encoding="utf-8"
        )
        extension_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION.read_text(
            encoding="utf-8"
        )
        capabilities_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_CAPABILITIES.read_text(
            encoding="utf-8"
        )

        self.assertIn(
            "def validate_plugin_feature_provider_capabilities(",
            capabilities_text,
            "capability projection comparison belongs in the feature-provider capabilities owner",
        )
        self.assertNotIn(
            "def validate_plugin_feature_provider_capabilities(",
            feature_provider_text,
            "validate_plugin_feature_provider_capabilities must not remain in plugin_validate_feature_provider.py",
        )
        self.assertIn(
            "from .plugin_validate_feature_provider_capabilities import",
            extension_text,
            "feature-provider extension owner should dispatch capability checks to the leaf owner",
        )
        self.assertNotIn(
            "from .plugin_validate_feature_provider_capabilities import",
            feature_provider_text,
            "feature-provider parent should dispatch through the extension owner",
        )
        self.assertNotIn(
            "plugin_validate_string_array",
            feature_provider_text,
            "capability array parsing belongs in the capabilities leaf owner",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_feature_provider import",
        ):
            self.assertNotIn(
                forbidden_import,
                capabilities_text,
                "feature-provider capabilities owner must stay independent from build, validate entry, and parent owners",
            )
        self.assertLessEqual(
            len(feature_provider_text.splitlines()),
            230,
            "feature-provider projection owner should keep shrinking after the capabilities split",
        )
        self.assertLessEqual(
            len(capabilities_text.splitlines()),
            70,
            "feature-provider capabilities owner must stay a focused leaf module",
        )

    def test_feature_provider_distribution_lives_in_distribution_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_FEATURE_PROVIDER_DISTRIBUTION.exists(),
            "feature-provider distribution projection belongs in plugin_validate_feature_provider_distribution.py",
        )
        feature_provider_text = PLUGIN_VALIDATE_FEATURE_PROVIDER.read_text(
            encoding="utf-8"
        )
        extension_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION.read_text(
            encoding="utf-8"
        )
        distribution_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_DISTRIBUTION.read_text(
            encoding="utf-8"
        )

        self.assertIn(
            "def validate_plugin_feature_provider_distribution_projection(",
            distribution_text,
            "distribution projection comparison belongs in the feature-provider distribution owner",
        )
        for symbol in (
            "PLUGIN_VALIDATE_FEATURE_PROVIDER_DISTRIBUTION_FIELDS",
            "plugin_validate_feature_provider_distribution_known_fields",
            "is not a known feature provider distribution field",
        ):
            self.assertIn(symbol, distribution_text)
        self.assertNotIn(
            "def validate_plugin_feature_provider_distribution_projection(",
            feature_provider_text,
            "validate_plugin_feature_provider_distribution_projection must not remain in plugin_validate_feature_provider.py",
        )
        self.assertIn(
            "from .plugin_validate_feature_provider_distribution import",
            extension_text,
            "feature-provider extension owner should dispatch distribution checks to the leaf owner",
        )
        self.assertNotIn(
            "from .plugin_validate_feature_provider_distribution import",
            feature_provider_text,
            "feature-provider parent should dispatch through the extension owner",
        )
        for helper_name in (
            "plugin_validate_compare_string_array_projection",
            "plugin_validate_compare_int_projection",
            "plugin_validate_compare_required_string_projection",
            "plugin_validate_compare_optional_string_projection",
            "plugin_validate_compare_optional_string_array_projection",
        ):
            self.assertNotIn(
                helper_name,
                feature_provider_text,
                f"{helper_name} belongs behind the distribution owner import boundary",
            )
            self.assertIn(
                helper_name,
                distribution_text,
                f"{helper_name} should be consumed by the distribution owner",
            )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_feature_provider import",
        ):
            self.assertNotIn(
                forbidden_import,
                distribution_text,
                "feature-provider distribution owner must stay independent from build, validate entry, and parent owners",
            )
        self.assertLessEqual(
            len(feature_provider_text.splitlines()),
            170,
            "feature-provider projection owner should keep shrinking after the distribution split",
        )
        self.assertLessEqual(
            len(distribution_text.splitlines()),
            100,
            "feature-provider distribution owner must stay a focused leaf module",
        )

    def test_feature_provider_extension_lives_in_extension_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION.exists(),
            "feature-provider extension projection belongs in plugin_validate_feature_provider_extension.py",
        )
        feature_provider_text = PLUGIN_VALIDATE_FEATURE_PROVIDER.read_text(
            encoding="utf-8"
        )
        extension_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION.read_text(
            encoding="utf-8"
        )

        self.assertIn(
            "def validate_plugin_feature_extension_projection(",
            extension_text,
            "feature extension owner should validate owner manifest selection and generated feature identity",
        )
        for symbol in (
            "PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION_FIELDS",
            "plugin_validate_feature_provider_extension_known_fields",
            "is not a known feature provider extension field",
        ):
            self.assertIn(symbol, extension_text)
        self.assertNotIn(
            "def validate_plugin_feature_extension_projection(",
            feature_provider_text,
            "validate_plugin_feature_extension_projection must not remain in plugin_validate_feature_provider.py",
        )
        self.assertIn(
            "from .plugin_validate_feature_provider_extension import",
            feature_provider_text,
            "feature-provider projection parent should dispatch extension checks to the extension owner",
        )
        for owner_import in (
            "plugin_validate_manifest_target_id",
            "plugin_validate_selected_feature",
            "read_toml",
        ):
            self.assertNotIn(
                owner_import,
                feature_provider_text,
                f"{owner_import} belongs behind the extension owner import boundary",
            )
            self.assertIn(
                owner_import,
                extension_text,
                f"{owner_import} should be consumed by the extension owner",
            )
        for child_owner_import in (
            "from .plugin_validate_feature_provider_capabilities import",
            "from .plugin_validate_feature_provider_dependencies import",
            "from .plugin_validate_feature_provider_distribution import",
        ):
            self.assertNotIn(
                child_owner_import,
                feature_provider_text,
                "feature-provider parent should not import projection leaf owners after the extension split",
            )
            self.assertIn(
                child_owner_import,
                extension_text,
                "feature-provider extension owner should orchestrate projection leaf owners",
            )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_feature_provider import",
        ):
            self.assertNotIn(
                forbidden_import,
                extension_text,
                "feature-provider extension owner must stay independent from build, validate entry, and parent owners",
            )
        self.assertLessEqual(
            len(feature_provider_text.splitlines()),
            90,
            "feature-provider package projection parent should become a narrow generated-package owner",
        )
        self.assertLessEqual(
            len(extension_text.splitlines()),
            130,
            "feature-provider extension owner must stay focused on owner manifest and child dispatch",
        )
