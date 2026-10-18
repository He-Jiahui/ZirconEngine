from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/core/commands/palette/locale_projection.rs"
PRODUCER = SOURCE.parent / "tests/locale_projection.rs"


class EditorPaletteLocalePostingCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.producer = PRODUCER.read_text(encoding="utf-8")
        start = cls.source.index("pub(super) fn build(")
        end = cls.source.index("\n    pub(super) fn matches", start)
        cls.body = cls.source[start:end]

    def test_posting_counts_are_collected_before_exact_bucket_reservation(self) -> None:
        self.assertIn("let mut posting_counts = [0usize; 256];", self.body)
        self.assertIn("posting_counts[byte] += 1;", self.body)
        self.assertIn("let mut search_postings = posting_counts.map(Vec::with_capacity);", self.body)
        self.assertIn("for (index, document) in search_documents.iter().enumerate()", self.body)
        self.assertIn("EDITOR812_PALETTE_LOCALE_POSTING_CAPACITY_BENCH_V1", self.producer)

    def test_posting_order_and_byte_deduplication_remain_explicit(self) -> None:
        self.assertGreaterEqual(self.body.count("indexed_bytes = [false; 256]"), 2)
        self.assertIn("search_postings[byte].push(index);", self.body)
        self.assertNotIn(
            "let mut search_postings: [Vec<usize>; 256] = std::array::from_fn(|_| Vec::new());",
            self.body,
        )

    def test_entry_and_document_bounds_remain_reserved(self) -> None:
        self.assertIn("Vec::with_capacity(seeds.len())", self.body)
        self.assertEqual(self.body.count("Vec::with_capacity(seeds.len())"), 2)
        self.assertIn("entries.push(entry);", self.body)
        self.assertIn("search_documents.push(document);", self.body)

    def test_dense_posting_model_removes_geometric_growth(self) -> None:
        seed_count = 4096
        bucket_count = 256
        legacy_growth_events = 0
        capacity = 0
        for length in range(1, seed_count + 1):
            if length > capacity:
                capacity = 4 if capacity == 0 else capacity * 2
                legacy_growth_events += 1
        self.assertGreater(legacy_growth_events, 0)
        self.assertEqual(legacy_growth_events * bucket_count, 2816)
        self.assertIn("workload=dense_bucket_geometric_model", self.producer)
        self.assertIn("EDITOR812_PALETTE_LOCALE_POSTING_ACTUAL_BUILD_BENCH_V2", self.producer)
        self.assertIn("EditorCommandPaletteLocaleProjection::build(", self.producer)
        self.assertIn("let previous_capacity = search_postings[byte].capacity();", self.body)
        self.assertIn("if search_postings[byte].capacity() != previous_capacity", self.body)
        self.assertIn("posting_capacity.growth_events[byte] += 1;", self.body)
        self.assertIn(
            "initial_capacities: std::array::from_fn(|byte| search_postings[byte].capacity())",
            self.body,
        )
        self.assertIn("posting_capacity.final_capacities =", self.body)
        self.assertIn("projection.posting_capacity.growth_events.iter().sum()", self.producer)
        self.assertIn("let previous_capacity = postings[byte].capacity();", self.producer)
        self.assertIn("actual.as_ref(), expected.as_slice()", self.producer)
        self.assertIn("projection.posting_capacity.initial_capacities[byte]", self.producer)
        self.assertIn("projection.posting_capacity.final_capacities[byte]", self.producer)


if __name__ == "__main__":
    unittest.main()
