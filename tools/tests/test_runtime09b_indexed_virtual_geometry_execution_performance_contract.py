from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
EXECUTION = (
    ROOT
    / "zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/execution.rs"
)
ROOT_SOURCE = (
    ROOT
    / "zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs"
)
RECORD = (
    ROOT
    / "docs/plans/optimize/zircon_runtime/09b/2026-08-27-indexed-virtual-geometry-execution-projection.md"
)


def production_source(path: Path) -> str:
    return path.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]


class Runtime09BIndexedVirtualGeometryExecutionContract(unittest.TestCase):
    def test_execution_projection_builds_one_reused_lookup(self) -> None:
        source = production_source(ROOT_SOURCE)
        self.assertEqual(source.count("ExecutionLookup::new(extract)"), 1)
        self.assertEqual(source.count("&execution_lookup"), 2)
        self.assertIn("build_execution_snapshot(", source)
        self.assertIn("build_selected_clusters_from_execution_segments(", source)

    def test_lookup_uses_first_match_hash_index_and_sorted_cluster_ids(self) -> None:
        source = production_source(EXECUTION)
        self.assertIn("HashMap<u64, Vec<u32>>", source)
        self.assertIn("HashMap<ExecutionClusterKey, usize>", source)
        self.assertIn(".or_insert(cluster_index)", source)
        self.assertIn("cluster_ids.sort_unstable();", source)
        self.assertIn("cluster_ids.dedup();", source)

    def test_production_projection_does_not_restore_nested_linear_searches(self) -> None:
        source = production_source(EXECUTION)
        self.assertNotIn("fn instance_index_for_cluster_array_index(", source)
        self.assertNotIn("fn cluster_ordinal_for_stable_instance_key(", source)
        self.assertNotIn("cluster_ids.iter().position", source)
        self.assertNotIn(".clusters\n        .iter()\n        .enumerate()\n        .find(", source)

    def test_regression_and_release_performance_gates_are_present(self) -> None:
        source = EXECUTION.read_text(encoding="utf-8")
        record = RECORD.read_text(encoding="utf-8")
        self.assertIn(
            "indexed_execution_lookup_preserves_first_match_and_sorted_unique_ordinal",
            source,
        )
        self.assertIn(
            "indexed_execution_lookup_preserves_legacy_stable_instance_key",
            source,
        )
        self.assertIn(
            "RUNTIME09B_INDEXED_VIRTUAL_GEOMETRY_EXECUTION_BENCH_V1",
            source,
        )
        self.assertIn("indexed_p95.saturating_mul(10_000)", source)
        self.assertIn("97.15%", record)
        self.assertIn("32.82%", record)
        self.assertIn("51.82%", record)


if __name__ == "__main__":
    unittest.main()
