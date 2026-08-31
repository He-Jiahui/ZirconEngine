import sys
import unittest
from pathlib import Path


class RuntimeEcsKernelDataAuditTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        audit_scripts = (
            self.repo_root
            / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
        )
        sys.path.insert(0, str(audit_scripts))

    def test_current_child_guard_owners_close_the_runtime_08_audit(self) -> None:
        from runtime_structure_audits.ecs_kernel_data_boundary import (
            ecs_kernel_data_boundary_audit,
        )

        audit = ecs_kernel_data_boundary_audit(self.repo_root)

        self.assertEqual(audit["expected_source_file_count"], 77)
        self.assertEqual(audit["expected_test_file_count"], 10)
        self.assertEqual(audit["missing_test_files"], [])
        self.assertEqual(audit["missing_test_anchors"], [])
        self.assertEqual(audit["missing_behavior_test_anchors"], [])
        self.assertTrue(audit["mirror_docs_guard_present"])
        self.assertEqual(audit["risks"], [])


if __name__ == "__main__":
    unittest.main()
