from pathlib import Path
import unittest

from tools.editor_borrowed_model_paint_pressure import run


ROOT = Path(__file__).resolve().parents[2]
PAINT = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer"
)
TEMPLATE_PAINT = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes"
)


class EditorBorrowedModelPaintPerformanceContractTests(unittest.TestCase):
    def assert_borrowed_model_reads(self, relative: Path) -> None:
        source = relative.read_text(encoding="utf-8")
        self.assertNotIn(".row_data(", source, str(relative))

    def test_visible_native_pane_rows_are_borrowed(self) -> None:
        for relative in (
            PAINT / "native_panes/hierarchy.rs",
            PAINT / "native_panes/hierarchy/viewport.rs",
            PAINT / "native_panes/assets/frame.rs",
            PAINT / "native_panes/diagnostics.rs",
        ):
            self.assert_borrowed_model_reads(relative)

    def test_menu_and_welcome_rows_are_borrowed(self) -> None:
        for relative in (
            PAINT / "menus/bar.rs",
            PAINT / "menus/rows.rs",
            PAINT / "menus/popup.rs",
            PAINT / "menus/popup/submenus.rs",
            PAINT / "welcome/recent_projects/rows.rs",
            PAINT / "docks/rail.rs",
            PAINT / "scene_layers/overlay/page_overflow.rs",
        ):
            self.assert_borrowed_model_reads(relative)

    def test_template_popup_rows_borrow_before_command_text_ownership(self) -> None:
        for relative in (
            TEMPLATE_PAINT / "template_popup_rows/menu/entry.rs",
            TEMPLATE_PAINT / "template_popup_rows/options/entry.rs",
            TEMPLATE_PAINT / "template_dialogs/actions/labels.rs",
            TEMPLATE_PAINT / "template_dropdowns/text.rs",
        ):
            self.assert_borrowed_model_reads(relative)

    def test_hierarchy_paint_uses_the_borrowed_model_api(self) -> None:
        source = (PAINT / "native_panes/hierarchy.rs").read_text(encoding="utf-8")
        self.assertIn("hierarchy_nodes.get(index)", source)
        self.assertNotIn("inline_hierarchy_rename_value(&node", source)
        self.assertNotIn("&node,\n            interaction", source)

    def test_pressure_model_counts_avoidable_stable_row_payload(self) -> None:
        result = run(
            stable_paint_count=10_000,
            visible_row_count=64,
            owned_text_fields_per_row=2,
            average_text_utf8_bytes=24,
        )

        self.assertEqual(result["delta"]["avoided_row_clone_count"], 640_000)
        self.assertEqual(
            result["delta"]["avoided_text_field_clone_count"], 1_280_000
        )
        self.assertEqual(
            result["delta"]["avoided_text_utf8_payload_bytes"], 30_720_000
        )
        self.assertEqual(
            result["borrowed_row_read"]["modeled_text_utf8_payload_bytes"], 0
        )


if __name__ == "__main__":
    unittest.main()
