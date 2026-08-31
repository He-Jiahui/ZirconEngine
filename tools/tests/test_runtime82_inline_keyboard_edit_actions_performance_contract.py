import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
EDIT_ACTIONS = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "ui"
    / "surface"
    / "input"
    / "text_keyboard"
    / "edit_actions.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(function_name)}\s*\(", source)
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


class InlineKeyboardEditActionsPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = EDIT_ACTIONS.read_text(encoding="utf-8")
        cls.production = cls.source.split("#[cfg(test)]", 1)[0]

    def test_keyboard_actions_use_fixed_inline_storage(self) -> None:
        self.assertIn("struct KeyboardTextEditActions", self.production)
        self.assertIn("first: UiTextEditAction", self.production)
        self.assertIn("second: Option<UiTextEditAction>", self.production)
        self.assertIn("impl IntoIterator for KeyboardTextEditActions", self.production)
        self.assertIn(
            "std::iter::once(self.first).chain(self.second)", self.production
        )

    def test_owner_path_no_longer_materializes_action_vectors(self) -> None:
        self.assertNotIn("Vec<UiTextEditAction>", self.production)
        self.assertNotIn("vec![", self.production)
        self.assertIn("fn single_action(", self.production)
        self.assertIn("fn double_action(", self.production)

    def test_keyboard_entrypoints_return_the_inline_sequence(self) -> None:
        logical = function_body(self.production, "keyboard_text_edit_actions")
        key_code = function_body(
            self.production, "keyboard_text_edit_actions_from_key_code"
        )
        self.assertIn("Option<KeyboardTextEditActions>", self.production)
        self.assertIn("delete_previous_word_actions(state)", logical)
        self.assertIn("delete_next_word_actions(state)", logical)
        self.assertIn("delete_previous_word_actions(state)", key_code)
        self.assertIn("delete_next_word_actions(state)", key_code)

    def test_rust_regressions_cover_inline_iteration(self) -> None:
        self.assertIn(
            "fn runtime82_batch_inline_single_action_preserves_order()", self.source
        )
        self.assertIn(
            "fn runtime82_batch_inline_two_action_word_delete_preserves_order()",
            self.source,
        )
        self.assertIn(
            "fn runtime82_batch_inline_action_iterator_stays_exhausted()", self.source
        )
        self.assertIn(
            "fn runtime82_batch_inline_keyboard_edit_actions_p95()", self.source
        )


if __name__ == "__main__":
    unittest.main()
