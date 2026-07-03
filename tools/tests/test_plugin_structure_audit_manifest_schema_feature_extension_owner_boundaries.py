import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

ROOT_TEST = "tools/tests/test_plugin_structure_audit_manifest_schema.py"
FEATURE_EXTENSION_TEST = (
    "tools/tests/test_plugin_structure_audit_manifest_schema_feature_extensions.py"
)
FEATURE_EXTENSION_MODULE_TEST = (
    "tools/tests/test_plugin_structure_audit_manifest_schema_feature_extension_modules.py"
)
FEATURE_EXTENSION_DEPENDENCY_TEST = (
    "tools/tests/test_plugin_structure_audit_manifest_schema_feature_extension_dependencies.py"
)
FEATURE_EXTENSION_DISTRIBUTION_TEST = (
    "tools/tests/test_plugin_structure_audit_manifest_schema_feature_extension_distribution.py"
)

FEATURE_EXTENSION_SCHEMA_TEST_NAMES = [
    "test_manifest_schema_rejects_unknown_feature_extension_field",
    "test_manifest_schema_rejects_empty_feature_extensions_array",
    "test_manifest_schema_rejects_feature_extension_non_table",
    "test_manifest_schema_rejects_feature_extension_missing_required_field",
    "test_manifest_schema_rejects_feature_extension_identity_semantics",
    "test_manifest_schema_rejects_feature_extension_id_namespace_segments",
    "test_manifest_schema_rejects_duplicate_feature_extension_id",
    "test_manifest_schema_rejects_feature_extension_capability_semantics",
]

FEATURE_EXTENSION_MODULE_TEST_NAMES = [
    "test_manifest_schema_rejects_feature_extension_empty_modules",
    "test_manifest_schema_rejects_feature_extension_module_missing_field",
]

FEATURE_EXTENSION_DEPENDENCY_TEST_NAMES = [
    "test_manifest_schema_rejects_feature_extension_missing_dependencies",
    "test_manifest_schema_rejects_feature_extension_empty_dependencies",
    "test_manifest_schema_rejects_feature_extension_dependency_non_table",
    "test_manifest_schema_rejects_feature_extension_dependency_missing_plugin_id",
    "test_manifest_schema_rejects_unknown_feature_extension_dependency_field",
    "test_manifest_schema_rejects_feature_extension_without_primary_dependency",
    "test_manifest_schema_rejects_feature_extension_multiple_primary_dependencies",
    "test_manifest_schema_rejects_feature_extension_duplicate_dependency_rows",
    "test_manifest_schema_rejects_feature_extension_primary_owner_mismatch",
    "test_manifest_schema_rejects_feature_extension_primary_capability_mismatch",
]

FEATURE_EXTENSION_DISTRIBUTION_TEST_NAMES = [
    "test_manifest_schema_rejects_feature_extension_distribution_non_table",
    "test_manifest_schema_rejects_feature_extension_distribution_missing_abi",
    "test_manifest_schema_rejects_feature_extension_distribution_missing_entry",
]

FOCUSED_TEST_OWNERS = {
    FEATURE_EXTENSION_TEST: FEATURE_EXTENSION_SCHEMA_TEST_NAMES,
    FEATURE_EXTENSION_MODULE_TEST: FEATURE_EXTENSION_MODULE_TEST_NAMES,
    FEATURE_EXTENSION_DEPENDENCY_TEST: FEATURE_EXTENSION_DEPENDENCY_TEST_NAMES,
    FEATURE_EXTENSION_DISTRIBUTION_TEST: FEATURE_EXTENSION_DISTRIBUTION_TEST_NAMES,
}

LINE_BUDGETS = {
    ROOT_TEST: 360,
    FEATURE_EXTENSION_TEST: 360,
    FEATURE_EXTENSION_MODULE_TEST: 140,
    FEATURE_EXTENSION_DEPENDENCY_TEST: 360,
    FEATURE_EXTENSION_DISTRIBUTION_TEST: 180,
}


class PluginStructureAuditManifestSchemaFeatureExtensionOwnerBoundaryTests(
    unittest.TestCase
):
    def test_feature_extension_schema_tests_live_in_focused_owners(self):
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
                "Feature extension manifest schema tests crossed focused owner boundaries:\n"
                + "\n".join(failures)
            )

    def test_manifest_schema_feature_extension_test_owners_stay_under_line_budgets(self):
        failures: list[str] = []
        for relative_path, budget in LINE_BUDGETS.items():
            line_count = len((REPO_ROOT / relative_path).read_text(encoding="utf-8").splitlines())
            if line_count > budget:
                failures.append(f"{relative_path}: {line_count} > {budget}")

        if failures:
            self.fail(
                "Manifest schema feature extension test owners exceeded line budgets:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
