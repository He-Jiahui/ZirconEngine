import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
NATIVE_DYNAMIC_PAYLOAD = REPO_ROOT / "tools/zircon_export/native_dynamic_payload.py"
NATIVE_DYNAMIC_PAYLOAD_DIRECTORY = (
    REPO_ROOT / "tools/zircon_export/native_dynamic_payload_directory.py"
)
NATIVE_DYNAMIC_PAYLOAD_PLATFORM_BUNDLE = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_platform_bundle.py"
)
NATIVE_DYNAMIC_PAYLOAD_BUNDLE_EVIDENCE = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_bundle_evidence.py"
)
NATIVE_DYNAMIC_STAGE_PAYLOAD = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_native_dynamic_stage_payload.py"
)


class ZirconExportNativeDynamicPayloadDirectoryOwnerBoundaryTests(
    unittest.TestCase
):
    def test_directory_payload_helpers_live_in_dedicated_owner(self):
        self.assertTrue(
            NATIVE_DYNAMIC_PAYLOAD_DIRECTORY.exists(),
            "NativeDynamic directory payload summary rules need a dedicated owner",
        )
        payload_text = NATIVE_DYNAMIC_PAYLOAD.read_text(encoding="utf-8")
        directory_text = NATIVE_DYNAMIC_PAYLOAD_DIRECTORY.read_text(encoding="utf-8")

        for function_name in (
            "native_dynamic_directory_payload_summary",
            "native_dynamic_directory_materialized_packages",
            "native_dynamic_payload_directory_children",
            "native_dynamic_package_report_id",
            "materialized_package_loadable_artifacts_match_manifest",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                payload_text,
                f"{function_name} belongs in the directory payload owner",
            )
            self.assertIn(
                f"def {function_name}(",
                directory_text,
            )

        self.assertIn(
            "from .native_dynamic_payload_directory import",
            payload_text,
            "payload summary should consume the directory payload owner",
        )
        self.assertNotIn(
            "from .native_dynamic_payload import",
            directory_text,
            "directory payload owner must not import the payload summary owner",
        )
        for consumer_path in (
            NATIVE_DYNAMIC_PAYLOAD_BUNDLE_EVIDENCE,
            NATIVE_DYNAMIC_STAGE_PAYLOAD,
        ):
            consumer_text = consumer_path.read_text(encoding="utf-8")
            self.assertIn(
                "from .native_dynamic_payload_directory import",
                consumer_text,
                f"{consumer_path.name} should consume directory payload helpers directly",
            )
        self.assertNotIn(
            "from .native_dynamic_payload_directory import",
            NATIVE_DYNAMIC_PAYLOAD_PLATFORM_BUNDLE.read_text(encoding="utf-8"),
            "PlatformBundle payload orchestration should delegate bundle evidence",
        )
        self.assertLess(
            len(payload_text.splitlines()),
            330,
            "NativeDynamic payload summary owner should stay below 330 lines after directory owner split",
        )
        self.assertLess(
            len(directory_text.splitlines()),
            230,
            "NativeDynamic directory payload owner should stay focused",
        )


if __name__ == "__main__":
    unittest.main()
