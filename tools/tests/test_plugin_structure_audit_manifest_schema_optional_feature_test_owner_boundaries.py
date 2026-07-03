import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

ROOT_TEST = "tools/tests/test_plugin_structure_audit_manifest_schema.py"
OPTIONAL_FEATURE_SCHEMA_TEST = (
    "tools/tests/test_plugin_structure_audit_manifest_schema_optional_features.py"
)
OPTIONAL_FEATURE_MODULE_TEST = (
    "tools/tests/test_plugin_structure_audit_manifest_schema_optional_feature_modules.py"
)
OPTIONAL_FEATURE_DEPENDENCY_TEST = (
    "tools/tests/test_plugin_structure_audit_manifest_schema_optional_feature_dependencies.py"
)
OPTIONAL_FEATURE_DISTRIBUTION_TEST = (
    "tools/tests/test_plugin_structure_audit_manifest_schema_optional_feature_distribution.py"
)

OPTIONAL_FEATURE_SCHEMA_TEST_NAMES = [
    "test_manifest_schema_rejects_empty_optional_features_array",
    "test_manifest_schema_rejects_unknown_optional_feature_field",
    "test_manifest_schema_rejects_optional_feature_identity_semantics",
    "test_manifest_schema_rejects_optional_feature_id_namespace_segments",
    "test_manifest_schema_rejects_duplicate_optional_feature_id",
    "test_manifest_schema_rejects_optional_feature_capability_semantics",
    "test_manifest_schema_rejects_optional_feature_missing_owner",
    "test_manifest_schema_rejects_optional_feature_enabled_by_default_type",
    "test_manifest_schema_rejects_unknown_optional_feature_default_packaging",
    "test_manifest_schema_rejects_optional_feature_provider_package_id_type",
    "test_manifest_schema_rejects_optional_feature_provider_package_id_untrimmed",
]

OPTIONAL_FEATURE_MODULE_TEST_NAMES = [
    "test_manifest_schema_rejects_optional_feature_module_missing_field",
    "test_manifest_schema_rejects_unknown_optional_feature_module_kind",
    "test_manifest_schema_rejects_unknown_optional_feature_module_target_mode",
]

OPTIONAL_FEATURE_DEPENDENCY_TEST_NAMES = [
    "test_manifest_schema_rejects_optional_feature_dependency_non_table",
    "test_manifest_schema_rejects_optional_feature_missing_dependencies",
    "test_manifest_schema_rejects_optional_feature_empty_dependencies",
    "test_manifest_schema_rejects_optional_feature_dependency_missing_plugin_id",
    "test_manifest_schema_rejects_unknown_optional_feature_dependency_field",
    "test_manifest_schema_rejects_optional_feature_dependency_missing_primary",
    "test_manifest_schema_rejects_optional_feature_dependency_primary_type",
    "test_manifest_schema_rejects_optional_feature_without_primary_dependency",
    "test_manifest_schema_rejects_optional_feature_multiple_primary_dependencies",
    "test_manifest_schema_rejects_optional_feature_duplicate_dependency_rows",
    "test_manifest_schema_rejects_optional_feature_primary_plugin_mismatch",
    "test_manifest_schema_rejects_optional_feature_primary_capability_mismatch",
]

OPTIONAL_FEATURE_DISTRIBUTION_TEST_NAMES = [
    "test_manifest_schema_rejects_optional_feature_distribution_non_table",
    "test_manifest_schema_rejects_optional_feature_distribution_missing_abi",
    "test_manifest_schema_rejects_unknown_optional_feature_distribution_form",
    "test_manifest_schema_rejects_optional_feature_distribution_missing_entry",
]

FOCUSED_TEST_OWNERS = {
    OPTIONAL_FEATURE_SCHEMA_TEST: OPTIONAL_FEATURE_SCHEMA_TEST_NAMES,
    OPTIONAL_FEATURE_MODULE_TEST: OPTIONAL_FEATURE_MODULE_TEST_NAMES,
    OPTIONAL_FEATURE_DEPENDENCY_TEST: OPTIONAL_FEATURE_DEPENDENCY_TEST_NAMES,
    OPTIONAL_FEATURE_DISTRIBUTION_TEST: OPTIONAL_FEATURE_DISTRIBUTION_TEST_NAMES,
}

LINE_BUDGETS = {
    ROOT_TEST: 360,
    OPTIONAL_FEATURE_SCHEMA_TEST: 360,
    OPTIONAL_FEATURE_MODULE_TEST: 160,
    OPTIONAL_FEATURE_DEPENDENCY_TEST: 360,
    OPTIONAL_FEATURE_DISTRIBUTION_TEST: 180,
}


class PluginStructureAuditManifestSchemaOptionalFeatureTestOwnerBoundaryTests(
    unittest.TestCase
):
    def test_optional_feature_schema_tests_live_in_focused_owners(self):
        root_text = (REPO_ROOT / ROOT_TEST).read_text(encoding="utf-8")

        failures: list[str] = []
        for relative_path, test_names in FOCUSED_TEST_OWNERS.items():
            owner_path = REPO_ROOT / relative_path
            if not owner_path.exists():
                failures.append(f"{relative_path}: missing focused test owner")
                owner_text = ""
            else:
                owner_text = owner_path.read_text(encoding="utf-8")
            for test_name in test_names:
                if test_name in root_text:
                    failures.append(f"{ROOT_TEST}: {test_name}")
                if test_name not in owner_text:
                    failures.append(f"{relative_path}: missing {test_name}")
                for other_path, other_names in FOCUSED_TEST_OWNERS.items():
                    if other_path == relative_path or test_name not in other_names:
                        continue
                    other_file = REPO_ROOT / other_path
                    other_text = (
                        other_file.read_text(encoding="utf-8")
                        if other_file.exists()
                        else ""
                    )
                    if test_name in other_text:
                        failures.append(
                            f"{other_path}: {test_name} belongs in {relative_path}"
                        )

        if failures:
            self.fail(
                "Optional feature manifest schema tests crossed focused owner boundaries:\n"
                + "\n".join(failures)
            )

    def test_manifest_schema_optional_feature_test_owners_stay_under_line_budgets(self):
        failures: list[str] = []
        for relative_path, budget in LINE_BUDGETS.items():
            line_count = len(
                (REPO_ROOT / relative_path)
                .read_text(encoding="utf-8")
                .splitlines()
            )
            if line_count > budget:
                failures.append(f"{relative_path}: {line_count} > {budget}")

        if failures:
            self.fail(
                "Manifest schema optional feature test owners exceeded line budgets:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
