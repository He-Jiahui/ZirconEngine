from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
REDUCER = ROOT / "zircon_runtime/src/ui/component/state_reducer"


class RuntimeUiAsciiSearchPerformanceContractTests(unittest.TestCase):
    def test_shared_matcher_keeps_ascii_menu_search_allocation_free(self) -> None:
        source = (REDUCER / "text_search.rs").read_text(encoding="utf-8")

        self.assertIn("value.is_ascii() && lowercase_query.is_ascii()", source)
        self.assertIn("eq_ignore_ascii_case", source)
        self.assertIn("value.to_lowercase()", source)

    def test_menu_and_command_palette_use_the_shared_matcher(self) -> None:
        menu = (REDUCER / "keyboard/menu.rs").read_text(encoding="utf-8")
        palette = (REDUCER / "command_palette.rs").read_text(encoding="utf-8")

        self.assertIn("starts_with_lowercase_query", menu)
        self.assertIn("contains_lowercase_query", menu)
        self.assertNotIn("text.trim_start().to_lowercase().starts_with(search)", menu)
        self.assertNotIn("text.trim().to_lowercase().contains(query)", menu)
        self.assertNotIn("id.trim().to_lowercase().contains(query)", menu)

        self.assertIn("contains_lowercase_query", palette)
        self.assertNotIn("value.trim().to_lowercase().contains(query)", palette)

    def test_command_navigation_reuses_one_filter_projection(self) -> None:
        palette = (REDUCER / "command_palette.rs").read_text(encoding="utf-8")
        navigation = palette.split("fn navigate_filtered_commands", 1)[1].split(
            "#[derive(Clone, Copy)]", 1
        )[0]

        self.assertIn("struct CommandFilterProjection", palette)
        self.assertIn("-> CommandFilterProjection", palette)
        self.assertIn("let projection = sync_filter_state", navigation)
        self.assertNotIn("let filtered = filtered_command_ids", navigation)
        self.assertNotIn("let entries = command_entries", navigation)

    def test_disabled_command_membership_is_set_backed(self) -> None:
        palette = (REDUCER / "command_palette.rs").read_text(encoding="utf-8")

        self.assertIn("HashSet", palette)
        self.assertIn("disabled.contains", palette)
        self.assertNotIn("disabled.iter().any", palette)

    def test_menu_navigation_builds_one_borrowed_eligibility_index(self) -> None:
        keyboard = (REDUCER / "keyboard.rs").read_text(encoding="utf-8")
        menu = (REDUCER / "keyboard/menu.rs").read_text(encoding="utf-8")

        self.assertIn("struct ExplicitOptionEligibility<'a>", keyboard)
        self.assertIn("struct OptionEligibility<'a>", keyboard)
        self.assertIn("HashSet<&'a str>", keyboard)
        self.assertIn("let eligibility = OptionEligibility::new", keyboard)
        self.assertIn("eligibility.is_disabled", keyboard)
        self.assertNotIn("option_is_explicitly_disabled(state", keyboard)

        self.assertIn("pub(super) struct MenuSearchFilter<'a>", menu)
        self.assertIn("HashSet<&'a str>", menu)
        self.assertIn("option_search_filter", menu)

    def test_menu_filter_sync_does_not_parse_top_level_options_twice(self) -> None:
        menu = (REDUCER / "keyboard/menu.rs").read_text(encoding="utf-8")
        sync = menu.split("fn sync_search_filter_state", 1)[1].split(
            "pub(super) struct MenuSearchFilter", 1
        )[0]

        self.assertNotIn("let options = super::option_entries", sync)


if __name__ == "__main__":
    unittest.main()
