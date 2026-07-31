from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
THEME_STATE = ROOT / "zircon_editor/src/ui/asset_editor/session/theme_state.rs"


class EditorUiAssetThemePerformanceContractTests(unittest.TestCase):
    def test_single_theme_actions_are_moved_out_of_built_action_lists(self) -> None:
        source = THEME_STATE.read_text(encoding="utf-8")
        helper_start = source.index("pub(crate) fn theme_rule_helper_action")
        helper_end = source.index("pub fn apply_theme_rule_helper_item", helper_start)
        refactor_start = source.index("pub(crate) fn theme_refactor_action")
        refactor_end = source.index("pub fn apply_theme_refactor_item", refactor_start)

        for function in (
            source[helper_start:helper_end],
            source[refactor_start:refactor_end],
        ):
            self.assertIn(".into_iter()", function)
            self.assertIn(".nth(index)", function)
            self.assertNotIn(".get(index)", function)
            self.assertNotIn(".cloned()", function)


if __name__ == "__main__":
    unittest.main()
