from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
HIT_TEST = ROOT / "zircon_runtime/src/ui/tree/hit_test.rs"
GEOMETRY_PATCH = ROOT / "zircon_runtime/src/ui/tree/hit_test/geometry_patch.rs"
FRAME_HIT_TEST = ROOT / "zircon_runtime/src/ui/surface/frame_hit_test.rs"


class RuntimeUiHitGridCellIteratorContractTests(unittest.TestCase):
    def test_cell_projection_is_a_lazy_iterator(self) -> None:
        source = HIT_TEST.read_text(encoding="utf-8")
        body = source.split("pub(crate) fn bounded_cells_for_frame", 1)[1].split(
            "fn cell_count_for_frame", 1
        )[0]
        self.assertRegex(body, r"->\s*impl\s+Iterator\s*<\s*Item\s*=\s*usize\s*>")
        self.assertRegex(body, r"span\s*\.into_iter\(\)\s*\.flat_map")
        self.assertNotIn("Vec::with_capacity", body)
        self.assertNotIn("indices.push", body)

    def test_full_grid_builds_consume_cells_without_collecting_per_entry(self) -> None:
        root_source = HIT_TEST.read_text(encoding="utf-8")
        root_build = root_source.split("fn build_hit_grid", 1)[1].split(
            "fn cell_bounds_for_query", 1
        )[0]
        frame_source = FRAME_HIT_TEST.read_text(encoding="utf-8")
        projected_build = frame_source.split("fn build_projected_grid(", 1)[1].split(
            "fn frame_has_area", 1
        )[0]
        for build in (root_build, projected_build):
            self.assertIn("for cell_index in", build)
            self.assertIn("bounded_cells_for_frame", build)
            self.assertNotRegex(
                build,
                r"let\s+\w+\s*=\s*bounded_cells_for_frame[\s\S]*?collect",
            )

    def test_incremental_patch_explicitly_owns_the_changed_entry_cells(self) -> None:
        source = GEOMETRY_PATCH.read_text(encoding="utf-8")
        production = source.split("#[cfg(test)]", 1)[0]
        self.assertIn("bounded_cells_for_frame", production)
        self.assertRegex(
            production,
            r"bounded_cells_for_frame\([\s\S]*?\)\s*\.collect::<Vec<_>>()",
        )


if __name__ == "__main__":
    unittest.main()
