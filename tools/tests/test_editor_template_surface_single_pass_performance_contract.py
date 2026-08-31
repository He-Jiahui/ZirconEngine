from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SURFACE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/"
    "template_node/surface_frame_builder/surface.rs"
)


class EditorTemplateSurfaceSinglePassPerformanceContractTests(unittest.TestCase):
    def source(self) -> str:
        return SURFACE.read_text(encoding="utf-8")

    def test_dispatchable_rows_are_selected_by_one_lazy_iterator(self) -> None:
        source = self.source()

        self.assertNotIn("nodes.iter().any(is_dispatchable)", source)
        self.assertEqual(source.count(".enumerate()"), 1)
        self.assertIn("let mut dispatchable_nodes = nodes", source)
        self.assertIn(".filter(|(_, node)| is_dispatchable(node))", source)
        self.assertIn("let first_dispatchable = dispatchable_nodes.next()?;", source)

    def test_surface_build_consumes_first_and_remaining_rows_without_rescan(self) -> None:
        source = self.source()

        self.assertNotIn("fn template_nodes_surface_frame", source)
        self.assertIn("std::iter::once(first_dispatchable).chain(dispatchable_nodes)", source)
        self.assertIn("Some(surface.surface_frame())", source)


if __name__ == "__main__":
    unittest.main()
