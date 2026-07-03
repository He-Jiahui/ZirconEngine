import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLATFORM_BUNDLE = REPO_ROOT / "tools/zircon_export/platform_bundle.py"
PLATFORM_BUNDLE_REPORT_PAYLOAD = (
    REPO_ROOT / "tools/zircon_export/platform_bundle_report_payload.py"
)


class ZirconExportPlatformBundleReportPayloadOwnerBoundaryTests(unittest.TestCase):
    def test_platform_bundle_report_payload_lives_in_payload_owner(self):
        self.assertTrue(
            PLATFORM_BUNDLE_REPORT_PAYLOAD.exists(),
            "PlatformBundle bundle manifest/report payload assembly needs a dedicated owner",
        )
        platform_bundle_text = PLATFORM_BUNDLE.read_text(encoding="utf-8")
        payload_text = PLATFORM_BUNDLE_REPORT_PAYLOAD.read_text(encoding="utf-8")

        for function_name in (
            "platform_bundle_stage_directory_failure_report",
            "platform_bundle_manifest_payload",
            "platform_bundle_stage_report_payload",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                platform_bundle_text,
                f"{function_name} belongs in the PlatformBundle report payload owner",
            )
            self.assertIn(f"def {function_name}(", payload_text)

        for inline_payload_marker in (
            "manifest = {",
            "report = {",
        ):
            self.assertNotIn(
                inline_payload_marker,
                platform_bundle_text,
                "PlatformBundle stage orchestration should not own payload dictionaries",
            )
            self.assertIn(inline_payload_marker, payload_text)

        self.assertIn(
            "from .platform_bundle_report_payload import",
            platform_bundle_text,
            "PlatformBundle orchestration should consume the report payload owner",
        )
        self.assertNotIn(
            "from .platform_bundle import",
            payload_text,
            "report payload owner must not import PlatformBundle orchestration",
        )

    def test_platform_bundle_report_payload_line_budgets(self):
        self.assertTrue(
            PLATFORM_BUNDLE_REPORT_PAYLOAD.exists(),
            "PlatformBundle report payload owner should exist before line-budget checks",
        )
        platform_bundle_line_count = len(
            PLATFORM_BUNDLE.read_text(encoding="utf-8").splitlines()
        )
        payload_line_count = len(
            PLATFORM_BUNDLE_REPORT_PAYLOAD.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            platform_bundle_line_count,
            400,
            "PlatformBundle orchestration should stay below 400 lines after report payload split",
        )
        self.assertLess(
            payload_line_count,
            180,
            "PlatformBundle report payload owner should stay below 180 lines",
        )


if __name__ == "__main__":
    unittest.main()
