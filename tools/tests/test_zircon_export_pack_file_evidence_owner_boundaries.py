import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PACK_STAGE_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_pack_stage_schema.py"
)
PACK_FILE_EVIDENCE = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_pack_file_evidence.py"
)


class ZirconExportPackFileEvidenceOwnerBoundaryTests(unittest.TestCase):
    def test_pack_file_and_binary_evidence_live_in_dedicated_owner(self):
        self.assertTrue(
            PACK_FILE_EVIDENCE.exists(),
            "Pack file and binary evidence diagnostics need a dedicated owner",
        )
        stage_schema_text = PACK_STAGE_SCHEMA.read_text(encoding="utf-8")
        file_evidence_text = PACK_FILE_EVIDENCE.read_text(encoding="utf-8")

        for function_name in (
            "pack_report_binary_manifest_evidence_diagnostics",
            "pack_report_chunk_payload_hash_diagnostics",
            "pack_report_embedded_manifest",
            "pack_report_file_evidence_diagnostics",
            "pack_report_path_file_evidence_diagnostics",
            "zrpack_content_hash",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                stage_schema_text,
                f"{function_name} belongs in the Pack file evidence owner",
            )
            self.assertIn(f"def {function_name}(", file_evidence_text)

        self.assertIn(
            "from .pipeline_report_pack_file_evidence import",
            stage_schema_text,
            "Pack stage schema should consume the Pack file evidence owner",
        )
        self.assertNotIn(
            "from .pipeline_report_pack_stage_schema import",
            file_evidence_text,
            "Pack file evidence owner must not import the stage schema owner",
        )

    def test_pack_stage_schema_owner_stays_under_large_file_threshold(self):
        line_count = len(PACK_STAGE_SCHEMA.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            560,
            "Pack stage schema owner should stay below 560 lines after file evidence split",
        )

    def test_pack_file_evidence_owner_stays_leaf_sized(self):
        self.assertTrue(
            PACK_FILE_EVIDENCE.exists(),
            "Pack file evidence owner should exist before its size can be checked",
        )
        line_count = len(PACK_FILE_EVIDENCE.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            360,
            "Pack file evidence owner should stay below 360 lines",
        )


if __name__ == "__main__":
    unittest.main()
