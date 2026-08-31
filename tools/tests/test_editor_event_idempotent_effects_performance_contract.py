from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
COMMON = ROOT / "zircon_editor/src/ui/host/editor_event_execution/common.rs"
SELECTION = ROOT / "zircon_editor/src/ui/host/editor_event_execution/selection_event.rs"
HIERARCHY = ROOT / "zircon_editor/src/ui/host/editor_event_execution/hierarchy_event.rs"
MENU = ROOT / "zircon_editor/src/ui/host/editor_event_execution/menu_action.rs"
HIERARCHY_TEST = (
    ROOT / "zircon_editor/src/tests/host/retained_callback_dispatch/hierarchy.rs"
)
CONSOLE_TEST = ROOT / "zircon_editor/src/tests/editor_event/runtime/console.rs"


class EditorEventIdempotentEffectsContract(unittest.TestCase):
    def test_shared_effect_gate_avoids_allocating_for_noop(self) -> None:
        source = COMMON.read_text(encoding="utf-8")
        self.assertIn("pub(super) fn effects_when", source)
        self.assertIn("if changed", source)
        self.assertIn("Vec::new()", source)

    def test_selection_and_hierarchy_gate_presentation_effects(self) -> None:
        for path in (SELECTION, HIERARCHY):
            source = path.read_text(encoding="utf-8")
            self.assertIn("effects: effects_when(", source)
            self.assertIn("changed,", source)

    def test_console_clear_and_filters_gate_presentation_effects(self) -> None:
        source = MENU.read_text(encoding="utf-8")
        region = source.split("MenuAction::ClearConsole", 1)[1].split(
            "MenuAction::SelectPlayMode", 1
        )[0]
        self.assertEqual(region.count("effects_when("), 3)
        self.assertNotIn("effects: vec![EditorEventEffect::PresentationChanged]", region)

    def test_product_regressions_cover_repeated_controls(self) -> None:
        hierarchy = HIERARCHY_TEST.read_text(encoding="utf-8")
        console = CONSOLE_TEST.read_text(encoding="utf-8")
        self.assertIn("fn repeated_hierarchy_selection_is_an_invalidation_noop", hierarchy)
        self.assertIn("assert!(!repeated.presentation_dirty)", hierarchy)
        self.assertIn("fn repeated_console_filter_is_an_invalidation_noop", console)
        self.assertIn("assert!(repeated.dirty_domains().is_empty())", console)


if __name__ == "__main__":
    unittest.main()
