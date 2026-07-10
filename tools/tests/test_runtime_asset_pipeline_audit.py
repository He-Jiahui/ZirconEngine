import sys
import unittest
from pathlib import Path


class RuntimeAssetPipelineAuditTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        audit_scripts = (
            self.repo_root
            / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
        )
        sys.path.insert(0, str(audit_scripts))

    def test_current_child_guard_owners_close_the_runtime_04_audit(self) -> None:
        from runtime_structure_audits.asset_pipeline_boundary import (
            asset_pipeline_boundary_audit,
        )

        audit = asset_pipeline_boundary_audit(self.repo_root)

        self.assertEqual(audit["expected_source_file_count"], 22)
        self.assertEqual(audit["expected_guard_file_count"], 17)
        self.assertEqual(audit["missing_guard_files"], [])
        self.assertEqual(audit["missing_test_anchors"], [])
        self.assertEqual(audit["missing_behavior_test_anchors"], [])
        self.assertTrue(audit["mirror_docs_guard_present"])
        self.assertEqual(audit["risks"], [])


if __name__ == "__main__":
    unittest.main()
