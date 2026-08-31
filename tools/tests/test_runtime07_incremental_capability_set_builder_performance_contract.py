from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
CAPABILITY_SET = ROOT / "zircon_runtime/src/script/vm/capability_set.rs"


def production_source() -> str:
    return CAPABILITY_SET.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]


class Runtime07IncrementalCapabilitySetBuilderPerformanceContractTests(
    unittest.TestCase
):
    def test_builder_uses_binary_insertion_on_sorted_unique_storage(self) -> None:
        source = production_source()

        self.assertIn("fn insert_sorted_unique(", source)
        self.assertIn("values.windows(2).all", source)
        self.assertIn("values.binary_search(&value)", source)
        self.assertIn("values.insert(index, value);", source)
        self.assertNotIn("self.capabilities.sort()", source)
        self.assertNotIn("self.capabilities.dedup()", source)

    def test_malformed_public_storage_keeps_compatibility_repair(self) -> None:
        source = production_source()

        self.assertEqual(source.count("values.sort();"), 1)
        self.assertEqual(source.count("values.dedup();"), 1)
        self.assertIn("values.push(value);", source)

    def test_sorted_duplicate_and_external_storage_paths_are_covered_by_rust(self) -> None:
        source = CAPABILITY_SET.read_text(encoding="utf-8")

        self.assertIn("fn with_keeps_capabilities_sorted_and_unique()", source)
        self.assertIn("fn with_repairs_externally_populated_capabilities()", source)
        self.assertIn("fn contains_accepts_manifest_order_without_sorted_storage()", source)


if __name__ == "__main__":
    unittest.main()
