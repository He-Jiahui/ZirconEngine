import sys
import unittest
from pathlib import Path


class RuntimeScriptBindingAuditTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        sys.path.insert(
            0,
            str(
                self.repo_root
                / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
            ),
        )

    def test_current_child_guard_owners_close_runtime_13_audit(self) -> None:
        from runtime_structure_audits.script_binding_boundary import (
            script_binding_boundary_audit,
        )

        audit = script_binding_boundary_audit(self.repo_root)

        self.assertEqual(audit["expected_source_file_count"], 28)
        self.assertEqual(audit["expected_test_file_count"], 3)
        self.assertEqual(audit["expected_guard_file_count"], 8)
        self.assertEqual(audit["missing_guard_files"], [])
        self.assertEqual(audit["missing_runtime_13_guards"], [])
        self.assertTrue(audit["mirror_docs_guard_present"])
        self.assertEqual(audit["risks"], [])


if __name__ == "__main__":
    unittest.main()
