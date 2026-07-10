import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


REPO_ROOT = Path(__file__).resolve().parents[2]

REPORT_OWNER_TEST = (
    "tools/tests/test_plugin_docs_current_status_native_dynamic_report_owner_splits.py"
)
REPORT_SCHEMA_OWNER_TEST = (
    "tools/tests/test_plugin_docs_current_status_native_dynamic_report_schema_owner_splits.py"
)
STAGE_REPORT_OWNER_TEST = (
    "tools/tests/test_plugin_docs_current_status_native_dynamic_stage_report_owner_splits.py"
)
SUPPORT_FILE = (
    "tools/tests/plugin_docs_current_status_native_dynamic_report_support.py"
)

REPORT_OWNER_METHODS = [
    "test_current_export_plan_reflects_native_dynamic_report_owner_splits",
]

REPORT_SCHEMA_OWNER_METHODS = [
    "test_current_export_plan_reflects_native_dynamic_package_report_schema_helper_owner_split",
    "test_current_export_plan_reflects_native_dynamic_build_execution_packages_schema_owner_split",
    "test_current_export_plan_reflects_native_dynamic_build_plan_schema_helper_owner_split",
    "test_current_export_plan_reflects_native_dynamic_operation_audit_stage_packages_owner_split",
    "test_current_export_plan_reflects_native_dynamic_build_plan_package_details_owner_split",
]

STAGE_REPORT_OWNER_METHODS = [
    "test_current_export_plan_reflects_native_dynamic_stage_loader_manifest_owner_split",
    "test_current_export_plan_reflects_native_dynamic_stage_package_report_owner_split",
    "test_current_export_plan_reflects_native_dynamic_stage_payload_finalize_owner_split",
    "test_current_export_plan_reflects_native_dynamic_stage_payload_operation_audit_owner_split",
]

FOCUSED_TEST_OWNERS = {
    REPORT_OWNER_TEST: REPORT_OWNER_METHODS,
    REPORT_SCHEMA_OWNER_TEST: REPORT_SCHEMA_OWNER_METHODS,
    STAGE_REPORT_OWNER_TEST: STAGE_REPORT_OWNER_METHODS,
}

LINE_BUDGETS = {
    REPORT_OWNER_TEST: 160,
    REPORT_SCHEMA_OWNER_TEST: 360,
    STAGE_REPORT_OWNER_TEST: 320,
    SUPPORT_FILE: 120,
}


class PluginDocsCurrentStatusNativeDynamicReportOwnerBoundaryTests(
    unittest.TestCase
):
    def test_native_dynamic_report_status_tests_live_in_focused_owners(self):
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
                "NativeDynamic report current-status tests crossed focused owner boundaries:\n"
                + "\n".join(failures)
            )

    def test_native_dynamic_report_status_owners_stay_under_line_budgets(self):
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
                "NativeDynamic report current-status owners exceeded line budgets:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
