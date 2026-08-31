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

    def test_runtime_job_system_has_no_editor_dependency(self) -> None:
        from runtime_structure_audits.job_system_boundary import (
            job_system_boundary_audit,
        )

        audit = job_system_boundary_audit(self.repo_root)

        self.assertEqual(audit["runtime_editor_dependency_references"], [])

    def test_runtime_task_diagnostic_structure_is_current(self) -> None:
        from runtime_structure_audits.job_system_boundary import (
            job_system_boundary_audit,
        )

        audit = job_system_boundary_audit(self.repo_root)

        self.assertEqual(audit["expected_module_count"], 22)
        self.assertEqual(audit["behavior_test_anchor_count"], 73)
        self.assertEqual(audit["missing_modules"], [])
        self.assertEqual(audit["unexpected_modules"], [])
        self.assertEqual(audit["missing_mod_declarations"], [])
        self.assertEqual(audit["missing_public_surface"], [])
        self.assertEqual(audit["missing_api_snippets"], {})
        self.assertEqual(audit["forbidden_scheduler_owner_snippets"], [])
        self.assertEqual(audit["forbidden_graphics_owner_snippets"], [])
        self.assertEqual(audit["forbidden_navigation_owner_snippets"], [])
        self.assertEqual(audit["missing_navigation_owner_snippets"], {})
        self.assertEqual(audit["forbidden_platform_owner_snippets"], [])
        self.assertEqual(audit["forbidden_platform_adapter_owner_snippets"], [])
        self.assertEqual(audit["platform_default_constructor_references"], [])
        self.assertEqual(audit["missing_platform_owner_snippets"], {})
        self.assertEqual(audit["missing_behavior_test_anchors"], [])
        self.assertEqual(audit["oversized_modules"], [])
        self.assertTrue(audit["mirror_docs_guard_present"])

    def test_folder_backed_mirror_owner_closes_runtime_11_audit(self) -> None:
        from runtime_structure_audits.job_system_boundary import (
            job_system_boundary_audit,
        )

        audit = job_system_boundary_audit(self.repo_root)

        self.assertEqual(audit["expected_module_count"], 22)
        self.assertEqual(audit["expected_guard_file_count"], 2)
        self.assertEqual(audit["missing_guard_files"], [])
        self.assertEqual(audit["diagnostic_anchor_count"], 11)
        self.assertEqual(audit["behavior_test_anchor_count"], 73)
        self.assertEqual(audit["missing_behavior_test_anchors"], [])
        self.assertTrue(audit["mirror_docs_guard_present"])
        self.assertEqual(
            audit["direct_rayon_paths"], audit["expected_direct_rayon_paths"]
        )
        self.assertEqual(audit["unexpected_rayon_paths"], [])
        self.assertEqual(audit["unclassified_direct_rayon"], [])
        self.assertEqual(audit["runtime_editor_dependency_references"], [])
        self.assertEqual(audit["forbidden_scheduler_owner_snippets"], [])
        self.assertEqual(audit["forbidden_graphics_owner_snippets"], [])
        self.assertEqual(audit["forbidden_navigation_owner_snippets"], [])
        self.assertEqual(audit["missing_navigation_owner_snippets"], {})
        self.assertEqual(audit["forbidden_platform_owner_snippets"], [])
        self.assertEqual(audit["forbidden_platform_adapter_owner_snippets"], [])
        self.assertEqual(audit["platform_default_constructor_references"], [])
        self.assertEqual(audit["missing_platform_owner_snippets"], {})

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

    def test_offline_font_sdf_bake_consumes_an_explicit_task_owner(self) -> None:
        bake_source = (
            self.repo_root
            / "zircon_runtime/src/text/font_sdf_build_tool/bake.rs"
        ).read_text(encoding="utf-8")
        cli_source = (
            self.repo_root
            / "zircon_runtime/src/bin/zircon_font_sdf_bake/main.rs"
        ).read_text(encoding="utf-8")
        integration_test_source = (
            self.repo_root
            / "zircon_runtime/tests/runtime_text_sdf_offline_artifact.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("TaskPools", bake_source)
        self.assertIn("generation_pool: &TaskPool", bake_source)
        self.assertIn(
            "bake_font_sdf_artifact(task_graph.worker_pool(),", cli_source
        )
        self.assertIn("EngineTaskGraph::try_new", cli_source)
        self.assertIn("task_graph.shutdown(", cli_source)
        self.assertIn("EngineTaskGraph::try_new", integration_test_source)
        self.assertNotIn(
            "bake_font_sdf_artifact(&font,", integration_test_source
        )


if __name__ == "__main__":
    unittest.main()
