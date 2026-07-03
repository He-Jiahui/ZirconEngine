import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

SOURCE_TEMPLATE_STATUS_TEST = (
    "tools/tests/test_plugin_docs_current_status_source_template_compile_host_owner_splits.py"
)
COMPILE_HOST_STATUS_TEST = (
    "tools/tests/test_plugin_docs_current_status_compile_host_owner_splits.py"
)
SUPPORT_FILE = (
    "tools/tests/plugin_docs_current_status_source_template_compile_host_support.py"
)

SOURCE_TEMPLATE_METHODS = [
    "test_current_export_plan_reflects_source_template_generated_files_owner_split",
    "test_current_export_plan_reflects_source_template_build_handoff_owner_split",
    "test_current_export_plan_reflects_source_template_generated_project_owner_split",
    "test_current_plugin_docs_reflect_source_template_plan_command_owner_split",
]

COMPILE_HOST_METHODS = [
    "test_current_export_plan_reflects_compile_host_plan_owner_split",
    "test_current_export_plan_reflects_pipeline_report_compile_host_owner_split",
    "test_current_export_plan_reflects_validate_compile_host_semantics_owner_split",
    "test_current_export_plan_reflects_validate_compile_host_command_semantics_owner_split",
    "test_current_export_plan_reflects_compile_host_plan_command_semantics_owner_split",
]

FOCUSED_TEST_OWNERS = {
    SOURCE_TEMPLATE_STATUS_TEST: SOURCE_TEMPLATE_METHODS,
    COMPILE_HOST_STATUS_TEST: COMPILE_HOST_METHODS,
}

LINE_BUDGETS = {
    SOURCE_TEMPLATE_STATUS_TEST: 340,
    COMPILE_HOST_STATUS_TEST: 380,
    SUPPORT_FILE: 120,
}


class PluginDocsCurrentStatusSourceTemplateCompileHostOwnerBoundaryTests(
    unittest.TestCase
):
    def test_source_template_compile_host_status_tests_live_in_focused_owners(self):
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
                "SourceTemplate/CompileHost current-status tests crossed focused "
                "owner boundaries:\n"
                + "\n".join(failures)
            )

    def test_source_template_compile_host_status_owners_stay_under_line_budgets(self):
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
                "SourceTemplate/CompileHost current-status owners exceeded line "
                "budgets:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
