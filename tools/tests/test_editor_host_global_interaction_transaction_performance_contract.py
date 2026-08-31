from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
HOST = ROOT / "zircon_editor/src/ui/retained_host"
GLOBALS = HOST / "host_contract/globals"


def function_body(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    end_offset = source.index(end, offset + len(start))
    return source[offset:end_offset]


class EditorHostGlobalInteractionTransactionPerformanceContractTests(
    unittest.TestCase
):
    def test_asset_surface_writeback_publishes_one_interaction_transaction(self) -> None:
        source = (
            HOST / "app/pointer_layout/asset_surfaces/ui_writeback.rs"
        ).read_text(encoding="utf-8")

        self.assertEqual(1, source.count("set_asset_surface_interaction("))
        self.assertNotIn("set_activity_asset_", source)
        self.assertNotIn("set_browser_asset_", source)

    def test_hierarchy_writeback_publishes_one_interaction_transaction(self) -> None:
        source = (HOST / "app/pointer_layout/hierarchy.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("set_hierarchy_interaction(", source)
        self.assertNotIn("set_hierarchy_scroll_px(", source)
        self.assertNotIn("set_hovered_hierarchy_index(", source)

    def test_transaction_setters_preflight_before_one_full_state_update(self) -> None:
        source = (GLOBALS / "pane_context/setters/interaction.rs").read_text(
            encoding="utf-8"
        )

        asset = function_body(
            source,
            "pub(crate) fn set_asset_surface_interaction(",
            "pub(crate) fn set_hierarchy_interaction(",
        )
        hierarchy = function_body(
            source,
            "pub(crate) fn set_hierarchy_interaction(",
            "pub(crate) fn set_hierarchy_scroll_px(",
        )
        for body in (asset, hierarchy):
            self.assertIn("if !changed", body)
            self.assertEqual(1, body.count("update_pane_interaction("))

    def test_duplicate_scene_viewport_capture_returns_the_replacement_result(self) -> None:
        source = (GLOBALS / "pane_context/setters/viewport.rs").read_text(
            encoding="utf-8"
        )
        capture = function_body(
            source,
            "pub(crate) fn set_scene_viewport_capture(",
            "pub(crate) fn set_scene_viewport_product(",
        )

        self.assertIn(
            "self.state.borrow_mut().replace_scene_viewport_image(image)", capture
        )
        self.assertNotIn("\n        true", capture)

    def test_stable_viewport_chrome_preflights_before_wide_copy_on_write(self) -> None:
        source = (GLOBALS / "state/viewport_chrome.rs").read_text(encoding="utf-8")
        copy_offset = source.index("Arc::make_mut(&mut self.host_presentation)")
        prefix = source[:copy_offset]

        self.assertIn("scene_viewport_chrome_needs_patch", prefix)
        self.assertIn("return false", prefix)


if __name__ == "__main__":
    unittest.main()
