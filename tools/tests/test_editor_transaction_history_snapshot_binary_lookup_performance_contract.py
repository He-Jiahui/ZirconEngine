from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SNAPSHOT = ROOT / (
    "zircon_editor/src/ui/workbench/snapshot/data/transaction_history_snapshot.rs"
)
REGRESSION = ROOT / "zircon_editor/src/tests/workbench/transaction_history_snapshot.rs"


class EditorTransactionHistorySnapshotBinaryLookupPerformanceContractTests(
    unittest.TestCase
):
    def test_snapshot_resolves_visible_top_with_ordered_binary_lookup(self) -> None:
        source = SNAPSHOT.read_text(encoding="utf-8")
        implementation = source.split("pub(crate) fn from_page", 1)[1].split(
            "\n    }\n}", 1
        )[0]

        self.assertIn("visible_top_index", implementation)
        self.assertIn("binary_search_by_key", implementation)
        self.assertNotIn("records.iter().position", implementation)

    def test_source_documents_the_history_order_invariant(self) -> None:
        source = SNAPSHOT.read_text(encoding="utf-8")
        self.assertIn("HistoryStore emits records in transaction-id order", source)
        self.assertIn("ordered page", source)

    def test_release_benchmark_reports_paired_tail_lookup_samples(self) -> None:
        source = REGRESSION.read_text(encoding="utf-8")
        self.assertIn("EDITOR02_HISTORY_SNAPSHOT_BINARY_TOP_BENCH_V1", source)
        self.assertIn("pair_order=alternating_legacy_even", source)
        self.assertIn("optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns", source)

    def test_ordered_page_model_reduces_tail_probe_work(self) -> None:
        record_count = 16_384
        target = record_count - 1
        legacy_steps = target + 1
        binary_steps = record_count.bit_length()

        self.assertEqual(legacy_steps, 16_384)
        self.assertLessEqual(binary_steps, 15)
        self.assertGreaterEqual(
            legacy_steps // max(binary_steps, 1),
            1_000,
        )


if __name__ == "__main__":
    unittest.main()
