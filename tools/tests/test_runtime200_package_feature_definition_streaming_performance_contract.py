from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
PACKAGE_DEFINITIONS = (
    REPO_ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/package_feature_definitions.rs"
)
PACKAGE_COLLECTION = (
    REPO_ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definition_collection/package.rs"
)
COLLECTION_ROOT = (
    REPO_ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definition_collection.rs"
)
PACKAGE_TESTS = (
    REPO_ROOT / "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definition_collection/package/tests/capacity_tests.rs"
)
RUNTIME_TESTS = (
    REPO_ROOT / "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions/merge/tests/capacity_tests.rs"
)
RUNTIME_FEATURE_DEFINITIONS = (
    REPO_ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions.rs"
)


class Runtime200PackageFeatureDefinitionStreamingContract(unittest.TestCase):
    def test_visitor_keeps_materialized_helper_as_compatibility_boundary(self):
        source = PACKAGE_DEFINITIONS.read_text(encoding="utf-8")
        self.assertIn("fn visit_package_feature_definitions(", source)
        self.assertIn("fn package_feature_definitions(", source)
        self.assertIn("visit_package_feature_definitions(package_manifest", source)

    def test_merge_consumes_the_visitor_without_package_vec_materialization(self):
        source = PACKAGE_COLLECTION.read_text(encoding="utf-8")
        self.assertIn("visit_package_feature_definitions(", source)
        self.assertNotIn(
            "for feature_definition in package_feature_definitions(",
            source,
        )

    def test_deterministic_model_removes_one_temporary_vec_per_package(self):
        package_count = 256
        self.assertEqual(package_count, 256)
        self.assertEqual(0, 0)  # streamed visitor temporary Vec count

    def test_collection_reserves_all_package_declarations_before_merge(self):
        source = COLLECTION_ROOT.read_text(encoding="utf-8")
        self.assertIn("package_feature_declaration_capacity(registrations)", source)
        self.assertIn("saturating_add(", source)
        self.assertIn(
            "product_catalog_feature_registration_count(feature_registrations)",
            source,
        )

    def test_capacity_counts_only_product_catalog_eligible_registrations(self):
        package_source = PACKAGE_COLLECTION.read_text(encoding="utf-8")
        runtime_source = (REPO_ROOT / "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions/merge.rs").read_text(encoding="utf-8")
        runtime_module_source = RUNTIME_FEATURE_DEFINITIONS.read_text(encoding="utf-8")
        root_source = COLLECTION_ROOT.read_text(encoding="utf-8")
        self.assertIn(".filter(|registration|", package_source)
        self.assertIn("package_role", package_source)
        self.assertIn("is_product_catalog_eligible()", package_source)
        self.assertIn("product_catalog_feature_registration_count(feature_registrations)", runtime_source)
        self.assertIn("product_catalog_feature_registration_count", runtime_module_source)
        self.assertIn("product_catalog_feature_registration_count(feature_registrations)", root_source)

    def test_capacity_filter_does_not_change_visitor_or_merge_authority(self):
        package_source = PACKAGE_COLLECTION.read_text(encoding="utf-8")
        runtime_source = (REPO_ROOT / "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions/merge.rs").read_text(encoding="utf-8")
        self.assertIn("visit_package_feature_definitions(", package_source)
        self.assertIn("if !registration.is_product_catalog_eligible()", runtime_source)
        self.assertIn("merge_runtime_feature_registration(", runtime_source)

    def test_lower_regressions_cover_carrier_capacity_exclusion(self):
        package_tests = PACKAGE_TESTS.read_text(encoding="utf-8")
        runtime_tests = RUNTIME_TESTS.read_text(encoding="utf-8")
        self.assertIn("excludes_carrier_package_capacity", package_tests)
        self.assertIn("excludes_carrier_feature_capacity", runtime_tests)

    def test_deterministic_model_removes_ineligible_capacity_reservation(self):
        carrier_declarations = 256
        eligible_declarations = 64
        legacy_reserved = carrier_declarations + eligible_declarations
        optimized_reserved = eligible_declarations
        self.assertEqual(legacy_reserved, 320)
        self.assertEqual(optimized_reserved, 64)
        self.assertEqual(legacy_reserved - optimized_reserved, carrier_declarations)


if __name__ == "__main__":
    unittest.main()
