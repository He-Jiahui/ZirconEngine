import sys
import unittest
from pathlib import Path


class RuntimeJobSystemAuditTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        audit_scripts = (
            self.repo_root
            / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
        )
        sys.path.insert(0, str(audit_scripts))

    def test_folder_backed_mirror_owner_closes_runtime_11_audit(self) -> None:
        from runtime_structure_audits.job_system_boundary import (
            job_system_boundary_audit,
        )

        audit = job_system_boundary_audit(self.repo_root)

        self.assertEqual(audit["expected_module_count"], 9)
        self.assertEqual(audit["expected_guard_file_count"], 2)
        self.assertEqual(audit["missing_guard_files"], [])
        self.assertEqual(audit["behavior_test_anchor_count"], 13)
        self.assertEqual(audit["missing_behavior_test_anchors"], [])
        self.assertTrue(audit["mirror_docs_guard_present"])
        self.assertEqual(
            audit["direct_rayon_paths"], audit["expected_direct_rayon_paths"]
        )
        self.assertEqual(audit["unexpected_rayon_paths"], [])
        self.assertEqual(audit["unclassified_direct_rayon"], [])

        source_mipmap = (
            self.repo_root
            / "zircon_runtime/src/core/framework/render/environment/source_cubemap/mipmap.rs"
        ).read_text(encoding="utf-8")
        parallel_contract = (
            self.repo_root
            / "zircon_runtime/src/core/framework/tasks/parallel_slice_executor.rs"
        ).read_text(encoding="utf-8")
        runtime_parallel_for = (
            self.repo_root
            / "zircon_runtime/src/core/runtime/tasks/parallel_for.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("ParallelSliceExecutor", source_mipmap)
        self.assertIn(
            "source_cubemap_mips_from_base_with_parallel_executor", source_mipmap
        )
        self.assertNotIn("rayon::", source_mipmap)
        self.assertNotIn("use rayon", source_mipmap)
        self.assertIn("pub trait ParallelSliceExecutor", parallel_contract)
        self.assertIn("impl ParallelSliceExecutor for TaskPool", runtime_parallel_for)
        self.assertEqual(audit["risks"], [])


if __name__ == "__main__":
    unittest.main()
