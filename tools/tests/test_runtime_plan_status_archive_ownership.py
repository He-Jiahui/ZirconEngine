import sys
import unittest
from pathlib import Path


RUNTIME_15_PRIORITY_RECORDS = (
    "2026-07-09-code-structure-and-module-conventions-output-records.md",
    "2026-07-09-engine-code-review-findings-output-records.md",
    "2026-07-09-engine-code-structure-output-records.md",
    "2026-07-09-runtime-index-output-records.md",
)


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

    def test_runtime15_priority_records_have_one_archive_owner(self) -> None:
        active_root = self.repo_root / "docs/plans/zircon_runtime/runtime/15"
        archive_root = self.repo_root / "docs/plans/_archive/zircon_runtime/runtime/15"
        source_root = self.repo_root / "zircon_runtime/src/tests/runtime_absorption"
        rust_sources = "\n".join(
            path.read_text(encoding="utf-8")
            for path in sorted(source_root.rglob("*.rs"))
        )

        for record in RUNTIME_15_PRIORITY_RECORDS:
            self.assertFalse(
                (active_root / record).exists(),
                f"Runtime15 concrete output must not remain under the active child: {record}",
            )
            self.assertTrue(
                (archive_root / record).is_file(),
                f"Runtime15 canonical archive record is missing: {record}",
            )
            active_anchor = f"docs/plans/zircon_runtime/runtime/15/{record}"
            archive_anchor = f"docs/plans/_archive/zircon_runtime/runtime/15/{record}"
            self.assertNotIn(active_anchor, rust_sources)
            self.assertIn(archive_anchor, rust_sources)


if __name__ == "__main__":
    unittest.main()
