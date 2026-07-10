import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CONVERGENCE_TEST = REPO_ROOT / "tools/tests/test_plugin_docs_current_status_convergence.py"
PACK_DOCS_TEST = REPO_ROOT / "tools/tests/test_plugin_docs_current_status_pack_owner_splits.py"


class PluginDocsCurrentStatusPackOwnerBoundaryTests(unittest.TestCase):
    def test_pack_docs_status_guards_live_in_dedicated_owner(self):
        self.assertTrue(
            PACK_DOCS_TEST.exists(),
            "Pack current-status docs guards must live in a dedicated owner.",
        )

        convergence = CONVERGENCE_TEST.read_text(encoding="utf-8")
        pack_owner = PACK_DOCS_TEST.read_text(encoding="utf-8")

        moved_markers = [
            "test_current_export_plan_reflects_pack_stage_owner_split",
            "test_current_export_plan_reflects_pack_stage_required_fields_owner_split",
            "test_current_export_plan_reflects_pack_file_evidence_owner_split",
            "test_current_export_plan_reflects_pack_manifest_schema_helper_owner_split",
            "test_current_export_plan_reflects_pack_delta_semantics_owner_split",
            "test_current_export_plan_reflects_pack_manifest_path_hash_schema_helper_owner_split",
            "test_current_plugin_docs_reflect_pack_stage_path_owner_split",
        ]
        for marker in moved_markers:
            self.assertNotIn(marker, convergence)
            self.assertIn(marker, pack_owner)

        self.assertLessEqual(
            len(convergence.splitlines()),
            3700,
            "Broad current-status convergence owner should keep shrinking.",
        )
        self.assertLessEqual(
            len(pack_owner.splitlines()),
            320,
            "Dedicated Pack current-status owner should stay focused.",
        )


if __name__ == "__main__":
    unittest.main()
