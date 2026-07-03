import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PAYLOAD_SUMMARY = REPO_ROOT / "tools/zircon_export/native_dynamic_payload.py"
PAYLOAD_STAGE_REPORT = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_stage_report.py"
)
PAYLOAD_OPERATION_AUDIT = (
    REPO_ROOT / "tools/zircon_export/native_dynamic_payload_operation_audit.py"
)


class ZirconExportNativeDynamicPayloadSummaryOwnerBoundaryTests(unittest.TestCase):
    def test_payload_operation_audit_summary_helpers_live_in_dedicated_owner(self):
        self.assertTrue(
            PAYLOAD_OPERATION_AUDIT.exists(),
            "NativeDynamic payload operation-audit summary rules need a dedicated owner",
        )
        payload_text = PAYLOAD_SUMMARY.read_text(encoding="utf-8")
        stage_report_text = PAYLOAD_STAGE_REPORT.read_text(encoding="utf-8")
        operation_audit_text = PAYLOAD_OPERATION_AUDIT.read_text(encoding="utf-8")

        for function_name in (
            "normalized_native_dynamic_operation_audit",
            "normalized_native_dynamic_stage_operation_audit",
            "native_dynamic_operation_audit_is_consistent",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                payload_text,
                f"{function_name} belongs in the operation-audit summary owner",
            )
            self.assertIn(
                f"def {function_name}(",
                operation_audit_text,
            )

        self.assertIn(
            "from .native_dynamic_payload_operation_audit import",
            payload_text,
            "payload summary should consume the operation-audit summary owner",
        )
        self.assertIn(
            "from .native_dynamic_payload_operation_audit import",
            stage_report_text,
            "stage-report diagnostics should consume the operation-audit summary owner directly",
        )
        payload_import_tail = stage_report_text.partition(
            "from .native_dynamic_payload import ("
        )[2]
        payload_import_block = payload_import_tail.partition(")")[0]
        self.assertNotIn(
            "normalized_native_dynamic_operation_audit,",
            payload_import_block,
            "stage-report diagnostics must not borrow operation-audit helpers from native_dynamic_payload",
        )
        self.assertNotIn(
            "from .native_dynamic_payload import",
            operation_audit_text,
            "operation-audit summary owner must not import the payload summary owner",
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_payload_stage_report import",
            operation_audit_text,
            "operation-audit summary owner must not import stage-report diagnostics",
        )

    def test_payload_summary_owner_stays_under_large_file_threshold(self):
        line_count = len(PAYLOAD_SUMMARY.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            460,
            "NativeDynamic payload summary owner should stay below 460 lines after operation-audit summary split",
        )


if __name__ == "__main__":
    unittest.main()
