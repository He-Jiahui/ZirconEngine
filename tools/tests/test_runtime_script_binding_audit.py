import sys
import unittest
from unittest import mock
from pathlib import Path

# 调用审计或模型实现验证脚本绑定：当前子节点守卫归属方闭合运行时 13 审计。
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

        self.assertEqual(audit["expected_source_file_count"], 31)
        self.assertEqual(audit["expected_test_file_count"], 3)
        self.assertEqual(audit["expected_guard_file_count"], 8)
        self.assertEqual(audit["missing_guard_files"], [])
        self.assertEqual(audit["missing_runtime_13_guards"], [])
        self.assertTrue(audit["mirror_docs_guard_present"])
        self.assertEqual(audit["risks"], [])

    def test_current_audit_does_not_read_active_plans(self) -> None:
        from runtime_structure_audits import script_binding_boundary as boundary

        read_text = boundary._read_text

        def reject_plan_reads(path: Path) -> str:
            relative = path.relative_to(self.repo_root).as_posix()
            self.assertFalse(
                relative.startswith("docs/plans/"),
                f"current contract audit must not read active plans: {relative}",
            )
            return read_text(path)

        with mock.patch.object(boundary, "_read_text", side_effect=reject_plan_reads):
            audit = boundary.script_binding_boundary_audit(self.repo_root)

        self.assertEqual(audit["expected_source_file_count"], 31)
        self.assertEqual(audit["expected_test_file_count"], 3)
        self.assertEqual(audit["expected_guard_file_count"], 8)
        self.assertEqual(audit["fixed_host_function_count"], 74)
        self.assertEqual(audit["builtin_callback_count"], 21)
        self.assertEqual(audit["gameplay_callback_count"], 51)
        self.assertEqual(audit["host_capability_count"], 14)
        self.assertEqual(audit["missing_ledger_doc_anchors"], [])
        self.assertEqual(audit["missing_cargo_gate_anchors"], [])
        self.assertEqual(audit["risks"], [])

    def test_missing_canonical_pending_gate_cannot_use_another_mirror(self) -> None:
        from runtime_structure_audits import script_binding_boundary as boundary

        read_text = boundary._read_text
        ledger = self.repo_root / "docs/crates/zircon_runtime/script/vm/host/function_ledger.md"
        other_mirror = self.repo_root / "docs/architecture/runtime-architecture-review-m0.md"
        self.assertIn("code_static_pending_cargo", read_text(other_mirror))
        self.assertIn("code_static_pending_cargo", read_text(ledger))

        def remove_canonical_gate(path: Path) -> str:
            source = read_text(path)
            if path == ledger:
                return source.replace("code_static_pending_cargo", "removed_pending_gate")
            return source

        with mock.patch.object(boundary, "_read_text", side_effect=remove_canonical_gate):
            audit = boundary.script_binding_boundary_audit(self.repo_root)

        self.assertEqual(audit["missing_cargo_gate_anchors"], ["code_static_pending_cargo"])
        self.assertIn("Runtime 13 pending Cargo gate anchors are missing.", audit["risks"])


if __name__ == "__main__":
    unittest.main()
