from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PROJECTED_INVENTORY = ROOT / (
    "zircon_runtime/src/asset/project/manager/scan_and_import/projected_inventory.rs"
)


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime04BorrowedIdentityChangeFastPathPerformanceContractTests(
    unittest.TestCase
):
    def test_duplicate_guid_normalization_borrows_the_base_change_batch(self) -> None:
        source = PROJECTED_INVENTORY.read_text(encoding="utf-8")
        normalize = function_region(
            source,
            "    pub(super) fn normalize_duplicate_guids(",
            "    pub(super) fn document(",
        )

        self.assertIn(
            "merged_identity_changes(&self.identity_changes, watch_changes)",
            normalize,
        )
        self.assertIn("identity_changes.as_ref()", normalize)
        self.assertNotIn("self.identity_changes.clone()", normalize)

    def test_watch_delta_merge_is_capacity_sized_and_skips_empty_deltas(self) -> None:
        source = PROJECTED_INVENTORY.read_text(encoding="utf-8")
        merge = function_region(
            source,
            "fn merged_identity_changes",
            "#[cfg(test)]",
        )

        self.assertIn("Cow::Borrowed(identity_changes)", merge)
        self.assertIn("Some(watch_changes) if !watch_changes.is_empty()", merge)
        self.assertIn(
            "Vec::with_capacity(identity_changes.len() + watch_changes.len())",
            merge,
        )
        self.assertIn("merged.extend_from_slice(identity_changes);", merge)
        self.assertIn("merged.extend_from_slice(watch_changes);", merge)
        self.assertIn("Cow::Owned(merged)", merge)

    def test_borrowed_and_merged_behavior_is_covered_by_rust(self) -> None:
        source = PROJECTED_INVENTORY.read_text(encoding="utf-8")

        self.assertIn(
            "fn identity_change_merge_borrows_without_a_watch_delta()",
            source,
        )
        self.assertIn(
            "fn identity_change_merge_preserves_base_then_watch_order()",
            source,
        )

    def test_release_gate_reports_raw_paired_latency_samples(self) -> None:
        source = PROJECTED_INVENTORY.read_text(encoding="utf-8")

        self.assertIn("const BENCHMARK_WARMUP_PAIRS: usize = 4;", source)
        self.assertIn("const BENCHMARK_SAMPLE_PAIRS: usize = 21;", source)
        self.assertIn("IDENTITY_CHANGE_MERGE_PERF", source)
        self.assertIn("legacy_samples_ns={:?}", source)
        self.assertIn("optimized_samples_ns={:?}", source)
        self.assertIn("optimized_change_clones=0", source)
        self.assertIn("optimized_p50_ns.saturating_mul(20) <= legacy_p50_ns", source)
        self.assertIn("optimized_p95_ns.saturating_mul(20) <= legacy_p95_ns", source)


if __name__ == "__main__":
    unittest.main()
