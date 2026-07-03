import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PAYLOAD_PLATFORM_BUNDLE = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_platform_bundle.py"
)
BUNDLE_EVIDENCE_OWNER = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_bundle_evidence.py"
)


class ZirconExportNativeDynamicPayloadBundleEvidenceOwnerBoundaryTests(
    unittest.TestCase
):
    def test_bundle_evidence_diagnostics_live_in_leaf_owner(self):
        self.assertTrue(
            BUNDLE_EVIDENCE_OWNER.exists(),
            "NativeDynamic payload bundle evidence diagnostics need a dedicated owner",
        )
        payload_platform_bundle_text = PAYLOAD_PLATFORM_BUNDLE.read_text(
            encoding="utf-8"
        )
        bundle_evidence_owner_text = BUNDLE_EVIDENCE_OWNER.read_text(encoding="utf-8")

        for symbol in (
            "platform_bundle_native_plugins_bundle_path_diagnostics",
            "platform_bundle_native_plugins_current_bundle_evidence_diagnostics",
        ):
            self.assertNotIn(
                f"def {symbol}(",
                payload_platform_bundle_text,
                f"{symbol} belongs in the bundle evidence owner",
            )
            self.assertIn(
                f"def {symbol}(",
                bundle_evidence_owner_text,
                f"{symbol} should be defined by the bundle evidence owner",
            )

        for borrowed_symbol in (
            "native_dynamic_plugins_bundle_file_manifest",
            "native_dynamic_content_hash",
            "materialized_package_loadable_artifacts_match_manifest",
        ):
            self.assertNotIn(
                borrowed_symbol,
                payload_platform_bundle_text,
                f"{borrowed_symbol} should be used by the bundle evidence owner",
            )
            self.assertIn(
                borrowed_symbol,
                bundle_evidence_owner_text,
                f"{borrowed_symbol} should be consumed by the bundle evidence owner",
            )

        self.assertIn(
            "from .pipeline_report_native_dynamic_payload_bundle_evidence import",
            payload_platform_bundle_text,
            "payload PlatformBundle owner should consume the bundle evidence owner",
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_payload_platform_bundle import",
            bundle_evidence_owner_text,
            "bundle evidence owner must not import payload PlatformBundle orchestration",
        )

    def test_payload_platform_bundle_owner_stays_narrow_after_bundle_evidence_split(self):
        line_count = len(PAYLOAD_PLATFORM_BUNDLE.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            210,
            "NativeDynamic payload PlatformBundle owner should stay below 210 "
            "lines after bundle evidence split",
        )

        bundle_evidence_line_count = len(
            BUNDLE_EVIDENCE_OWNER.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            bundle_evidence_line_count,
            130,
            "NativeDynamic payload bundle evidence owner should stay below 130 lines",
        )


if __name__ == "__main__":
    unittest.main()
