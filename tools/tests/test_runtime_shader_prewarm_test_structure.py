import unittest
from pathlib import Path


class RuntimeShaderPrewarmTestStructureTests(unittest.TestCase):
    def test_shader_prewarm_tests_are_folder_backed(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        prewarm_root = (
            repo_root / "zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs"
        )
        worker_path = (
            repo_root
            / "zircon_runtime/src/graphics/shader/variant_cache/prewarm/worker.rs"
        )
        tests_path = (
            repo_root
            / "zircon_runtime/src/graphics/shader/variant_cache/prewarm/tests.rs"
        )
        combined_path = (
            repo_root
            / "zircon_runtime/src/graphics/shader/variant_cache/prewarm/tests/combined_validation_tests.rs"
        )

        prewarm = prewarm_root.read_text(encoding="utf-8")
        worker = worker_path.read_text(encoding="utf-8")
        tests = tests_path.read_text(encoding="utf-8")
        combined = combined_path.read_text(encoding="utf-8")

        for source in (prewarm, worker, tests, combined):
            self.assertLessEqual(len(source.splitlines()), 800)
        self.assertIn('#[path = "prewarm/tests.rs"]', prewarm)
        self.assertIn("mod tests;", prewarm)
        self.assertNotIn("mod tests {", prewarm)
        self.assertEqual(tests.count("#[test]"), 11)
        self.assertIn(
            '#[path = "tests/combined_validation_tests.rs"]', tests
        )
        self.assertIn("mod combined_validation_tests;", tests)
        self.assertIn("fn test_disk_key(", tests)
        for anchor in (
            "&source.source_hash",
            "&source.include_content_hashes",
            "&source.template_revision",
            "&source.naga_version",
            "&source.wgpu_version",
        ):
            self.assertIn(anchor, tests)
        self.assertEqual(combined.count("#[test]"), 2)
        self.assertIn("fn prewarm_shader_variants_to_disk_inner(", worker)

    def test_shader_prewarm_structure_guards_read_current_owners(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        budget_root = (
            repo_root
            / "zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly"
        )
        budget_sources = (budget_root / "sources.rs").read_text(encoding="utf-8")
        budget_assertions = (
            budget_root / "assembly_assertions/owner_budget.rs"
        ).read_text(encoding="utf-8")
        guard_root = (
            repo_root
            / "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget"
        )
        guard_expectations = {
            "shader_prewarm_cache_artifact_contract.rs": (
                "graphics/shader/variant_cache/prewarm/worker.rs",
                "graphics/shader/variant_cache/prewarm/tests.rs",
            ),
            "shader_prewarm_source_provenance_summary.rs": (
                "graphics/shader/variant_cache/prewarm/worker.rs",
                "graphics/shader/variant_cache/prewarm/tests.rs",
            ),
            "shader_prewarm_wgpu_module_validation.rs": (
                "graphics/shader/variant_cache/prewarm.rs",
                "graphics/shader/variant_cache/prewarm/worker.rs",
                "graphics/shader/variant_cache/prewarm/tests.rs",
            ),
            "shader_prewarm_wgpu_pipeline_validation.rs": (
                "graphics/shader/variant_cache/prewarm.rs",
                "graphics/shader/variant_cache/prewarm/worker.rs",
                "graphics/shader/variant_cache/prewarm/tests.rs",
            ),
            "shader_prewarm_wgpu_validation_report_summary.rs": (
                "graphics/shader/variant_cache/prewarm/worker.rs",
                "graphics/shader/variant_cache/prewarm/tests.rs",
            ),
        }

        for relative_path in (
            "graphics/shader/variant_cache/prewarm.rs",
            "graphics/shader/variant_cache/prewarm/worker.rs",
            "graphics/shader/variant_cache/prewarm/tests.rs",
            "graphics/shader/variant_cache/prewarm/tests/combined_validation_tests.rs",
        ):
            self.assertIn(relative_path, budget_sources)
            self.assertIn(relative_path, budget_assertions)

        for guard_name, expected_paths in guard_expectations.items():
            guard = (guard_root / guard_name).read_text(encoding="utf-8")
            for expected_path in expected_paths:
                self.assertIn(expected_path, guard)


if __name__ == "__main__":
    unittest.main()
