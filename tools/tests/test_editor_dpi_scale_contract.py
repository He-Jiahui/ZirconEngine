import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
DRAWER_LAYOUT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/drawer_layout.rs"
)


class EditorDpiScaleContractTests(unittest.TestCase):
    def test_scaled_workbench_projection_forwards_scale_to_drawers_and_breakpoints(self):
        source = DRAWER_LAYOUT.read_text(encoding="utf-8")
        self.assertIn(
            "let physical_shell_size = UiSize::new(mount_frame.width, mount_frame.height);",
            source,
        )
        self.assertRegex(
            source,
            re.compile(
                r"apply_workbench_responsive_layout\(\s*"
                r"&mut self\.template_surface\.surface,\s*"
                r"physical_shell_size,\s*"
                r"scale_factor,\s*"
                r"self\.compact_module_details_drawer_open,?\s*\)\?;",
                re.DOTALL,
            ),
        )
        self.assertNotIn(
            "apply_workbench_responsive_layout(&mut self.template_surface.surface, shell_size, 1.0)",
            source,
        )
        self.assertIn(
            "right_drawer_should_collapse_for_logical_width(shell_size.width)",
            source,
        )
        self.assertIn(
            "workbench_layout_tier_for_logical_width(shell_size.width)",
            source,
        )


if __name__ == "__main__":
    unittest.main()
