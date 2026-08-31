import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SURFACE = ROOT / "zircon_runtime/src/ui/surface/surface.rs"
ARRANGED = ROOT / "zircon_runtime/src/ui/surface/arranged.rs"


class RuntimeSurfaceFocusPathIndexPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.surface = SURFACE.read_text(encoding="utf-8")
        cls.arranged = ARRANGED.read_text(encoding="utf-8")

    def test_surface_focus_path_uses_the_arranged_node_index(self) -> None:
        focus_path = self.surface.split("pub fn focus_path(&self)", 1)[1].split(
            "pub fn focused_route", 1
        )[0]
        self.assertIn("arranged_focus_path_indexed(", focus_path)
        self.assertIn("&self.arranged_node_indices", focus_path)
        self.assertNotIn("arranged_focus_path(&self.arranged_tree", focus_path)

    def test_indexed_focus_path_preserves_missing_node_fallback(self) -> None:
        indexed = self.arranged.split("pub(crate) fn arranged_focus_path_indexed", 1)[
            1
        ].split("pub fn is_arranged_render_visible", 1)[0]
        self.assertIn("arranged_bubble_route_indexed", indexed)
        self.assertIn("focused: Some(focused)", indexed)


if __name__ == "__main__":
    unittest.main()
