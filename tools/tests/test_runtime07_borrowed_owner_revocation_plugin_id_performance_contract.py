from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs"
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


class BorrowedOwnerRevocationPluginIdPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.revoke = function_body(
            cls.source,
            "pub fn revoke_owner_registrations(",
        )

    def test_revocation_does_not_allocate_an_owned_plugin_id(self) -> None:
        self.assertNotIn("map(str::to_owned)", self.revoke)

    def test_plugin_id_remains_borrowed_for_shader_removal(self) -> None:
        self.assertIn("if let Some(plugin_id) = plugin_id {", self.revoke)
        self.assertNotIn("if let Some(plugin_id) = plugin_id.as_deref()", self.revoke)

    def test_same_borrow_is_reused_for_asset_importer_removal(self) -> None:
        self.assertIn("let asset_importers = plugin_id", self.revoke)
        self.assertNotIn("plugin_id\n            .as_deref()", self.revoke)

    def test_rust_regression_proves_plugin_id_is_a_prefix_borrow(self) -> None:
        self.assertIn("plugin_id_from_module_name_borrows_the_prefix", self.source)
        self.assertIn("plugin_id.as_ptr(), module_name.as_ptr()", self.source)


if __name__ == "__main__":
    unittest.main()
