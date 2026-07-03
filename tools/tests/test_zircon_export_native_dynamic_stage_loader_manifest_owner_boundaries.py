import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
NATIVE_DYNAMIC_STAGE_PAYLOAD = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_native_dynamic_stage_payload.py"
)
NATIVE_DYNAMIC_STAGE_LOADER_MANIFEST = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_stage_loader_manifest.py"
)


class ZirconExportNativeDynamicStageLoaderManifestOwnerBoundaryTests(unittest.TestCase):
    def test_stage_loader_manifest_package_diagnostics_live_in_dedicated_owner(self):
        self.assertTrue(
            NATIVE_DYNAMIC_STAGE_LOADER_MANIFEST.exists(),
            "NativeDynamic stage loader manifest package diagnostics need a dedicated owner",
        )
        stage_payload_text = NATIVE_DYNAMIC_STAGE_PAYLOAD.read_text(encoding="utf-8")
        stage_loader_manifest_text = NATIVE_DYNAMIC_STAGE_LOADER_MANIFEST.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "native_dynamic_loader_manifest_package_diagnostics",
            "native_dynamic_loader_manifest_package_export_diagnostics",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                stage_payload_text,
                f"{function_name} belongs in the stage loader manifest owner",
            )
            self.assertIn(
                f"def {function_name}(",
                stage_loader_manifest_text,
            )

        self.assertIn(
            "from .pipeline_report_native_dynamic_stage_loader_manifest import",
            stage_payload_text,
            "stage payload diagnostics should consume the stage loader manifest owner",
        )
        self.assertIn(
            "from .pipeline_report_native_dynamic_loader_manifest import",
            stage_loader_manifest_text,
            "stage loader manifest owner should consume the parser/schema owner",
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_stage_payload import",
            stage_loader_manifest_text,
            "stage loader manifest diagnostics must not import the stage payload owner",
        )

    def test_native_dynamic_stage_payload_owner_stays_under_large_file_threshold(self):
        line_count = len(
            NATIVE_DYNAMIC_STAGE_PAYLOAD.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            line_count,
            700,
            "NativeDynamic stage payload owner should stay below 700 lines after loader manifest split",
        )


if __name__ == "__main__":
    unittest.main()
