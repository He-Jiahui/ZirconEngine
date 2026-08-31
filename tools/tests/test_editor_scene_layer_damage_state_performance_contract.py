from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SCENE = ROOT / "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/scene_layers"


class EditorSceneLayerDamageStatePerformanceContract(unittest.TestCase):
    def test_componentized_chrome_routes_damage_before_one_shared_focus_snapshot(self):
        source = (SCENE / "overlay/componentized.rs").read_text(encoding="utf-8")
        chrome = source.split("fn draw_componentized_workbench_chrome(", 1)[1]
        chrome = chrome.split("fn componentized_chrome_clips(", 1)[0]

        self.assertIn("componentized_chrome_damage_route", chrome)
        self.assertEqual(chrome.count("paint_text_input_focus(presentation)"), 2)
        self.assertIn("text_input_focus: &HostTextInputFocusData", chrome)
        clip = chrome.split("fn draw_componentized_workbench_chrome_clip(", 1)[1]
        self.assertNotIn("paint_text_input_focus(presentation)", clip)

    def test_page_overflow_rejects_off_popup_damage_before_metrics_and_rows(self):
        source = (SCENE / "overlay/page_overflow.rs").read_text(encoding="utf-8")
        draw = source.split("fn draw_host_page_overflow_menu(", 1)[1]
        draw = draw.split("fn page_overflow_palette(", 1)[0]

        damage_gate = draw.index("intersect(&popup, damage).is_none()")
        metrics = draw.index("current_host_metrics()")
        rows = draw.index("host_page_overflow_visible_row_range_with_state")
        self.assertLess(damage_gate, metrics)
        self.assertLess(damage_gate, rows)

    def test_floating_layer_prepares_state_only_after_an_accepted_window(self):
        source = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/floating_windows.rs"
        ).read_text(encoding="utf-8")
        draw = source.split("fn draw_floating_layer(", 1)[1]

        self.assertIn("get_or_insert_with", draw)
        first_cull = draw.index("continue;")
        lazy_state = draw.index("get_or_insert_with")
        self.assertLess(first_cull, lazy_state)
        self.assertEqual(draw.count("paint_pane_interaction_state(presentation)"), 1)
        self.assertEqual(draw.count("paint_viewport_images(presentation)"), 1)
        self.assertEqual(draw.count("paint_text_input_focus(presentation)"), 1)


if __name__ == "__main__":
    unittest.main()
