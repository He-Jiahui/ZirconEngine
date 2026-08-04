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

        self.assertEqual(audit["expected_guard_file_count"], 10)
        self.assertEqual(audit["missing_guard_files"], [])
        self.assertEqual(audit["missing_test_anchors"], [])
        self.assertEqual(audit["missing_behavior_test_anchors"], [])
        self.assertTrue(audit["mirror_docs_guard_present"])
        self.assertEqual(audit["missing_cargo_gate_anchors"], [])
        self.assertEqual(audit["risks"], [])

    def test_time_gate_uses_the_managed_validator_and_precise_module_filter(self) -> None:
        from runtime_structure_audits.schedule_frame_loop_anchor_inventory import (
            CARGO_GATE_ANCHORS,
        )

        precise = (
            r".\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 "
            "-Package zircon_runtime -SkipBuild -LibTests -TestFilter tests::time::"
        )
        broad = "-TestFilter time"

        self.assertIn(precise, CARGO_GATE_ANCHORS)
        self.assertFalse(any(broad in command for command in CARGO_GATE_ANCHORS))

if __name__ == "__main__":
    unittest.main()
