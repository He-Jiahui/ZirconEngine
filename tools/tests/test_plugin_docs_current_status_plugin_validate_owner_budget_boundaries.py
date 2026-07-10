import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


REPO_ROOT = Path(__file__).resolve().parents[2]

DISTRIBUTION_MODULES_STATUS_TEST = (
    "tools/tests/test_plugin_docs_current_status_plugin_validate_owner_splits.py"
)
FEATURE_PROVIDER_STATUS_TEST = (
    "tools/tests/test_plugin_docs_current_status_plugin_validate_feature_provider_status_owner_splits.py"
)
DISTRIBUTION_STATUS_TEST = (
    "tools/tests/test_plugin_docs_current_status_plugin_validate_distribution_status_owner_splits.py"
)
ENTRY_STATUS_TEST = (
    "tools/tests/test_plugin_docs_current_status_plugin_validate_entry_status_owner_splits.py"
)
SUPPORT_FILE = "tools/tests/plugin_docs_current_status_plugin_validate_support.py"

DISTRIBUTION_MODULES_METHODS = [
    "test_current_plugin_docs_reflect_distribution_modules_test_owner",
]

FEATURE_PROVIDER_METHODS = [
    "test_current_export_plan_reflects_feature_provider_projection_compare_owner_split",
    "test_current_plugin_docs_reflect_feature_provider_dependencies_owner_split",
    "test_current_plugin_docs_reflect_projection_optional_owner_split",
    "test_current_plugin_docs_reflect_feature_provider_capabilities_owner_split",
    "test_current_plugin_docs_reflect_feature_provider_distribution_owner_split",
    "test_current_plugin_docs_reflect_feature_provider_extension_owner_split",
]

DISTRIBUTION_METHODS = [
    "test_current_plugin_docs_reflect_dist_crate_dependency_owner_split",
    "test_current_plugin_docs_reflect_distribution_assets_owner_split",
    "test_current_plugin_docs_reflect_distribution_engine_compat_owner_split",
    "test_current_plugin_docs_reflect_distribution_descriptor_symbol_owner_split",
    "test_current_plugin_docs_reflect_distribution_entries_owner_split",
    "test_current_plugin_docs_reflect_distribution_packaging_owner_split",
    "test_current_plugin_docs_reflect_distribution_scalars_owner_split",
    "test_current_plugin_docs_reflect_distribution_module_target_modes_owner_split",
]

ENTRY_METHODS = [
    "test_current_plugin_docs_reflect_plugin_validate_single_target_owner_split",
]

FOCUSED_TEST_OWNERS = {
    DISTRIBUTION_MODULES_STATUS_TEST: DISTRIBUTION_MODULES_METHODS,
    FEATURE_PROVIDER_STATUS_TEST: FEATURE_PROVIDER_METHODS,
    DISTRIBUTION_STATUS_TEST: DISTRIBUTION_METHODS,
    ENTRY_STATUS_TEST: ENTRY_METHODS,
}

LINE_BUDGETS = {
    DISTRIBUTION_MODULES_STATUS_TEST: 140,
    FEATURE_PROVIDER_STATUS_TEST: 320,
    DISTRIBUTION_STATUS_TEST: 420,
    ENTRY_STATUS_TEST: 100,
    SUPPORT_FILE: 180,
}


class PluginDocsCurrentStatusPluginValidateOwnerBudgetBoundaryTests(
    unittest.TestCase
):
    def test_plugin_validate_status_tests_live_in_focused_owners(self):
        failures: list[str] = []
        owner_text_by_path: dict[str, str] = {}
        for relative_path in FOCUSED_TEST_OWNERS:
            owner_path = REPO_ROOT / relative_path
            if not owner_path.exists():
                failures.append(f"{relative_path}: missing focused test owner")
                owner_text_by_path[relative_path] = ""
            else:
                owner_text_by_path[relative_path] = owner_path.read_text(
                    encoding="utf-8"
                )

        for relative_path, method_names in FOCUSED_TEST_OWNERS.items():
            owner_text = owner_text_by_path[relative_path]
            for method_name in method_names:
                if f"def {method_name}" not in owner_text:
                    failures.append(f"{relative_path}: missing {method_name}")
                for other_path, other_text in owner_text_by_path.items():
                    if other_path == relative_path:
                        continue
                    if f"def {method_name}" in other_text:
                        failures.append(
                            f"{other_path}: {method_name} belongs in {relative_path}"
                        )

        if failures:
            self.fail(
                "PluginValidate current-status tests crossed focused owner boundaries:\n"
                + "\n".join(failures)
            )

    def test_plugin_validate_status_owners_stay_under_line_budgets(self):
        failures: list[str] = []
        for relative_path, budget in LINE_BUDGETS.items():
            owner_path = REPO_ROOT / relative_path
            if not owner_path.exists():
                failures.append(f"{relative_path}: missing owner file")
                continue
            line_count = len(owner_path.read_text(encoding="utf-8").splitlines())
            if line_count > budget:
                failures.append(f"{relative_path}: {line_count} > {budget}")

        if failures:
            self.fail(
                "PluginValidate current-status owners exceeded line budgets:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
