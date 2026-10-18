import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SEPARATOR_COMMANDS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_shell_panels/separators/commands.rs"
)
SHELL_PANELS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_shell_panels.rs"
)
VIEWPORT_SCENE_COMMANDS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_viewport_scene/commands.rs"
)
TIMELINE_GEOMETRY = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "material_primitives/timeline/geometry.rs"
)


class EditorUiShellSeparatorContractTests(unittest.TestCase):
    def test_shell_surface_preserves_post_dpi_fractional_geometry(self):
        source = SHELL_PANELS.read_text(encoding="utf-8")

        self.assertNotIn("pixel_aligned_rect", source)
        self.assertIn(
            "push_shell_panel_surface(commands, node, kind, rect, &clip, order, opacity);",
            source,
        )

    def test_viewport_does_not_duplicate_the_side_panel_seam(self):
        source = SEPARATOR_COMMANDS.read_text(encoding="utf-8")

        self.assertIn(
            "ShellPanelKind::ActivityRail | ShellPanelKind::ScenePanel =>",
            source,
        )
        self.assertIn("ShellPanelKind::InspectorPanel =>", source)
        self.assertNotIn("ShellPanelKind::ViewportPanel => {", source)
        self.assertIn("| ShellPanelKind::ViewportPanel", source)

    def test_viewport_scene_preserves_post_dpi_fractional_geometry(self):
        source = VIEWPORT_SCENE_COMMANDS.read_text(encoding="utf-8")

        self.assertNotIn("pixel_aligned_rect", source)
        self.assertNotIn("let rect =", source)
        self.assertIn("push_base_surface(commands, node, rect, clip, order, opacity);", source)

    def test_analytic_timeline_dot_preserves_fractional_centered_square(self):
        source = TIMELINE_GEOMETRY.read_text(encoding="utf-8")

        self.assertNotIn("inward_pixel_aligned_rect", source)
        self.assertIn("x: rect.x + (width - size) * 0.5", source)
        self.assertIn("width: size", source)


if __name__ == "__main__":
    unittest.main()
