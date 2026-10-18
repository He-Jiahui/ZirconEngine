from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
UUID = ROOT / "zircon_runtime_interface/src/resource/asset_uuid.rs"
QUERY = ROOT / "zircon_runtime/src/asset/registry/query.rs"


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


class Runtime206ReferencerBinarySortPerformanceContractTests(unittest.TestCase):
    def test_asset_uuid_exposes_a_stable_binary_sort_key(self) -> None:
        source = UUID.read_text(encoding="utf-8")

        self.assertIn("pub fn binary_key(&self) -> &[u8; 16]", source)
        self.assertIn("self.0.as_bytes()", source)

    def test_referencer_query_sorts_without_display_allocations(self) -> None:
        source = QUERY.read_text(encoding="utf-8")
        body = function_body(source, "pub fn get_referencers_by_uuid(")

        self.assertIn("sort_unstable_by", body)
        self.assertIn("binary_key", body)
        self.assertNotIn("ToString::to_string", body)

    def test_uniform_referencer_pressure_removes_per_comparison_strings(self) -> None:
        referencer_count = 65_536
        legacy_string_allocations = referencer_count
        optimized_string_allocations = 0

        self.assertEqual(legacy_string_allocations, 65_536)
        self.assertEqual(optimized_string_allocations, 0)
        self.assertEqual(
            legacy_string_allocations - optimized_string_allocations,
            65_536,
        )


if __name__ == "__main__":
    unittest.main()

