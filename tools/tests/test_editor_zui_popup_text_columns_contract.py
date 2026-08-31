import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
POPUP_ROWS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_popup_rows"
)


class EditorPopupTextColumnsContractTests(unittest.TestCase):
    def test_menu_row_publishes_one_shared_label_shortcut_geometry(self):
        menu = (POPUP_ROWS / "menu/entry.rs").read_text(encoding="utf-8")
        label = (POPUP_ROWS / "text/label.rs").read_text(encoding="utf-8")
        shortcut = (POPUP_ROWS / "text/shortcut.rs").read_text(encoding="utf-8")

        self.assertEqual(menu.count("popup_row_text_columns("), 1)
        self.assertNotIn("popup_row_label_rect", label)
        self.assertNotIn("popup_row_shortcut_rect", shortcut)

    def test_columns_measure_shortcut_and_reserve_gap_and_adornment_slot(self):
        geometry = (POPUP_ROWS / "text/geometry.rs").read_text(encoding="utf-8")
        metrics = (POPUP_ROWS / "metrics.rs").read_text(encoding="utf-8")

        self.assertIn("menu_popup_text_width(shortcut)", geometry)
        self.assertIn("MENU_POPUP_LABEL_SHORTCUT_GAP", geometry)
        self.assertIn("metrics.adornment_reserved_width", geometry)
        self.assertNotIn("SHORTCUT_LEFT_RATIO", metrics)
        self.assertNotIn("SHORTCUT_WIDTH_RATIO", metrics)


if __name__ == "__main__":
    unittest.main()
