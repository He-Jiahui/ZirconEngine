import unittest
from pathlib import Path


class RuntimeUiTableModuleStructureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        self.table_root = (
            self.repo_root
            / "zircon_runtime/src/ui/surface/surface/default_interactions/table"
        )

    def test_table_mutation_behavior_is_owned_by_named_child_module(self) -> None:
        route_source = (self.table_root / "mod.rs").read_text(encoding="utf-8")
        mutation_path = self.table_root / "mutation.rs"
        self.assertTrue(mutation_path.is_file(), "table mutation child owner is missing")
        mutation_source = mutation_path.read_text(encoding="utf-8")

        self.assertIn("mod mutation;", route_source)
        mutation_methods = (
            "apply_table_column_widths_mutation",
            "apply_table_columns_width_mutation",
            "apply_table_sort_model_mutation",
            "apply_table_columns_sort_direction_mutation",
            "apply_table_rows_sort_mutation",
            "apply_table_mutation",
        )
        for method in mutation_methods:
            self.assertNotIn(f"fn {method}", route_source, method)
            self.assertIn(f"fn {method}", mutation_source, method)

        self.assertLess(len(route_source.splitlines()), 540)
        self.assertLess(len(mutation_source.splitlines()), 240)

    def test_table_mutation_owner_is_documented(self) -> None:
        module_doc = (
            self.repo_root / "docs/zircon_runtime/ui/surface/default_interactions.md"
        ).read_text(encoding="utf-8")
        acceptance_path = (
            self.repo_root / "tests/acceptance/runtime-ui-table-mutation-owner-split.md"
        )
        self.assertTrue(acceptance_path.is_file(), "table mutation acceptance is missing")
        acceptance = acceptance_path.read_text(encoding="utf-8")
        owner_path = (
            "zircon_runtime/src/ui/surface/surface/default_interactions/table/mutation.rs"
        )

        self.assertIn(owner_path, module_doc)
        self.assertIn(owner_path, acceptance)


if __name__ == "__main__":
    unittest.main()
