import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PIPELINE_REPORT = REPO_ROOT / "tools/zircon_export/pipeline_report.py"
COOK_ASSETS_REPORT = REPO_ROOT / "tools/zircon_export/pipeline_report_cook_assets.py"
COOK_ASSETS_MANIFEST_IO = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_cook_assets_manifest_io.py"
)
COOK_ASSETS_PACK_HANDOFF = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_cook_assets_pack_handoff.py"
)


class ZirconExportCookAssetsReportOwnerBoundaryTests(unittest.TestCase):
    def test_cook_assets_manifest_io_lives_in_manifest_io_owner(self):
        self.assertTrue(
            COOK_ASSETS_MANIFEST_IO.exists(),
            "CookAssets manifest IO/path helpers need a dedicated owner",
        )
        report_text = COOK_ASSETS_REPORT.read_text(encoding="utf-8")
        manifest_io_text = COOK_ASSETS_MANIFEST_IO.read_text(encoding="utf-8")

        for function_name in (
            "resolve_cook_assets_path_or_diagnostic",
            "cook_assets_manifest_path",
            "cook_assets_report_manifest_path",
            "cook_assets_manifest_json",
            "cook_assets_is_non_empty_trimmed_string",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                report_text,
                f"{function_name} belongs in the CookAssets manifest IO owner",
            )
            self.assertIn(f"def {function_name}(", manifest_io_text)

        self.assertIn(
            "from .pipeline_report_cook_assets_manifest_io import",
            report_text,
            "CookAssets report diagnostics should consume manifest IO owner",
        )
        self.assertNotIn(
            "from .pipeline_report_cook_assets import",
            manifest_io_text,
            "manifest IO owner must not import CookAssets report orchestration",
        )

    def test_cook_assets_pack_handoff_lives_in_pack_handoff_owner(self):
        self.assertTrue(
            COOK_ASSETS_PACK_HANDOFF.exists(),
            "CookAssets Pack handoff and trim-closure diagnostics need a dedicated owner",
        )
        pipeline_report_text = PIPELINE_REPORT.read_text(encoding="utf-8")
        report_text = COOK_ASSETS_REPORT.read_text(encoding="utf-8")
        pack_handoff_text = COOK_ASSETS_PACK_HANDOFF.read_text(encoding="utf-8")

        for function_name in (
            "cook_assets_pack_manifest_handoff_diagnostics",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                report_text,
                f"{function_name} belongs in the CookAssets Pack handoff owner",
            )
            self.assertIn(f"def {function_name}(", pack_handoff_text)

        self.assertIn(
            "from .pipeline_report_cook_assets_pack_handoff import",
            pipeline_report_text,
            "final report aggregation should consume Pack handoff owner directly",
        )
        self.assertNotIn(
            "from .pipeline_report_cook_assets import",
            pack_handoff_text,
            "Pack handoff owner must not import CookAssets report orchestration",
        )

    def test_cook_assets_report_orchestration_stays_under_large_file_threshold(self):
        line_count = len(COOK_ASSETS_REPORT.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            520,
            "CookAssets report orchestration should stay below the split threshold",
        )


if __name__ == "__main__":
    unittest.main()
