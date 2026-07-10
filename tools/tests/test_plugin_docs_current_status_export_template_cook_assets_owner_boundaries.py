import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


REPO_ROOT = Path(__file__).resolve().parents[2]

EXPORT_TEMPLATE_STATUS_TEST = (
    "tools/tests/test_plugin_docs_current_status_export_template_cook_assets_owner_splits.py"
)
COOK_ASSETS_STATUS_TEST = (
    "tools/tests/test_plugin_docs_current_status_cook_assets_owner_splits.py"
)
STAGE_HANDOFF_STATUS_TEST = (
    "tools/tests/test_plugin_docs_current_status_stage_handoff_owner_splits.py"
)
SUPPORT_FILE = (
    "tools/tests/plugin_docs_current_status_export_template_cook_assets_support.py"
)

EXPORT_TEMPLATE_METHODS = [
    "test_current_export_plan_reflects_export_template_manifest_owner_split",
    "test_current_export_plan_reflects_cli_argument_owner_split",
    "test_current_export_plan_reflects_export_template_resolution_owner_split",
    "test_current_plugin_docs_reflect_schema_string_array_owner_split",
]

COOK_ASSETS_METHODS = [
    "test_current_export_plan_reflects_cook_assets_report_owner_split",
    "test_current_export_plan_reflects_cook_assets_manifest_owner_split",
    "test_current_export_plan_reflects_cook_assets_pack_trim_closure_owner_split",
    "test_current_export_plan_reflects_cook_assets_project_fallback_owner_split",
]

STAGE_HANDOFF_METHODS = [
    "test_current_export_plan_reflects_stage_handoff_strategy_owner_split",
]

FOCUSED_TEST_OWNERS = {
    EXPORT_TEMPLATE_STATUS_TEST: EXPORT_TEMPLATE_METHODS,
    COOK_ASSETS_STATUS_TEST: COOK_ASSETS_METHODS,
    STAGE_HANDOFF_STATUS_TEST: STAGE_HANDOFF_METHODS,
}

LINE_BUDGETS = {
    EXPORT_TEMPLATE_STATUS_TEST: 320,
    COOK_ASSETS_STATUS_TEST: 340,
    STAGE_HANDOFF_STATUS_TEST: 120,
    SUPPORT_FILE: 120,
}


class PluginDocsCurrentStatusExportTemplateCookAssetsOwnerBoundaryTests(
    unittest.TestCase
):
    def test_export_template_cook_assets_status_tests_live_in_focused_owners(self):
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
                "ExportTemplate/CookAssets current-status tests crossed focused owner boundaries:\n"
                + "\n".join(failures)
            )

    def test_export_template_cook_assets_status_owners_stay_under_line_budgets(self):
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
                "ExportTemplate/CookAssets current-status owners exceeded line budgets:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
