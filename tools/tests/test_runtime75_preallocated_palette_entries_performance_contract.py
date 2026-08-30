from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/ui/component/catalog/palette_view.rs"


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


class PreallocatedPaletteEntriesPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.body = function_body(cls.source, "fn palette_entries_for_host(")

    def test_palette_projection_reserves_registry_upper_bound(self) -> None:
        self.assertIn("Vec::with_capacity(registry.len())", self.body)
        self.assertIn("entries.extend(", self.body)

    def test_palette_projection_does_not_grow_from_collect(self) -> None:
        self.assertNotIn("collect::<Vec", self.body)
        self.assertNotIn("let mut entries = registry", self.body)

    def test_rust_regressions_cover_capacity_and_sorting(self) -> None:
        self.assertIn(
            "preallocated_palette_projection_reserves_registry_upper_bound",
            self.source,
        )
        self.assertIn(
            "preallocated_palette_projection_preserves_sort_order", self.source
        )


if __name__ == "__main__":
    unittest.main()
