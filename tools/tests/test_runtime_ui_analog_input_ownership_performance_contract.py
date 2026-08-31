from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
ANALOG = ROOT / "zircon_runtime/src/ui/surface/input/analog.rs"
ANALOG_NAVIGATION = ROOT / "zircon_runtime/src/ui/surface/input/analog_navigation.rs"


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


class RuntimeUiAnalogInputOwnershipPerformanceContractTests(unittest.TestCase):
    def test_retained_analog_value_does_not_clone_the_owned_event(self) -> None:
        source = ANALOG.read_text(encoding="utf-8")
        dispatch = rust_block(source, "pub(super) fn dispatch_analog_input")

        self.assertNotIn("analog_with_retained_control_value", source)
        self.assertNotIn("analog.clone()", dispatch)
        self.assertIn("let retained_analog_value =", dispatch)
        self.assertIn(
            "analog_navigation_decision(&mut surface.input, &analog, retained_analog_value)",
            dispatch,
        )

    def test_canonical_control_normalization_borrows_the_input(self) -> None:
        source = ANALOG_NAVIGATION.read_text(encoding="utf-8")
        decision = rust_block(source, "pub(super) fn analog_navigation_decision")
        normalize = rust_block(source, "fn normalized_control_name")

        self.assertIn("use std::borrow::Cow;", source)
        self.assertIn("analog_value: f32", source)
        self.assertIn("analog_navigation_kind(axis, analog_value)", decision)
        self.assertIn("Cow::Borrowed(control)", normalize)
        self.assertIn("Cow::Owned", normalize)
        self.assertIn("is_ascii_lowercase", normalize)


if __name__ == "__main__":
    unittest.main()
