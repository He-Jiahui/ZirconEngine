import unittest
from pathlib import Path


class RuntimeShaderModuleRegistryTestStructureTests(unittest.TestCase):
    def test_shader_module_registry_tests_are_folder_backed(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        registry_path = (
            repo_root
            / "zircon_runtime/src/graphics/shader/template/module_registry.rs"
        )
        tests_path = (
            repo_root
            / "zircon_runtime/src/graphics/shader/template/module_registry/tests.rs"
        )

        registry = registry_path.read_text(encoding="utf-8")
        tests = tests_path.read_text(encoding="utf-8")

        self.assertLessEqual(len(registry.splitlines()), 800)
        self.assertLessEqual(len(tests.splitlines()), 800)
        self.assertIn('#[path = "module_registry/tests.rs"]', registry)
        self.assertIn("mod tests;", registry)
        self.assertNotIn("mod tests {", registry)
        self.assertEqual(tests.count("#[test]"), 11)
        self.assertIn("builtin_pbr_extras_is_independent_from_volumetric_uv_helpers", tests)
        self.assertIn("PBR_COMMON_INCLUDE_TOKEN", tests)

    def test_shader_module_registry_budget_reads_test_owner(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        budget_root = (
            repo_root
            / "zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly"
        )
        sources = (budget_root / "sources.rs").read_text(encoding="utf-8")
        owner_budget = (
            budget_root / "assembly_assertions/owner_budget.rs"
        ).read_text(encoding="utf-8")
        tests_path = "graphics/shader/template/module_registry/tests.rs"

        self.assertIn(tests_path, sources)
        self.assertIn(tests_path, owner_budget)
        self.assertIn("module_registry_tests", sources)
        self.assertIn("module_registry_tests", owner_budget)


if __name__ == "__main__":
    unittest.main()
