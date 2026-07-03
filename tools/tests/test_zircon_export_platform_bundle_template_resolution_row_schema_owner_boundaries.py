import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RESOLUTION_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_resolution_schema.py"
)
RESOLUTION_ROW_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_resolution_row_schema.py"
)


class ZirconExportPlatformBundleTemplateResolutionRowSchemaOwnerBoundaryTests(
    unittest.TestCase
):
    def test_resolution_candidate_row_schema_lives_in_dedicated_owner(self):
        self.assertTrue(
            RESOLUTION_ROW_SCHEMA.exists(),
            "PlatformBundle template resolution row schema needs a dedicated owner",
        )
        schema_text = RESOLUTION_SCHEMA.read_text(encoding="utf-8")
        row_schema_text = RESOLUTION_ROW_SCHEMA.read_text(encoding="utf-8")

        moved_symbols = (
            "PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_CANDIDATE_FIELDS",
            "PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_CANDIDATE_STRING_FIELDS",
            "PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_CANDIDATE_STRING_ARRAY_FIELDS",
            "PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_SKIPPED_CANDIDATE_FIELDS",
            "PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_SKIPPED_CANDIDATE_STRING_FIELDS",
            "PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_SKIPPED_CANDIDATE_STRING_ARRAY_FIELDS",
            "template_resolution_sequence_schema_diagnostics",
            "sequence_required_field_diagnostics",
        )
        for symbol in moved_symbols:
            self.assertNotIn(
                f"{symbol} =",
                schema_text,
                f"{symbol} belongs in the resolution row schema owner",
            )
            self.assertNotIn(
                f"def {symbol}(",
                schema_text,
                f"{symbol} belongs in the resolution row schema owner",
            )
            self.assertTrue(
                f"{symbol} =" in row_schema_text or f"def {symbol}(" in row_schema_text,
                f"{symbol} is missing from the resolution row schema owner",
            )

        self.assertIn(
            "from .pipeline_report_platform_bundle_template_resolution_row_schema import",
            schema_text,
            "resolution schema should consume the row schema owner directly",
        )
        self.assertNotIn(
            "from .pipeline_report_platform_bundle_template_resolution_schema import",
            row_schema_text,
            "row schema owner must not import schema orchestration",
        )

    def test_resolution_schema_and_row_owner_stay_small(self):
        schema_lines = len(RESOLUTION_SCHEMA.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            schema_lines,
            340,
            "template resolution schema owner should stay below 340 lines after row schema split",
        )
        self.assertTrue(RESOLUTION_ROW_SCHEMA.exists())
        row_schema_lines = len(
            RESOLUTION_ROW_SCHEMA.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            row_schema_lines,
            180,
            "template resolution row schema owner should stay below 180 lines",
        )


if __name__ == "__main__":
    unittest.main()
