import sys
import unittest
from pathlib import Path


class RuntimeInputStackAuditTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        audit_scripts = (
            self.repo_root
            / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
        )
        sys.path.insert(0, str(audit_scripts))

    def test_current_child_guard_and_doc_owners_close_runtime_12_audit(self) -> None:
        from runtime_structure_audits.input_stack_boundary import (
            input_stack_boundary_audit,
        )

        audit = input_stack_boundary_audit(self.repo_root)

        self.assertEqual(audit["expected_runtime_module_count"], 12)
        self.assertEqual(audit["expected_framework_module_count"], 20)
        self.assertEqual(audit["expected_test_module_count"], 7)
        self.assertEqual(audit["expected_guard_file_count"], 6)
        self.assertEqual(audit["missing_guard_files"], [])
        self.assertEqual(audit["missing_runtime_12_guards"], [])
        self.assertEqual(audit["missing_doc_anchors"], [])
        self.assertEqual(audit["missing_cargo_gate_anchors"], [])
        self.assertEqual(audit["missing_behavior_test_anchors"], [])
        self.assertTrue(audit["mirror_docs_guard_present"])
        self.assertEqual(audit["risks"], [])


if __name__ == "__main__":
    unittest.main()
