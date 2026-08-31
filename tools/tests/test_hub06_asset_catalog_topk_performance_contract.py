from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CATALOG = ROOT / "zircon_hub/src/assets/catalog.rs"


def rust_function_body(source: str, name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(name)}\b[^{{]*{{", source)
    if match is None:
        raise AssertionError(f"missing Rust function {name}")
    depth = 1
    index = match.end()
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError(f"unterminated Rust function {name}")
    return source[match.end() : index - 1]


class Hub06AssetCatalogTopKPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = CATALOG.read_text(encoding="utf-8")

    def test_catalog_routes_ranking_through_bounded_helper(self) -> None:
        body = rust_function_body(self.source, "discover_asset_catalog_for_scope")
        self.assertIn("retain_top_ranked_entries(&mut entries)", body)
        self.assertNotIn("entries.sort_by", body)
        self.assertNotIn("entries.truncate", body)

    def test_large_catalog_uses_linear_partial_selection(self) -> None:
        body = rust_function_body(self.source, "retain_top_ranked_entries")
        self.assertIn("entries.select_nth_unstable_by(ASSET_CATALOG_LIMIT", body)
        self.assertIn("entries.truncate(ASSET_CATALOG_LIMIT)", body)
        self.assertIn("entries.sort_by(ranked_asset_order)", body)

    def test_partial_selection_only_runs_above_the_limit(self) -> None:
        body = rust_function_body(self.source, "retain_top_ranked_entries")
        self.assertIn("entries.len() > ASSET_CATALOG_LIMIT", body)

    def test_total_order_is_shared_by_selection_and_final_sort(self) -> None:
        body = rust_function_body(self.source, "ranked_asset_order")
        for marker in (
            "source_priority",
            "root_rank",
            "entry.source",
            "entry.kind",
            "entry.name",
            "entry.path",
        ):
            self.assertIn(marker, body)

    def test_release_evidence_benchmarks_the_real_topk_helper(self) -> None:
        self.assertIn("hub06_asset_catalog_topk_release_benchmark_evidence", self.source)
        self.assertIn("HUB06_ASSET_CATALOG_TOPK_BENCH_V1", self.source)
        self.assertIn("const INPUT_ENTRIES: usize = 100_000", self.source)
        self.assertIn("legacy_full_sort", self.source)
        self.assertIn("retain_top_ranked_entries(&mut entries)", self.source)
        self.assertIn(".div_ceil(100)", self.source)

    def test_release_evidence_keeps_p50_and_p95_improvement_gates(self) -> None:
        self.assertIn(
            "optimized_p50_ns.saturating_mul(100)"
            " <= legacy_p50_ns.saturating_mul(65)",
            self.source,
        )
        self.assertIn(
            "optimized_p95_ns.saturating_mul(100)"
            " <= legacy_p95_ns.saturating_mul(65)",
            self.source,
        )


if __name__ == "__main__":
    unittest.main()
