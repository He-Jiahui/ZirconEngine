from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/plugin/bridge/import.rs"


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


class StaticBridgeImportInterfaceIdPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.constructor = function_body(cls.source, "pub(crate) fn new()")

    def test_erased_import_borrows_the_static_interface_id(self) -> None:
        self.assertIn("interface_id: &'static str", self.source)
        self.assertNotIn("interface_id: String", self.source)

    def test_constructor_does_not_allocate_the_interface_id(self) -> None:
        self.assertIn("interface_id: T::INTERFACE_ID,", self.constructor)
        self.assertNotIn("T::INTERFACE_ID.to_string()", self.constructor)
        self.assertNotIn("T::INTERFACE_ID.to_owned()", self.constructor)

    def test_interface_id_accessor_keeps_the_borrowed_api(self) -> None:
        accessor = function_body(self.source, "pub(crate) fn interface_id(&self)")
        self.assertIn("self.interface_id", accessor)
        self.assertNotIn("clone", accessor)

    def test_rust_regression_proves_clones_share_the_static_identity(self) -> None:
        self.assertIn(
            "erased_import_clones_share_static_interface_identity",
            self.source,
        )
        self.assertIn("std::ptr::eq", self.source)
        self.assertIn("StaticInterface::INTERFACE_ID.as_ptr()", self.source)


if __name__ == "__main__":
    unittest.main()
