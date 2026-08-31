from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
RESOLVE = ROOT / "zircon_runtime/src/ui/template/asset/localization/resolve.rs"
PERFORMANCE = ROOT / "zircon_runtime/src/ui/template/asset/localization/resolve/performance_tests.rs"


class Runtime83LocalizationHashKeyIndexPerformanceContractTests(unittest.TestCase):
    def test_catalog_uses_hash_indexes_for_locale_and_table_membership(self) -> None:
        source = RESOLVE.read_text(encoding="utf-8")

        self.assertIn(
            "tables: HashMap<String, HashMap<String, UiLocalizationTableEntry>>",
            source,
        )
        self.assertIn("Option<&HashMap<String, UiLocalizationTableEntry>>", source)

    def test_catalog_uses_hash_index_for_repeated_key_membership(self) -> None:
        source = RESOLVE.read_text(encoding="utf-8")

        self.assertIn("use std::collections::{BTreeSet, HashMap, HashSet};", source)
        self.assertIn("keys: HashSet<String>,", source)
        self.assertIn("if table.keys.contains(&dependency.reference.key)", source)

    def test_toml_key_projection_remains_deterministically_ordered(self) -> None:
        source = RESOLVE.read_text(encoding="utf-8")

        self.assertIn(
            ") -> Result<BTreeSet<String>, toml::de::Error>",
            source,
        )
        self.assertIn("let mut keys = BTreeSet::new();", source)

    def test_release_benchmark_compares_ordered_and_hash_membership(self) -> None:
        performance = PERFORMANCE.read_text(encoding="utf-8")

        self.assertIn("RUNTIME83_LOCALIZATION_HASH_KEY_INDEX_BENCH_V1", performance)
        self.assertIn("legacy_btree_p95_ns", performance)
        self.assertIn("optimized_hash_p95_ns", performance)
        self.assertIn("KEY_GROUPS * KEYS_PER_GROUP", performance)

    def test_release_benchmark_covers_dependency_table_lookup(self) -> None:
        performance = PERFORMANCE.read_text(encoding="utf-8")

        self.assertIn("RUNTIME83_LOCALIZATION_HASH_TABLE_INDEX_BENCH_V1", performance)
        self.assertIn("legacy_table_btree_p95_ns", performance)
        self.assertIn("optimized_table_hash_p95_ns", performance)


if __name__ == "__main__":
    unittest.main()
