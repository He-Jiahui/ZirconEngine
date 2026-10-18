from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/host/play_hierarchy_projection.rs"
REGRESSIONS = SOURCE.parent / "tests/play_hierarchy_projection.rs"
BENCHMARKS = SOURCE.parent / "tests/play_hierarchy_project_perf_batch.rs"


class EditorPlayHierarchyChangedRowCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.regressions = REGRESSIONS.read_text(encoding="utf-8")
        cls.benchmarks = BENCHMARKS.read_text(encoding="utf-8")
        start = cls.source.index("    fn apply_rows(")
        end = cls.source.index("\n    fn apply_not_modified", start)
        cls.body = cls.source[start:end]
        start = cls.source.index("fn same_topology_changed_count(")
        end = cls.source.index("\nfn hierarchy_anchor(", start)
        cls.topology = cls.source[start:end]

    def test_changed_rows_and_anchor_bounds_are_reserved_before_append(self) -> None:
        self.assertIn(
            "same_topology_changed_count(&previous.rows, &rows)", self.body
        )
        self.assertIn("if generation == previous.generation {", self.body)
        self.assertIn("Some(0)", self.body)
        self.assertLess(
            self.body.index("SameGenerationMutation { generation }"),
            self.body.index("Some(0)"),
        )
        self.assertIn("changed_count += usize::from(previous != current);", self.topology)
        self.assertIn("Some(changed_count)", self.topology)
        self.assertIn(
            "let mut changed_rows = Vec::with_capacity(changed_count);", self.body
        )
        self.assertNotIn("Vec::with_capacity(rows.len())", self.body)
        self.assertIn(
            "let mut changed_anchors = Vec::with_capacity(changed_rows.len());",
            self.body,
        )
        self.assertLess(
            self.body.index("Vec::with_capacity(changed_count)"),
            self.body.index("changed_rows.push("),
        )
        self.assertLess(
            self.body.index("Vec::with_capacity(changed_rows.len())"),
            self.body.index("changed_anchors.push("),
        )
        self.assertIn(
            '#[path = "tests/play_hierarchy_project_perf_batch.rs"]', self.source
        )
        self.assertIn(
            "editor815_play_hierarchy_changed_row_capacity_bench_v1", self.benchmarks
        )

    def test_sparse_order_and_noop_delta_semantics_remain_explicit(self) -> None:
        self.assertIn(
            "for (previous_row, current_row) in previous.rows.iter().zip(rows.iter())",
            self.body,
        )
        self.assertIn("if previous_row != current_row", self.body)
        self.assertIn("changed_rows.push(current_row.clone());", self.body)
        self.assertIn("for row in &changed_rows", self.body)
        self.assertIn("changed_anchors.push(hierarchy_anchor(row));", self.body)
        self.assertIn("if generation == previous.generation && changed_rows.is_empty()", self.body)
        self.assertIn("SceneInspectionMessage::delta(", self.body)
        self.assertNotIn(".filter_map(", self.body)
        self.assertNotIn("changed_rows.iter().map(hierarchy_anchor).collect()", self.body)

    def test_empty_change_set_keeps_zero_capacity_path(self) -> None:
        self.assertIn("const ROW_COUNT: usize = 4096;", self.regressions)
        self.assertIn("for changed_count in [0, 1, ROW_COUNT]", self.regressions)
        self.assertIn(
            "apply_same_topology_change(ROW_COUNT, changed_count)", self.regressions
        )
        self.assertIn(
            "assert_eq!(capacity.initial_capacity, changed_count);", self.regressions
        )
        self.assertIn(
            "assert_eq!(capacity.final_capacity, changed_count);", self.regressions
        )
        self.assertIn("assert_eq!(capacity.growth_events, 0);", self.regressions)
        self.assertIn("assert!(fragment.is_none());", self.regressions)
        self.assertIn("let initial_capacity = changed_rows.capacity();", self.body)
        self.assertIn("if changed_count != 0 {", self.body)
        self.assertLess(
            self.body.index("if changed_count != 0 {"),
            self.body.index("for (previous_row, current_row)"),
        )
        self.assertIn("let previous_capacity = changed_rows.capacity();", self.body)
        self.assertIn("if changed_rows.capacity() != previous_capacity", self.body)

    def test_deterministic_dense_change_projection_removes_growth(self) -> None:
        self.assertIn(
            "apply_same_topology_change(ROW_COUNT, ROW_COUNT)", self.benchmarks
        )
        self.assertIn("apply_same_topology_change(ROW_COUNT, 0)", self.benchmarks)
        self.assertIn("apply_same_topology_change(ROW_COUNT, 1)", self.benchmarks)
        self.assertIn(
            "let optimized_growth_events = dense_capacity.growth_events;",
            self.benchmarks,
        )
        self.assertIn("let mut legacy_changed_rows = Vec::new();", self.benchmarks)
        self.assertIn("legacy_changed_rows.push(row.clone());", self.benchmarks)
        self.assertIn(
            "if legacy_changed_rows.capacity() != previous_capacity", self.benchmarks
        )
        self.assertIn("assert_eq!(legacy_growth_events, 11);", self.benchmarks)
        self.assertIn("assert_eq!(optimized_growth_events, 0);", self.benchmarks)
        self.assertIn(
            "EDITOR815_PLAY_HIERARCHY_CHANGED_ROW_CAPACITY_BENCH_V1", self.benchmarks
        )
        self.assertNotIn("let optimized_growth_events = 0", self.benchmarks)
        self.assertNotIn("geometric_growth_events", self.benchmarks)


if __name__ == "__main__":
    unittest.main()
