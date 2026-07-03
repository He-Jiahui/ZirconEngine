import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PAYLOAD = REPO_ROOT / "tools/zircon_export/pipeline_report_native_dynamic_payload.py"
PAYLOAD_PLATFORM_BUNDLE = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_platform_bundle.py"
)
PAYLOAD_LOADER_MANIFEST = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_loader_manifest.py"
)


class ZirconExportNativeDynamicPayloadLoaderManifestOwnerBoundaryTests(
    unittest.TestCase
):
    def test_payload_loader_manifest_helpers_live_in_dedicated_owner(self):
        self.assertTrue(
            PAYLOAD_LOADER_MANIFEST.exists(),
            "PlatformBundle NativeDynamic payload loader manifest diagnostics "
            "need a dedicated owner",
        )
        payload_text = PAYLOAD.read_text(encoding="utf-8")
        handoff_text = PAYLOAD_PLATFORM_BUNDLE.read_text(encoding="utf-8")
        loader_manifest_text = PAYLOAD_LOADER_MANIFEST.read_text(encoding="utf-8")

        for function_name in (
            "platform_bundle_native_plugins_loader_manifest_diagnostics",
            "platform_bundle_native_plugins_loader_manifest_package_diagnostics",
            "platform_bundle_native_plugins_loader_manifest_expected_plugins_by_id",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                payload_text,
                f"{function_name} belongs in the payload loader-manifest owner",
            )
            self.assertNotIn(
                f"def {function_name}(",
                handoff_text,
                f"{function_name} belongs in the payload loader-manifest owner",
            )
            self.assertIn(f"def {function_name}(", loader_manifest_text)

        self.assertIn(
            "from .pipeline_report_native_dynamic_payload_loader_manifest import",
            handoff_text,
            "PlatformBundle payload handoff should consume the payload loader-manifest owner",
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_payload import",
            loader_manifest_text,
            "payload loader-manifest owner must not import payload orchestration",
        )

    def test_payload_owner_stays_under_large_file_threshold(self):
        line_count = len(PAYLOAD.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            560,
            "NativeDynamic payload owner should stay below 560 lines after "
            "loader-manifest diagnostics split",
        )


if __name__ == "__main__":
    unittest.main()
