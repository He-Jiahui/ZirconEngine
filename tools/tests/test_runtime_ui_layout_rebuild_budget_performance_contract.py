from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs"


class RuntimeUiLayoutRebuildBudgetPerformanceContract(unittest.TestCase):
    def test_layout_path_has_an_explicit_bounded_incremental_budget(self):
        source = SOURCE.read_text(encoding="utf-8")
        self.assertIn("UI_LAYOUT_INCREMENTAL_MAX_DIRTY_RATIO_DENOMINATOR", source)
        self.assertIn("UI_LAYOUT_INCREMENTAL_MAX_DIRTY_NODE_COUNT", source)
        self.assertIn("fn should_use_full_layout_rebuild", source)
        self.assertIn("let layout_dirty_node_count", source)
        self.assertIn("self.tree.nodes.len()", source)
        self.assertIn("should_use_full_layout_rebuild(", source)

    def test_broad_layout_updates_take_one_full_layout_pass(self):
        source = SOURCE.read_text(encoding="utf-8")
        rebuild = source.split("pub fn rebuild_dirty(", 1)[1].split(
            "fn should_use_full_layout_rebuild", 1
        )[0]
        self.assertIn("self.compute_layout(root_size)", rebuild)
        self.assertIn("ui.layout.full_rebuild_threshold_count", rebuild)


if __name__ == "__main__":
    unittest.main()
