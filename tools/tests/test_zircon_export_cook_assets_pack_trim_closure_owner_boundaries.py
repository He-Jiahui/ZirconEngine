import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PIPELINE_REPORT = REPO_ROOT / "tools/zircon_export/pipeline_report.py"
COOK_ASSETS_PACK_HANDOFF = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_cook_assets_pack_handoff.py"
)
COOK_ASSETS_PACK_TRIM_CLOSURE = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_cook_assets_pack_trim_closure.py"
)
COOK_ASSETS_TRIM_EVIDENCE = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_cook_assets_trim_evidence.py"
)


class ZirconExportCookAssetsPackTrimClosureOwnerBoundaryTests(unittest.TestCase):
    def test_pack_trim_closure_lives_in_trim_closure_owner(self):
        self.assertTrue(
            COOK_ASSETS_PACK_TRIM_CLOSURE.exists(),
            "CookAssets Pack trim closure diagnostics need a dedicated owner",
        )
        pipeline_report_text = PIPELINE_REPORT.read_text(encoding="utf-8")
        pack_handoff_text = COOK_ASSETS_PACK_HANDOFF.read_text(encoding="utf-8")
        trim_closure_text = COOK_ASSETS_PACK_TRIM_CLOSURE.read_text(encoding="utf-8")
        trim_evidence_text = COOK_ASSETS_TRIM_EVIDENCE.read_text(encoding="utf-8")

        trim_closure_function_names = (
            "cook_assets_pack_trim_closure_diagnostics",
            "cook_assets_manifest_assets_by_path",
            "cook_assets_pack_included_source_diagnostics",
            "normalized_pack_trimmed_assets",
            "normalized_pack_missing_dependencies",
            "normalized_pack_trim_reason",
        )
        for function_name in trim_closure_function_names:
            self.assertNotIn(
                f"def {function_name}(",
                pack_handoff_text,
                f"{function_name} belongs in the CookAssets Pack trim closure owner",
            )
            self.assertIn(f"def {function_name}(", trim_closure_text)

        trim_evidence_function_names = (
            "cook_assets_manifest_trim_evidence",
            "cook_assets_manifest_asset_path_is_schema_clean",
            "cook_assets_manifest_reachable_assets",
            "cook_assets_manifest_asset_matches_filter",
            "cook_assets_manifest_trim_reason",
            "cook_assets_manifest_trim_reason_label",
        )
        for function_name in trim_evidence_function_names:
            self.assertNotIn(
                f"def {function_name}(",
                pack_handoff_text,
                f"{function_name} belongs in the CookAssets trim evidence owner",
            )
            self.assertNotIn(
                f"def {function_name}(",
                trim_closure_text,
                f"{function_name} belongs in the CookAssets trim evidence owner",
            )
            self.assertIn(f"def {function_name}(", trim_evidence_text)

        self.assertIn(
            "from .pipeline_report_cook_assets_pack_trim_closure import",
            pipeline_report_text,
            "final report aggregation should consume the trim closure owner directly",
        )
        self.assertIn(
            "from .pipeline_report_cook_assets_trim_evidence import",
            trim_closure_text,
            "trim closure owner should consume manifest evidence reconstruction",
        )
        self.assertNotIn(
            "from .pipeline_report_cook_assets_pack_handoff import",
            trim_closure_text,
            "trim closure owner must not import Pack handoff orchestration",
        )

    def test_pack_handoff_owner_stays_focused_after_trim_split(self):
        pack_handoff_line_count = len(
            COOK_ASSETS_PACK_HANDOFF.read_text(encoding="utf-8").splitlines()
        )
        trim_closure_line_count = len(
            COOK_ASSETS_PACK_TRIM_CLOSURE.read_text(encoding="utf-8").splitlines()
        )
        trim_evidence_line_count = len(
            COOK_ASSETS_TRIM_EVIDENCE.read_text(encoding="utf-8").splitlines()
        )

        self.assertLess(
            pack_handoff_line_count,
            120,
            "Pack handoff owner should only keep asset_manifest drift checks",
        )
        self.assertLess(
            trim_closure_line_count,
            260,
            "Pack trim closure owner should stay below the local split budget",
        )
        self.assertLess(
            trim_evidence_line_count,
            240,
            "CookAssets trim evidence helper should stay below the local split budget",
        )


if __name__ == "__main__":
    unittest.main()
