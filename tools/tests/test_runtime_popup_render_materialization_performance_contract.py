import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RENDER_ROOT = REPO_ROOT / "zircon_runtime" / "src" / "ui" / "surface" / "render"
MENU_PATH = RENDER_ROOT / "popup_menu.rs"
OPTIONS_PATH = RENDER_ROOT / "popup_options.rs"
ROWS_PATH = RENDER_ROOT / "popup_rows.rs"


class RuntimePopupRenderMaterializationPerformanceContractTests(unittest.TestCase):
    def test_transient_attribute_sets_borrow_small_inputs_and_index_large_inputs(self):
        rows = ROWS_PATH.read_text(encoding="utf-8")

        self.assertIn("const POPUP_ATTRIBUTE_LINEAR_SCAN_LIMIT: usize", rows)
        self.assertRegex(rows, r"enum\s+PopupAttributeIdSet<'a>")
        self.assertRegex(rows, r"Linear\(&'a\s*\[toml::Value\]\)")
        self.assertRegex(rows, r"Indexed\(HashSet<&'a str>\)")
        self.assertIn("HashSet::with_capacity", rows)
        self.assertNotIn("HashSet<String>", rows)

    def test_menu_state_projection_does_not_clone_attribute_ids(self):
        menu = MENU_PATH.read_text(encoding="utf-8")

        self.assertNotIn("BTreeSet", menu)
        self.assertNotIn("option_id_set", menu)
        self.assertGreaterEqual(menu.count("PopupAttributeIdSet::new"), 6)
        self.assertRegex(menu, r"struct\s+RuntimePopupMenuItem<'a>\s*\{")
        self.assertRegex(menu, r"\bid:\s*&'a str,")
        self.assertRegex(
            menu,
            r"Vec::with_capacity\(item_count\.saturating_mul\(3\)\.saturating_add\(3\)\)",
        )

    def test_option_state_projection_only_owns_semantic_selected_values(self):
        options = OPTIONS_PATH.read_text(encoding="utf-8")

        self.assertNotIn("fn option_id_set", options)
        self.assertGreaterEqual(options.count("PopupAttributeIdSet::new"), 8)
        self.assertIn("fn value_selected_option_ids", options)
        self.assertNotIn("fn selected_option_ids", options)
        self.assertNotIn("fn option_values", options)
        self.assertNotIn("selected.extend(", options)
        self.assertRegex(options, r"struct\s+RuntimePopupOption<'a>\s*\{")
        self.assertRegex(options, r"\bid:\s*&'a str,")
        self.assertRegex(
            options,
            r"Vec::with_capacity\(option_count\.saturating_mul\(3\)\.saturating_add\(3\)\)",
        )


if __name__ == "__main__":
    unittest.main()
