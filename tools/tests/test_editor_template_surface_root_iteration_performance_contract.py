from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SURFACE_ROOT_DIRTY_PATHS = (
    ROOT
    / "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "floating_window_source/surface.rs",
    ROOT
    / "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "viewport_toolbar/host_projection.rs",
    ROOT
    / "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/drawer_layout.rs",
    ROOT
    / "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/host_projection.rs",
)


class EditorTemplateSurfaceRootIterationPerformanceContractTests(unittest.TestCase):
    def test_root_dirty_loops_do_not_clone_the_root_vector(self) -> None:
        for path in SURFACE_ROOT_DIRTY_PATHS:
            with self.subTest(path=path.relative_to(ROOT)):
                source = path.read_text(encoding="utf-8")

                self.assertNotIn("surface.tree.roots.clone()", source)
                self.assertIn(
                    "for root_index in 0..surface.tree.roots.len()", source
                )
                self.assertIn("let root_id = surface.tree.roots[root_index];", source)


if __name__ == "__main__":
    unittest.main()
