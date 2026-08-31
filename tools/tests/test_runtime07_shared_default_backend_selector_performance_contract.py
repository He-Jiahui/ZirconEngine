from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs"


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


class SharedDefaultBackendSelectorPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.load_package = function_body(cls.source, "pub fn load_package(")
        cls.public_name = function_body(cls.source, "pub fn selected_backend_name(")
        cls.selection = function_body(cls.source, "pub fn select_default_backend(")

    def test_selected_backend_lock_owns_shared_string_storage(self) -> None:
        self.assertIn("selected_backend: RwLock<Arc<str>>", self.source)
        self.assertIn(
            "selected_backend: RwLock::new(Arc::from(DEFAULT_BACKEND_SELECTOR))",
            self.source,
        )
        self.assertNotIn("selected_backend: RwLock<String>", self.source)

    def test_default_load_clones_the_arc_not_the_string(self) -> None:
        self.assertIn("let backend_name = self.selected_backend_selector();", self.load_package)
        self.assertNotIn("self.selected_backend_name()", self.load_package)
        selector = function_body(self.source, "fn selected_backend_selector(")
        self.assertIn("Arc::clone", selector)
        self.assertNotIn(".to_string()", selector)
        self.assertNotIn(".to_owned()", selector)

    def test_owned_public_accessor_and_selection_behavior_are_preserved(self) -> None:
        self.assertIn("self.selected_backend_selector().to_string()", self.public_name)
        self.assertIn("Arc::from(backend_name)", self.selection)
        self.assertIn("self.backends.resolve(backend_name)?", self.selection)

    def test_lock_guards_follow_shared_selector_storage(self) -> None:
        self.assertIn("RwLockReadGuard<'_, Arc<str>>", self.source)
        self.assertIn("RwLockWriteGuard<'_, Arc<str>>", self.source)


if __name__ == "__main__":
    unittest.main()
