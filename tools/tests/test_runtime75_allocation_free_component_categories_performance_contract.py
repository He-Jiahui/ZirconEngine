from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/ui/component/catalog/registry.rs"


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


class AllocationFreeComponentCategoriesPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_registry_categories_delegates_to_fixed_capacity_projection(self) -> None:
        body = function_body(self.source, "pub fn categories(&self)")

        self.assertIn("unique_component_categories", body)
        self.assertNotIn("collect::<BTreeSet", body)
        self.assertNotIn("collect::<HashSet", body)

    def test_category_projection_uses_no_heap_collection(self) -> None:
        body = function_body(self.source, "fn unique_component_categories(")

        self.assertIn("[None; COMPONENT_CATEGORIES.len()]", body)
        self.assertIn("component_category_index", body)
        self.assertIn(
            "ordered[component_category_index(category)] = Some(category)", body
        )
        self.assertIn("ordered.into_iter().flatten()", body)
        self.assertNotIn("BTreeSet", body)
        self.assertNotIn("HashSet", body)
        self.assertNotIn("Vec", body)

    def test_rust_regressions_cover_order_duplicates_and_empty_input(self) -> None:
        self.assertIn("allocation_free_categories_preserve_enum_order", self.source)
        self.assertIn("allocation_free_categories_deduplicate_repeated_values", self.source)
        self.assertIn("allocation_free_categories_handle_empty_input", self.source)


if __name__ == "__main__":
    unittest.main()
