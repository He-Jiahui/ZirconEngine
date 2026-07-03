import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
VALIDATE_SCHEMA_ROOT = (
    REPO_ROOT / "tools/zircon_export/tests/test_pipeline_report_validate_schema.py"
)
COMPILE_HOST_PLAN_SCHEMA_OWNER = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_validate_compile_host_plan_schema.py"
)

COMPILE_HOST_PLAN_METHODS = (
    "test_report_stage_rejects_validate_compile_host_plan_non_object",
    "test_report_stage_rejects_validate_compile_host_plan_unknown_field",
    "test_report_stage_rejects_validate_compile_host_plan_string_fields_non_string",
    "test_report_stage_rejects_validate_compile_host_plan_release_non_bool",
    "test_report_stage_rejects_validate_compile_host_plan_string_array_fields_non_string_array",
)

ROOT_VALIDATE_SCHEMA_METHODS = (
    "test_report_stage_rejects_validate_missing_release_evidence_field",
    "test_report_stage_rejects_validate_profile_feature_owner_plugin_ids_invalid",
    "test_report_stage_rejects_validate_plan_summary_unknown_field",
)


class ZirconExportValidateSchemaTestOwnerBoundaryTests(unittest.TestCase):
    def test_compile_host_plan_schema_tests_live_in_dedicated_owner(self) -> None:
        root_text = VALIDATE_SCHEMA_ROOT.read_text(encoding="utf-8")
        compile_host_owner_text = COMPILE_HOST_PLAN_SCHEMA_OWNER.read_text(
            encoding="utf-8"
        )

        for method_name in COMPILE_HOST_PLAN_METHODS:
            self.assertNotIn(
                f"def {method_name}",
                root_text,
                f"{method_name} should not return to the broad Validate schema owner",
            )
            self.assertIn(
                f"def {method_name}",
                compile_host_owner_text,
                f"{method_name} belongs in the CompileHost plan schema owner",
            )

        for method_name in ROOT_VALIDATE_SCHEMA_METHODS:
            self.assertIn(
                f"def {method_name}",
                root_text,
                f"{method_name} should remain in the broad Validate schema owner",
            )
            self.assertNotIn(
                f"def {method_name}",
                compile_host_owner_text,
                f"{method_name} should not move to the CompileHost plan schema owner",
            )

    def test_validate_schema_test_owners_stay_under_line_budgets(self) -> None:
        budgets = {
            VALIDATE_SCHEMA_ROOT: 780,
            COMPILE_HOST_PLAN_SCHEMA_OWNER: 280,
        }
        failures: list[str] = []
        for path, budget in budgets.items():
            line_count = len(path.read_text(encoding="utf-8").splitlines())
            if line_count > budget:
                failures.append(f"{path.relative_to(REPO_ROOT)}: {line_count} > {budget}")

        if failures:
            self.fail(
                "Validate schema test owners exceeded line budgets:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
