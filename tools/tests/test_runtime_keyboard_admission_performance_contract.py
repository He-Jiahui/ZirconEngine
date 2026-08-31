from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
KEYBOARD_ACTION = ROOT / "zircon_runtime/src/ui/surface/input/keyboard_action.rs"
KEYBOARD_NAVIGATION = (
    ROOT / "zircon_runtime/src/ui/surface/input/keyboard_navigation.rs"
)


def rust_block(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated Rust block: {signature}")


class RuntimeKeyboardAdmissionPerformanceContractTests(unittest.TestCase):
    def test_keyboard_text_combines_control_and_whitespace_classification(self) -> None:
        source = KEYBOARD_ACTION.read_text(encoding="utf-8")
        body = rust_block(source, "fn keyboard_text_is_usable(")

        self.assertEqual(body.count("for character in text.chars()"), 1)
        self.assertIn("character.is_control()", body)
        self.assertIn("character.is_whitespace()", body)
        self.assertNotIn("text.chars().any", body)
        self.assertNotIn("text.chars().all", body)

    def test_accepted_keyboard_text_remains_borrowed(self) -> None:
        source = KEYBOARD_ACTION.read_text(encoding="utf-8")
        body = rust_block(source, "pub(super) fn keyboard_component_text(")

        self.assertIn("Option<&str>", source)
        self.assertIn("Some(text)", body)
        self.assertNotIn("to_string", body)
        self.assertNotIn("to_owned", body)

    def test_direction_key_normalizes_once_into_the_fixed_stack_buffer(self) -> None:
        source = KEYBOARD_NAVIGATION.read_text(encoding="utf-8")
        body = rust_block(source, "fn normalized_direction_key(")

        self.assertIn('MAX_NORMALIZED_DIRECTION_KEY_BYTES: usize = "gamepaddpadright".len()', source)
        self.assertIn("let mut normalized = [0; MAX_NORMALIZED_DIRECTION_KEY_BYTES]", body)
        self.assertEqual(body.count("for byte in key.bytes()"), 1)
        self.assertIn("if len == normalized.len()", body)
        self.assertNotIn("Vec", body)

    def test_direction_matching_uses_one_normalized_slice_match(self) -> None:
        source = KEYBOARD_NAVIGATION.read_text(encoding="utf-8")
        body = rust_block(source, "fn logical_directional_navigation_kind(")

        self.assertEqual(body.count("normalized_direction_key(logical_key)"), 1)
        self.assertIn("match &normalized[..len]", body)
        self.assertNotIn("expected.iter()", body)


if __name__ == "__main__":
    unittest.main()
