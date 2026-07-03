import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PAYLOAD_PLATFORM_BUNDLE = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_platform_bundle.py"
)
PACKAGE_PATH_OWNER = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_package_path.py"
)


class ZirconExportNativeDynamicPayloadPackagePathOwnerBoundaryTests(
    unittest.TestCase
):
    def test_package_path_diagnostics_live_in_leaf_owner(self):
        self.assertTrue(
            PACKAGE_PATH_OWNER.exists(),
            "NativeDynamic payload package path diagnostics need a dedicated owner",
        )
        payload_platform_bundle_text = PAYLOAD_PLATFORM_BUNDLE.read_text(
            encoding="utf-8"
        )
        package_path_owner_text = PACKAGE_PATH_OWNER.read_text(encoding="utf-8")

        for symbol in (
            "_resolve_user_path",
            "_resolve_user_path_or_diagnostic",
            "platform_bundle_native_plugins_package_path_diagnostics",
        ):
            self.assertNotIn(
                f"def {symbol}(",
                payload_platform_bundle_text,
                f"{symbol} belongs in the package path owner",
            )
            self.assertIn(
                f"def {symbol}(",
                package_path_owner_text,
                f"{symbol} should be defined by the package path owner",
            )

        self.assertIn(
            "from .pipeline_report_native_dynamic_payload_package_path import",
            payload_platform_bundle_text,
            "payload PlatformBundle owner should consume the package path owner",
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_payload_platform_bundle import",
            package_path_owner_text,
            "package path owner must not import payload PlatformBundle orchestration",
        )

    def test_payload_platform_bundle_owner_stays_narrow_after_package_path_split(self):
        line_count = len(PAYLOAD_PLATFORM_BUNDLE.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            270,
            "NativeDynamic payload PlatformBundle owner should stay below 270 "
            "lines after package path split",
        )

        package_path_line_count = len(
            PACKAGE_PATH_OWNER.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            package_path_line_count,
            120,
            "NativeDynamic payload package path owner should stay below 120 lines",
        )


if __name__ == "__main__":
    unittest.main()
