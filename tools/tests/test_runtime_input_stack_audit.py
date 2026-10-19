import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

# 调用审计或模型实现验证输入栈：当前子节点守卫与文档归属方闭合运行时 12 审计。
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

        self.assertEqual(audit["expected_runtime_module_count"], 27)
        self.assertEqual(audit["expected_framework_module_count"], 44)
        self.assertEqual(audit["expected_test_module_count"], 7)
        self.assertEqual(audit["expected_guard_file_count"], 6)
        self.assertEqual(audit["missing_guard_files"], [])
        self.assertEqual(audit["public_surface_anchor_count"], 31)
        self.assertEqual(audit["missing_public_surface"], [])
        self.assertEqual(audit["missing_input_prelude_anchors"], [])
        self.assertEqual(audit["missing_crate_prelude_anchors"], [])
        self.assertEqual(audit["missing_axis_frame_index_anchors"], [])
        self.assertEqual(audit["missing_event_buffer_visibility_anchors"], [])
        self.assertEqual(audit["missing_runtime_12_guards"], [])
        self.assertEqual(audit["missing_doc_anchors"], [])
        self.assertEqual(audit["missing_cargo_gate_anchors"], [])
        self.assertEqual(audit["behavior_test_anchor_count"], 25)
        self.assertEqual(audit["missing_behavior_test_anchors"], [])
        self.assertTrue(audit["mirror_docs_guard_present"])
        self.assertEqual(audit["risks"], [])

    def test_fixed_runtime_12_history_reads_verified_archive_identity(self) -> None:
        from runtime_structure_audits.input_stack_boundary import (
            RUNTIME_12_CLOSEOUT_ARCHIVE,
            RUNTIME_12_HISTORICAL_IDENTITIES,
            input_stack_boundary_audit,
        )

        audit = input_stack_boundary_audit(self.repo_root)
        records = audit["historical_archive_readback"]
        self.assertEqual(1, len(records))
        record = records[0]
        self.assertEqual(RUNTIME_12_CLOSEOUT_ARCHIVE, record["path"])
        self.assertTrue(record["historical_only"])
        self.assertEqual("verified", record["status"])
        length, sha256 = RUNTIME_12_HISTORICAL_IDENTITIES[RUNTIME_12_CLOSEOUT_ARCHIVE]
        self.assertEqual(length, record["actual_length"])
        self.assertEqual(sha256, record["actual_sha256"])

    def test_archive_failure_is_not_masked_by_checkout_or_current_document_anchors(self) -> None:
        from runtime_structure_audits import input_stack_boundary as boundary

        current_doc = self.repo_root / "docs/crates/zircon_runtime/input/input_state.md"
        complete_current_doc = "\n".join(
            (*boundary.RUNTIME_12_DOC_ANCHORS, *boundary.CARGO_GATE_ANCHORS)
        )
        original_read = boundary._read_text

        def read_current(path: Path) -> str:
            return complete_current_doc if path == current_doc else original_read(path)

        with tempfile.TemporaryDirectory() as directory:
            archive_root = Path(directory)
            for tampered in (False, True):
                with self.subTest(tampered=tampered):
                    if tampered:
                        target = archive_root / boundary.RUNTIME_12_CLOSEOUT_ARCHIVE
                        target.parent.mkdir(parents=True)
                        target.write_bytes(b"tampered history")
                    with patch.object(boundary, "_read_text", side_effect=read_current):
                        audit = boundary.input_stack_boundary_audit(
                            self.repo_root, archive_root=archive_root
                        )
                    self.assertEqual([], audit["missing_doc_anchors"])
                    self.assertEqual([], audit["missing_cargo_gate_anchors"])
                    self.assertIn(
                        "Runtime 12 fixed historical closeout archive readback failed.",
                        audit["risks"],
                    )
                    record = audit["historical_archive_readback"][0]
                    self.assertTrue(record["historical_only"])
                    self.assertEqual("unavailable", record["status"])
                    self.assertEqual(
                        "length-mismatch" if tampered else "archive-read-error",
                        record["reason"],
                    )


if __name__ == "__main__":
    unittest.main()
