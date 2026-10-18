import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/ui/template/asset/surface_index.rs"
LOWER = ROOT / "zircon_runtime/src/ui/tests/asset_surface_index/target_capacity.rs"


class RuntimeSurfaceIndexTargetCapacityContractTests(unittest.TestCase):
    def test_surface_target_projection_reserves_all_category_inputs(self):
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("pub fn all_target_surfaces")
        end = source.index("impl UiAssetNodeHotReloadTargets", start)
        projection = " ".join(source[start:end].split())

        self.assertRegex(
            projection,
            r"let total_capacity = self\s*\.\s*template_rebuild_surfaces\s*\.\s*len\(\)",
        )
        self.assertIn(".saturating_add(self.removed_compiled_surfaces.len())", projection)
        self.assertIn(".saturating_add(self.theme_restyle_surfaces.len())", projection)
        self.assertIn(".saturating_add(self.resource_damage_surfaces.len())", projection)
        self.assertIn("let mut targets = Vec::with_capacity(total_capacity)", projection)
        self.assertNotIn("let mut targets = Vec::new()", projection)

    def test_node_target_projection_reserves_all_category_inputs(self):
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("pub fn all_target_nodes")
        end = source.index("fn dedupe_asset_ids", start)
        projection = " ".join(source[start:end].split())

        self.assertRegex(
            projection,
            r"let total_capacity = self\s*\.\s*template_rebuild_nodes\s*\.\s*len\(\)",
        )
        self.assertIn(".saturating_add(self.removed_compiled_nodes.len())", projection)
        self.assertIn(".saturating_add(self.theme_restyle_nodes.len())", projection)
        self.assertIn(".saturating_add(self.resource_damage_nodes.len())", projection)
        self.assertIn("let mut targets = Vec::with_capacity(total_capacity)", projection)
        self.assertNotIn("let mut targets = Vec::new()", projection)

    def test_lower_capacity_regression_and_release_marker_are_wired(self):
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn("all_target_surface_outputs_reserve_category_upper_bound", lower)
        self.assertIn("all_target_node_outputs_reserve_category_upper_bound", lower)
        self.assertIn(
            "RUNTIME792_SURFACE_INDEX_TARGET_CAPACITY_BENCH_V1",
            lower,
        )
        self.assertIn("geometric_growth_events", lower)
        self.assertIn("reserved_growth_events", lower)


if __name__ == "__main__":
    unittest.main()
