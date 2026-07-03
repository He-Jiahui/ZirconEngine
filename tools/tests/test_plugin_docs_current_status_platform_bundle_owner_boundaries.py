import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

MATERIALIZATION_STATUS_TEST = (
    "tools/tests/test_plugin_docs_current_status_platform_bundle_owner_splits.py"
)
REPORT_STATUS_TEST = (
    "tools/tests/test_plugin_docs_current_status_platform_bundle_report_owner_splits.py"
)
STRATEGY_STATUS_TEST = (
    "tools/tests/test_plugin_docs_current_status_platform_bundle_strategy_owner_splits.py"
)
SUPPORT_FILE = "tools/tests/plugin_docs_current_status_platform_bundle_support.py"

MATERIALIZATION_METHODS = [
    "test_current_export_plan_reflects_platform_bundle_materialize_owner_split",
    "test_current_export_plan_reflects_platform_bundle_native_plugins_payload_owner_split",
    "test_current_plugin_docs_reflect_platform_bundle_native_plugins_materialize_owner_split",
]

REPORT_METHODS = [
    "test_current_export_plan_reflects_platform_bundle_stage_handoff_report_owner_split",
    "test_current_export_plan_reflects_platform_bundle_file_evidence_owner_split",
    "test_current_plugin_docs_reflect_platform_bundle_report_payload_owner_split",
]

STRATEGY_METHODS = [
    "test_current_export_plan_reflects_platform_bundle_argument_path_owner_split",
    "test_current_export_plan_reflects_platform_bundle_strategy_handoff_owner_split",
]

FOCUSED_TEST_OWNERS = {
    MATERIALIZATION_STATUS_TEST: MATERIALIZATION_METHODS,
    REPORT_STATUS_TEST: REPORT_METHODS,
    STRATEGY_STATUS_TEST: STRATEGY_METHODS,
}

LINE_BUDGETS = {
    MATERIALIZATION_STATUS_TEST: 280,
    REPORT_STATUS_TEST: 280,
    STRATEGY_STATUS_TEST: 220,
    SUPPORT_FILE: 120,
}


class PluginDocsCurrentStatusPlatformBundleOwnerBoundaryTests(unittest.TestCase):
    def test_platform_bundle_status_tests_live_in_focused_owners(self):
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
                "PlatformBundle current-status tests crossed focused owner "
                "boundaries:\n"
                + "\n".join(failures)
            )

    def test_platform_bundle_status_owners_stay_under_line_budgets(self):
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
                "PlatformBundle current-status owners exceeded line budgets:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
