import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STAGE_PAYLOAD = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_native_dynamic_stage_payload.py"
)
STAGE_PAYLOAD_OPERATION_AUDIT = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_stage_payload_operation_audit.py"
)


class ZirconExportNativeDynamicStagePayloadOperationAuditOwnerBoundaryTests(
    unittest.TestCase
):
    def test_stage_payload_operation_audit_artifacts_live_in_dedicated_owner(self):
        self.assertTrue(
            STAGE_PAYLOAD_OPERATION_AUDIT.exists(),
            "NativeDynamic stage payload operation-audit artifact diagnostics "
            "need a dedicated owner",
        )
        stage_text = STAGE_PAYLOAD.read_text(encoding="utf-8")
        operation_audit_text = STAGE_PAYLOAD_OPERATION_AUDIT.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "materialized_package_relative_artifacts",
            "native_dynamic_audit_artifacts_are_schema_clean",
            "native_dynamic_operation_audit_artifact_diagnostics",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                stage_text,
                f"{function_name} belongs in the stage payload "
                "operation-audit owner",
            )
            self.assertIn(f"def {function_name}(", operation_audit_text)

        self.assertIn(
            "from .pipeline_report_native_dynamic_stage_payload_operation_audit import",
            stage_text,
            "stage payload diagnostics should consume the operation-audit owner",
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_stage_payload import",
            operation_audit_text,
            "operation-audit owner must not import stage payload orchestration",
        )

    def test_stage_payload_owner_stays_under_operation_audit_split_budget(self):
        stage_line_count = len(
            STAGE_PAYLOAD.read_text(encoding="utf-8").splitlines()
        )
        operation_audit_line_count = len(
            STAGE_PAYLOAD_OPERATION_AUDIT.read_text(encoding="utf-8").splitlines()
        )

        self.assertLess(
            stage_line_count,
            380,
            "NativeDynamic stage payload owner should stay below 380 lines "
            "after operation-audit artifact diagnostics split",
        )
        self.assertLess(
            operation_audit_line_count,
            180,
            "NativeDynamic stage payload operation-audit owner should stay "
            "small enough to remain a focused diagnostic owner",
        )


if __name__ == "__main__":
    unittest.main()
