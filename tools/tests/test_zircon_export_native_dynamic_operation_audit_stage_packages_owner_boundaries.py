import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
OPERATION_AUDIT_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_operation_audit_schema.py"
)
STAGE_PACKAGES_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_operation_audit_stage_packages_schema.py"
)


class ZirconExportNativeDynamicOperationAuditStagePackagesOwnerTests(
    unittest.TestCase
):
    def test_stage_packages_schema_lives_in_dedicated_owner(self):
        self.assertTrue(
            STAGE_PACKAGES_SCHEMA.exists(),
            "NativeDynamic operation audit stage packages need a dedicated owner",
        )
        schema_text = OPERATION_AUDIT_SCHEMA.read_text(encoding="utf-8")
        stage_packages_text = STAGE_PACKAGES_SCHEMA.read_text(encoding="utf-8")

        for symbol in (
            "NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_FIELDS",
            "NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_FIELDS",
            "def native_dynamic_operation_audit_stage_packages_schema_diagnostics(",
        ):
            self.assertNotIn(
                symbol,
                schema_text,
                f"{symbol} belongs in the stage packages schema owner",
            )
            self.assertIn(symbol, stage_packages_text)

        self.assertIn(
            "from .pipeline_report_native_dynamic_operation_audit_stage_packages_schema import",
            schema_text,
            "operation audit schema owner should consume stage packages directly",
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_operation_audit_schema import",
            stage_packages_text,
            "stage packages owner must not import operation-audit orchestration",
        )

    def test_operation_audit_schema_owner_stays_under_large_file_threshold(self):
        line_count = len(OPERATION_AUDIT_SCHEMA.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            430,
            "Operation audit schema owner should stay below 430 lines "
            "after stage packages split",
        )


if __name__ == "__main__":
    unittest.main()
