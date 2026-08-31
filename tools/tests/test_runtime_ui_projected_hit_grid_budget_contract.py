import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
FRAME_HIT_TEST = (
    ROOT / "zircon_runtime/src/ui/surface/frame_hit_test.rs"
)
TREE_HIT_TEST = ROOT / "zircon_runtime/src/ui/tree/hit_test.rs"
TREE_GEOMETRY_PATCH = (
    ROOT / "zircon_runtime/src/ui/tree/hit_test/geometry_patch.rs"
)
TREE_MOD = ROOT / "zircon_runtime/src/ui/tree/mod.rs"


def function_body(source: str, signature: str, next_signature: str) -> str:
    return source.split(signature, 1)[1].split(next_signature, 1)[0]


class ProjectedHitGridBudgetContractTests(unittest.TestCase):
    def test_projected_grid_reuses_the_base_grid_budget_authority(self) -> None:
        source = FRAME_HIT_TEST.read_text(encoding="utf-8")
        body = function_body(
            source,
            "fn build_projected_grid(",
            "fn frame_has_area(",
        )

        self.assertIn("bounded_hit_grid_dimensions", body)
        self.assertIn("bounded_cells_for_frame", body)
        self.assertIn("checked_mul(rows as usize)", body)
        self.assertNotIn("(columns * rows) as usize", body)
        self.assertNotIn("projected_cells_for_frame", body)

    def test_projected_geometry_uses_the_shared_finite_guard(self) -> None:
        source = FRAME_HIT_TEST.read_text(encoding="utf-8")
        body = function_body(
            source,
            "fn frame_has_area(",
            "fn frame_is_contained(",
        )

        self.assertIn("frame_is_finite_positive(frame)", body)

    def test_tree_module_exports_one_shared_hit_grid_budget_surface(self) -> None:
        tree_source = TREE_HIT_TEST.read_text(encoding="utf-8")
        module_source = TREE_MOD.read_text(encoding="utf-8")

        self.assertIn("pub(crate) fn bounded_hit_grid_dimensions", tree_source)
        self.assertIn("pub(crate) fn bounded_cells_for_frame", tree_source)
        self.assertIn("pub(crate) fn frame_is_finite_positive", tree_source)
        for symbol in (
            "bounded_hit_grid_dimensions",
            "bounded_cells_for_frame",
            "frame_is_finite_positive",
            "hit_grid_capacity_bounds",
        ):
            self.assertIn(symbol, module_source)

    def test_base_geometry_patch_imports_the_shared_cell_mapper(self) -> None:
        source = TREE_GEOMETRY_PATCH.read_text(encoding="utf-8")

        self.assertIn("bounded_cells_for_frame, entry_sort_key", source)
        self.assertNotIn("    cells_for_frame, entry_sort_key", source)

    def test_shared_dimension_policy_coarsens_without_collapsing_the_grid(self) -> None:
        source = TREE_HIT_TEST.read_text(encoding="utf-8")
        body = function_body(
            source,
            "pub(crate) fn bounded_hit_grid_dimensions(",
            "fn cell_bounds_for_query(",
        )

        self.assertIn("ui.hit_grid.adaptive_coarsening_count", body)
        self.assertIn("coarsened_cell_size", body)
        self.assertNotIn("return (1, 1", body)


if __name__ == "__main__":
    unittest.main()
