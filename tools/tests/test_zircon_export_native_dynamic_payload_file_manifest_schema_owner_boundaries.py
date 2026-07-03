import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PAYLOAD_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_native_dynamic_payload_schema.py"
)
FILE_MANIFEST_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_file_manifest_schema.py"
)


class ZirconExportNativeDynamicPayloadFileManifestSchemaOwnerBoundaryTests(
    unittest.TestCase
):
    def test_file_manifest_schema_helpers_live_in_leaf_owner(self):
        self.assertTrue(
            FILE_MANIFEST_SCHEMA.exists(),
            "NativeDynamic payload file_manifest schema needs a dedicated owner",
        )
        payload_schema_text = PAYLOAD_SCHEMA.read_text(encoding="utf-8")
        file_manifest_schema_text = FILE_MANIFEST_SCHEMA.read_text(encoding="utf-8")

        for symbol in (
            "NATIVE_DYNAMIC_FILE_MANIFEST_FIELDS",
            "NATIVE_DYNAMIC_FILE_MANIFEST_STRING_FIELDS",
            "NATIVE_DYNAMIC_FILE_MANIFEST_INTEGER_FIELDS",
            "NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_STRING_FIELDS",
            "NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_INTEGER_FIELDS",
            "platform_bundle_native_plugins_payload_file_manifest_schema_diagnostics",
            "native_dynamic_file_manifest_schema_diagnostics",
        ):
            self.assertNotIn(
                f"{symbol} =",
                payload_schema_text,
                f"{symbol} belongs in the payload file_manifest schema owner",
            )
            self.assertNotIn(
                f"def {symbol}(",
                payload_schema_text,
                f"{symbol} belongs in the payload file_manifest schema owner",
            )
            self.assertTrue(
                f"{symbol} =" in file_manifest_schema_text
                or f"def {symbol}(" in file_manifest_schema_text,
                f"{symbol} should be defined by the file_manifest schema owner",
            )

        self.assertIn(
            "from .pipeline_report_native_dynamic_payload_file_manifest_schema import",
            payload_schema_text,
            "payload schema should consume the file_manifest schema owner",
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_payload_schema import",
            file_manifest_schema_text,
            "file_manifest schema owner must not import payload schema orchestration",
        )

    def test_payload_schema_owner_stays_narrow_after_file_manifest_split(self):
        line_count = len(PAYLOAD_SCHEMA.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            330,
            "NativeDynamic payload schema owner should stay below 330 lines after "
            "file_manifest schema split",
        )

        file_manifest_line_count = len(
            FILE_MANIFEST_SCHEMA.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            file_manifest_line_count,
            140,
            "NativeDynamic payload file_manifest schema owner should stay below "
            "140 lines",
        )


if __name__ == "__main__":
    unittest.main()
