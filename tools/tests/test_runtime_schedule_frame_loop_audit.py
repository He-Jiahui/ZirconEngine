import sys
import unittest
from pathlib import Path


class RuntimeScheduleFrameLoopAuditTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        audit_scripts = (
            self.repo_root
            / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
        )
        sys.path.insert(0, str(audit_scripts))

    def test_current_child_guard_owners_close_the_runtime_03_audit(self) -> None:
        from runtime_structure_audits.schedule_frame_loop_boundary import (
            schedule_frame_loop_boundary_audit,
        )

        audit = schedule_frame_loop_boundary_audit(self.repo_root)

        self.assertEqual(audit["expected_guard_file_count"], 11)
        self.assertEqual(audit["missing_guard_files"], [])
        self.assertEqual(audit["missing_test_anchors"], [])
        self.assertEqual(audit["missing_behavior_test_anchors"], [])
        self.assertTrue(audit["mirror_docs_guard_present"])
        self.assertEqual(audit["missing_cargo_gate_anchors"], [])
        self.assertEqual(audit["risks"], [])

    def test_time_cargo_gate_uses_the_precise_test_module_filter(self) -> None:
        from runtime_structure_audits.schedule_frame_loop_anchor_inventory import (
            CARGO_GATE_ANCHORS,
        )

        precise = "cargo test -p zircon_runtime --lib tests::time:: --locked"
        broad = "cargo test -p zircon_runtime --lib time --locked"

        self.assertIn(precise, CARGO_GATE_ANCHORS)
        self.assertNotIn(broad, CARGO_GATE_ANCHORS)

    def test_runtime_03_plan_status_anchors_follow_the_current_inventory(self) -> None:
        from runtime_structure_audits.runtime_plan_status_boundary import (
            runtime_plan_status_boundary_audit,
        )

        audit = runtime_plan_status_boundary_audit(self.repo_root)

        self.assertEqual(audit["missing_runtime_03_module_doc_status_index_anchors"], [])
        self.assertEqual(audit["missing_runtime_03_module_doc_status_guard_anchors"], [])
        self.assertEqual(audit["missing_runtime_03_module_doc_status_doc_anchors"], [])
        self.assertTrue(audit["runtime_03_module_doc_status_guard_present"])


if __name__ == "__main__":
    unittest.main()
