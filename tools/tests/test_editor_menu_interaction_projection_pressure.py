from pathlib import Path
import unittest

from tools.editor_menu_interaction_projection_pressure import (
    pressure_report,
    validate_output_path,
)


ROOT = Path(__file__).resolve().parents[2]
HOVER = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_node_pipeline/hover.rs"
)
KEYBOARD_MENU = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/menu.rs"
)
POINTER_ITEMS = ROOT / (
    "zircon_editor/src/ui/retained_host/menu_pointer/"
    "host_menu_pointer_bridge_popup_items.rs"
)
POINTER_TREE = ROOT / (
    "zircon_editor/src/ui/retained_host/menu_pointer/menu_item_tree.rs"
)
POINTER_REBUILD = ROOT / (
    "zircon_editor/src/ui/retained_host/menu_pointer/"
    "host_menu_pointer_bridge_rebuild_surface.rs"
)
UNREAL_MENU_ENTRY = ROOT / (
    "dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/"
    "MultiBox/SMenuEntryBlock.cpp"
)


class EditorMenuInteractionProjectionPressureTests(unittest.TestCase):
    def test_large_menu_counts_event_owned_row_materialization(self):
        report = pressure_report(10_000, 1_000, 1_000, 100, 4, 7)

        current = report["current_event_owned_projection"]
        target = report["published_interaction_index"]
        self.assertEqual(current["hover_row_materializations"], 10_000_000)
        self.assertEqual(current["keyboard_row_materializations"], 10_000_000)
        self.assertEqual(current["operation_units"], 50_001_300)
        self.assertEqual(target["publication_row_materializations"], 10_000)
        self.assertEqual(target["operation_units"], 22_400)
        self.assertEqual(
            report["comparison"]["target_event_time_row_materializations"], 0
        )
        self.assertFalse(report["is_product_timing"])

    def test_small_menu_still_removes_event_time_row_vector_rebuilds(self):
        report = pressure_report(20, 1_000, 1_000, 100, 4, 7)

        self.assertEqual(
            report["current_event_owned_projection"]["operation_units"], 101_300
        )
        self.assertEqual(
            report["published_interaction_index"]["operation_units"], 2_440
        )
        self.assertGreater(
            report["comparison"]["operation_reduction_ratio"], 40.0
        )

    def test_rejects_non_positive_inputs(self):
        valid = [20, 1_000, 1_000, 100, 4, 7]
        for index in range(len(valid)):
            values = valid.copy()
            values[index] = 0
            with self.subTest(index=index):
                with self.assertRaises(ValueError):
                    pressure_report(*values)

    def test_artifact_output_rejects_the_system_drive(self):
        with self.assertRaises(ValueError):
            validate_output_path(r"C:\zircon-profiles\menu-pressure.json")
        self.assertEqual(
            validate_output_path(r"E:\zircon-profiles\menu-pressure.json").drive.upper(),
            "E:",
        )

    def test_model_is_bound_to_current_zircon_event_owned_projection(self):
        hover = HOVER.read_text(encoding="utf-8")
        keyboard = KEYBOARD_MENU.read_text(encoding="utf-8")
        popup_items = POINTER_ITEMS.read_text(encoding="utf-8")
        pointer_tree = POINTER_TREE.read_text(encoding="utf-8")
        pointer_rebuild = POINTER_REBUILD.read_text(encoding="utf-8")

        self.assertIn("structured_menu_items", hover)
        self.assertIn(".iter()", hover)
        self.assertIn(".cloned()", hover)
        self.assertIn("VecModel::from(items)", hover)
        self.assertIn("let rows: Vec<_> = (0..row_count)", keyboard)
        self.assertIn("action_id: item.action_id.clone()", keyboard)
        self.assertIn("value_text: item.label.clone()", keyboard)
        self.assertIn("menu_item_route_indices(&self.popup_items)", popup_items)
        self.assertIn("HashMap<Vec<usize>, usize>", pointer_tree)
        self.assertIn("let mut surface = UiSurface::new", pointer_rebuild)
        self.assertIn("surface.rebuild();", pointer_rebuild)

    def test_unreal_keeps_hover_on_the_row_and_defers_submenu_toggle(self):
        unreal = UNREAL_MENU_ENTRY.read_text(encoding="utf-8")
        self.assertIn("void SMenuEntryBlock::OnMouseEnter", unreal)
        self.assertIn("SMultiBlockBaseWidget::OnMouseEnter", unreal)
        self.assertIn("void SMenuEntryBlock::RequestSubMenuToggle", unreal)
        self.assertIn("RegisterActiveTimer(TimeToSubMenuOpen", unreal)
        self.assertIn("SMultiBoxWidget::FocusNextWidget", unreal)


if __name__ == "__main__":
    unittest.main()
