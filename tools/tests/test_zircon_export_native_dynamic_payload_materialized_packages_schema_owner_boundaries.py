import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PAYLOAD_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_native_dynamic_payload_schema.py"
)
MATERIALIZED_PACKAGES_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_materialized_packages_schema.py"
)


class ZirconExportNativeDynamicPayloadMaterializedPackagesSchemaOwnerBoundaryTests(
    unittest.TestCase
):
    def test_materialized_packages_schema_helpers_live_in_leaf_owner(self):
        self.assertTrue(
            MATERIALIZED_PACKAGES_SCHEMA.exists(),
            "NativeDynamic payload materialized_packages schema needs a dedicated owner",
        )
        payload_schema_text = PAYLOAD_SCHEMA.read_text(encoding="utf-8")
        materialized_packages_schema_text = MATERIALIZED_PACKAGES_SCHEMA.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_FIELDS",
            "NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_FIELDS",
            "NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_INTEGER_FIELDS",
            "NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_ARRAY_FIELDS",
            "NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_REQUIRED_STRING_FIELDS",
            "NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_REQUIRED_INTEGER_FIELDS",
            "NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_REQUIRED_STRING_ARRAY_FIELDS",
            "platform_bundle_native_plugins_payload_materialized_packages_schema_diagnostics",
            "native_dynamic_materialized_packages_schema_diagnostics",
        ):
            self.assertNotIn(
                f"{symbol} =",
                payload_schema_text,
                f"{symbol} belongs in the payload materialized_packages schema owner",
            )
            self.assertNotIn(
                f"def {symbol}(",
                payload_schema_text,
                f"{symbol} belongs in the payload materialized_packages schema owner",
            )
            self.assertTrue(
                f"{symbol} =" in materialized_packages_schema_text
                or f"def {symbol}(" in materialized_packages_schema_text,
                f"{symbol} should be defined by the materialized_packages schema owner",
            )

        self.assertIn(
            "from .pipeline_report_native_dynamic_payload_materialized_packages_schema import",
            payload_schema_text,
            "payload schema should consume the materialized_packages schema owner",
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_payload_schema import",
            materialized_packages_schema_text,
            "materialized_packages schema owner must not import payload schema orchestration",
        )

    def test_payload_schema_owner_stays_narrow_after_materialized_packages_split(self):
        line_count = len(PAYLOAD_SCHEMA.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            210,
            "NativeDynamic payload schema owner should stay below 210 lines after "
            "materialized_packages schema split",
        )

        materialized_packages_line_count = len(
            MATERIALIZED_PACKAGES_SCHEMA.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            materialized_packages_line_count,
            170,
            "NativeDynamic payload materialized_packages schema owner should stay "
            "below 170 lines",
        )


if __name__ == "__main__":
    unittest.main()
