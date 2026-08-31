from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/keys.rs"
)


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


class BorrowedLiveRegistryReplacementPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.insert = function_body(cls.source, "pub(super) fn insert(")

    def test_existing_plugin_id_is_borrowed_before_owned_insert(self) -> None:
        borrowed_lookup = self.insert.index("map.get_mut(key.plugin_id())")
        owned_insert = self.insert.index(
            "map.insert(key.plugin_id().to_string(), value)"
        )
        self.assertLess(borrowed_lookup, owned_insert)

    def test_replacement_updates_value_without_rebuilding_key(self) -> None:
        self.assertIn("std::mem::replace(current, value)", self.insert)
        self.assertIn("return Some(previous);", self.insert)

    def test_rust_regression_covers_replacement_contract(self) -> None:
        self.assertIn(
            "replacement_returns_previous_value_without_replacing_key",
            self.source,
        )
        regression = function_body(
            self.source,
            "fn replacement_returns_previous_value_without_replacing_key()",
        )
        self.assertIn("registry", regression)
        self.assertIn(".plugin_ids(", regression)
        self.assertIn("PluginModuleKind::Runtime", regression)


if __name__ == "__main__":
    unittest.main()
