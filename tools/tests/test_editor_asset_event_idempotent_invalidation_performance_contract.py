from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
ASSET_EVENT = ROOT / "zircon_editor/src/ui/host/editor_event_execution/asset_event.rs"
COMMON = ROOT / "zircon_editor/src/ui/host/editor_event_execution/common.rs"
WORKSPACE = ROOT / "zircon_editor/src/ui/workbench/project/asset_workspace_state.rs"
EDITOR_STATE = ROOT / "zircon_editor/src/ui/workbench/project/editor_state_asset_workspace.rs"
PRODUCT_TEST = (
    ROOT
    / "zircon_editor/src/tests/host/retained_callback_dispatch/asset/direct_dispatch.rs"
)


class AssetEventIdempotentInvalidationContract(unittest.TestCase):
    def test_asset_state_mutators_report_real_change(self) -> None:
        source = WORKSPACE.read_text(encoding="utf-8")
        for name in (
            "select_folder",
            "select_asset",
            "navigate_to_asset",
            "set_search_query",
            "set_kind_filter",
            "set_activity_view_mode",
            "set_browser_view_mode",
            "set_activity_utility_tab",
            "set_browser_utility_tab",
        ):
            self.assertRegex(source, rf"pub fn {name}\([^)]*\) -> bool")

    def test_editor_state_preserves_change_result(self) -> None:
        source = EDITOR_STATE.read_text(encoding="utf-8")
        self.assertGreaterEqual(source.count(") -> bool {"), 11)
        self.assertIn("self.asset_workspace.set_search_query(query)", source)
        self.assertIn("self.asset_workspace.set_browser_utility_tab(tab)", source)

    def test_asset_events_gate_invalidation_on_mutation_result(self) -> None:
        source = ASSET_EVENT.read_text(encoding="utf-8")
        self.assertIn("asset_mutation_effects", source)
        self.assertNotIn("Ok(asset_effects(true, false, true))", source)
        self.assertNotIn("Ok(asset_effects(true, true, true))", source)

    def test_unchanged_mutation_has_no_effects(self) -> None:
        source = COMMON.read_text(encoding="utf-8")
        self.assertIn("pub(super) fn asset_mutation_effects", source)
        self.assertIn("effects: Vec::new()", source)

    def test_product_regression_covers_repeated_dispatch(self) -> None:
        source = PRODUCT_TEST.read_text(encoding="utf-8")
        self.assertIn("fn repeated_asset_search_is_an_invalidation_noop", source)
        self.assertIn("assert!(!repeated.presentation_dirty)", source)
        self.assertIn("assert!(!repeated.refresh_visible_asset_previews)", source)


if __name__ == "__main__":
    unittest.main()
