import sys
import unittest
from pathlib import Path


class RuntimePlanStatusArchiveOwnershipTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        sys.path.insert(
            0,
            str(
                self.repo_root
                / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
            ),
        )

    def test_numbered_archives_own_concrete_runtime_status_records(self) -> None:
        from runtime_structure_audits.runtime_plan_status_boundary import (
            runtime_plan_status_boundary_audit,
        )

        audit = runtime_plan_status_boundary_audit(self.repo_root)

        self.assertEqual(audit["expected_support_file_count"], 84)
        self.assertEqual(audit["subplan_index_row_count"], 15)
        self.assertEqual(audit["problem_row_count"], 17)
        self.assertEqual(audit["backlog_row_count"], 7)
        self.assertEqual(audit["missing_backlog_gaps"], [])
        self.assertEqual(audit["status_table_gaps"], [])
        self.assertEqual(audit["in_progress_without_gate"], [])
        self.assertEqual(audit["missing_core_guard_anchors"], [])
        self.assertEqual(audit["last_refined_violations"], [])
        self.assertTrue(audit["runtime_03_module_doc_status_guard_present"])
        self.assertTrue(audit["runtime_07_scene_status_guard_present"])
        self.assertTrue(audit["runtime_07_owner_budget_status_guard_present"])
        self.assertTrue(audit["runtime_02_generated_status_guard_present"])
        self.assertTrue(audit["runtime_10_behavior_status_guard_present"])
        self.assertTrue(audit["cargo_attempt_status_guard_present"])
        self.assertEqual(audit["risks"], [])


if __name__ == "__main__":
    unittest.main()
