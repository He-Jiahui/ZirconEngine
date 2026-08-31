from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
RETAINED_UI = ROOT / "zircon_editor/src/ui/retained_host/ui"
PANE_DATA = RETAINED_UI / "pane_data_conversion"


class EditorCommandPaletteProjectionPerformanceContractTests(unittest.TestCase):
    def test_combined_command_rows_are_exposed_to_workbench_projection(self) -> None:
        component_module = (
            PANE_DATA / "pane_component_projection/mod.rs"
        ).read_text(encoding="utf-8")
        pane_module = (PANE_DATA / "mod.rs").read_text(encoding="utf-8")

        self.assertIn("projected_command_palette_option_rows", component_module)
        self.assertIn("projected_command_palette_option_rows", pane_module)

    def test_workbench_projects_command_rows_once(self) -> None:
        source = (RETAINED_UI / "workbench_window_projection.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("projected_command_palette_option_rows", source)
        self.assertNotIn("projected_command_palette_options", source)
        self.assertNotIn("projected_command_palette_structured_options", source)


if __name__ == "__main__":
    unittest.main()
