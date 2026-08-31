from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/plugin/extension_registry/owner.rs"


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


class SharedPluginModuleInternerNamesPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.intern = function_body(
            cls.source,
            "pub(in crate::plugin::extension_registry) fn intern(",
        )

    def test_interner_stores_one_shared_name_type_in_both_indexes(self) -> None:
        self.assertIn("use std::sync::Arc;", self.source)
        self.assertIn("names: Vec<Arc<str>>", self.source)
        self.assertIn("ids_by_name: HashMap<Arc<str>, PluginModuleId>", self.source)

    def test_duplicate_lookup_happens_before_shared_name_construction(self) -> None:
        lookup = self.intern.index("self.ids_by_name.get(name.as_str())")
        shared = self.intern.index("let name: Arc<str> = name.into();")
        self.assertLess(lookup, shared)

    def test_new_name_is_shared_without_a_deep_clone(self) -> None:
        self.assertIn("self.names.push(Arc::clone(&name))", self.intern)
        self.assertIn("self.ids_by_name.insert(name, id)", self.intern)
        self.assertNotIn("self.names.push(name.clone())", self.intern)

    def test_rust_regression_proves_indexes_and_clones_share_storage(self) -> None:
        self.assertIn("interner_indexes_and_clones_share_name_storage", self.source)
        self.assertGreaterEqual(self.source.count("Arc::ptr_eq"), 3)
        self.assertIn("get_key_value", self.source)


if __name__ == "__main__":
    unittest.main()
