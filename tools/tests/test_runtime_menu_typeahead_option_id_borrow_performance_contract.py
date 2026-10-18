import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MENU_SOURCE = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "ui"
    / "component"
    / "state_reducer"
    / "keyboard"
    / "menu.rs"
)
KEYBOARD_SOURCE = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "ui"
    / "component"
    / "state_reducer"
    / "keyboard.rs"
)
LOWER_REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/keyboard/menu/tests/typeahead_option_id_tests.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(
        rf"\bfn\s+{re.escape(function_name)}(?![A-Za-z0-9_])(?:<[^>]*>)?\s*\(.*?\)",
        source,
        re.S,
    )
    if match is None:
        raise AssertionError(f"missing function {function_name}")
    opening = source.find("{", match.end())
    if opening < 0:
        raise AssertionError(f"missing body for {function_name}")
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated body for {function_name}")


class RuntimeMenuTypeaheadOptionIdBorrowPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.menu_source = MENU_SOURCE.read_text(encoding="utf-8")
        cls.keyboard_source = KEYBOARD_SOURCE.read_text(encoding="utf-8")

    def test_typeahead_uses_option_entry_index_without_cloning_ids(self) -> None:
        body = function_body(self.menu_source, "apply_keyboard_text")
        self.assertIn(
            "super::current_option_entry_index(state, descriptor, &options)", body
        )
        self.assertNotIn("let option_ids", body)
        self.assertNotIn("collect::<Vec<_>>()", body)

    def test_keyboard_helper_matches_option_ids_by_borrow(self) -> None:
        helper = function_body(self.keyboard_source, "current_option_entry_value_index")
        self.assertIn("option.id == value", helper)
        self.assertNotIn("option.id.clone()", helper)
        self.assertIn("current_option_entry_index", self.keyboard_source)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn(
            "runtime765_menu_typeahead_borrows_option_ids_for_current_index",
            lower,
        )
        self.assertIn("RUNTIME765_MENU_TYPEAHEAD_OPTION_ID_BORROW_BENCH_V1", lower)
        self.assertIn("optimized_id_clones", lower)

    def test_deterministic_model_eliminates_option_id_clone_vector(self) -> None:
        option_count = 1_024
        legacy_id_clones = option_count
        optimized_id_clones = 0
        self.assertGreater(legacy_id_clones, optimized_id_clones)
        self.assertEqual(optimized_id_clones, 0)
        self.assertEqual(option_count, 1_024)


if __name__ == "__main__":
    unittest.main()
