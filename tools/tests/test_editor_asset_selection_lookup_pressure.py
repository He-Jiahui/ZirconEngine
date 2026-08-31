import importlib.util
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
TOOL = ROOT / "tools" / "editor_asset_selection_lookup_pressure.py"
SELECTION = ROOT / "zircon_editor/src/ui/layouts/views/asset_browser/selection_text.rs"
CONTEXT_MENU = ROOT / (
    "zircon_editor/src/ui/retained_host/app/asset_content_pointer/context_menu.rs"
)

spec = importlib.util.spec_from_file_location("asset_selection_pressure", TOOL)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = module
spec.loader.exec_module(module)


class AssetSelectionLookupPressureTests(unittest.TestCase):
    def test_selection_uses_uuid_and_selected_indices(self):
        source = SELECTION.read_text(encoding="utf-8")
        body = source.split("pub(super) fn selected_asset(", 1)[1].split(
            "\npub(super) fn has_asset_selection", 1
        )[0]
        self.assertIn("visible_assets.selected_index", body)
        self.assertIn("visible_assets.selected_indices()", body)
        self.assertIn("visible_assets.get(selected_index)", body)
        self.assertNotIn("visible_assets.iter()", body)

    def test_context_menu_uses_the_same_uuid_index(self):
        source = CONTEXT_MENU.read_text(encoding="utf-8")
        self.assertIn("visible_assets.selected_index(&asset_uuid)", source)
        self.assertIn("visible_assets.get(index)", source)
        self.assertNotIn("visible_assets.iter()", source)

    def test_pressure_model_removes_full_visible_asset_scans(self):
        report = module.pressure_report(100_000, 1_000, 1_000)
        self.assertEqual(report["current"]["total_lookup_units"], 200_000_000)
        self.assertEqual(report["target"]["total_lookup_units"], 5_000)
        self.assertAlmostEqual(
            report["ratios"]["total_lookup_units"], 200_000_000 / 5_000
        )


if __name__ == "__main__":
    unittest.main()
