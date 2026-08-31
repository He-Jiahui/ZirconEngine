from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/script/vm/host_interface/registry.rs"


def function_body(source: str, signature: str) -> str:
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
    raise AssertionError(f"unterminated function: {signature}")


class BorrowedActiveRegistrationIdentityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.latest_active = function_body(cls.source, "fn latest_active<")

    def test_selection_map_borrows_registration_identity(self) -> None:
        self.assertIn(
            "HashMap<(PluginSlotId, &str), (u32, T)>",
            self.latest_active,
        )
        self.assertIn("let identity = (key.slot, key.id.as_str());", self.latest_active)

    def test_selection_loop_does_not_clone_registration_ids(self) -> None:
        self.assertNotIn("key.id.clone()", self.latest_active)
        self.assertNotIn("key.id.to_owned()", self.latest_active)
        self.assertNotIn("key.id.to_string()", self.latest_active)

    def test_registration_values_and_deterministic_order_are_preserved(self) -> None:
        self.assertIn("registration.clone()", self.latest_active)
        self.assertIn("selected.sort_by", self.latest_active)
        self.assertIn("left.0.cmp(&right.0)", self.latest_active)


if __name__ == "__main__":
    unittest.main()
