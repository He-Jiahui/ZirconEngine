from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PAINT_INDEX = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/"
    "template_node/index.rs"
)
PRESENTATION_GENERATION = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/data/"
    "presentation_generation.rs"
)
TEMPLATE_DRAW = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_node_pipeline/draw.rs"
)
COMPONENTIZED_OVERLAY = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/"
    "scene_layers/overlay/componentized.rs"
)


class EditorDamagePaintIndexPerformanceContractTests(unittest.TestCase):
    def source(self) -> str:
        return PAINT_INDEX.read_text(encoding="utf-8")

    def test_build_preorders_full_and_cell_paint_rows(self) -> None:
        source = self.source()
        constructor = source.split("fn new(indexed_nodes", 1)[1]
        constructor = constructor.split("fn rows_for_clip", 1)[0]

        self.assertIn("paint_order_rows", constructor)
        self.assertIn(
            "sort_rows_in_paint_order(&index.indexed_nodes, &mut paint_order_rows)",
            constructor,
        )
        self.assertIn(".values_mut()", constructor)
        self.assertIn("sort_rows_in_paint_order(&indexed_nodes, rows)", constructor)

    def test_full_and_single_cell_queries_stream_borrowed_preordered_rows(self) -> None:
        source = self.source()
        query = source.split("fn visit_rows_for_clip", 1)[1]
        query = query.split("fn sort_rows_in_paint_order", 1)[0]

        full_visit = query.index("visit_rows(self.paint_order_rows.as_slice(), visit)")
        single_branch = query.index("if min_x == max_x && min_y == max_y")
        single_visit = query.index("visit_rows(rows, visit)", single_branch)
        scratch_lock = query.index(".query_scratch")
        query_sort = query.index("self.sort_rows_in_paint_order(&mut scratch.rows);")

        self.assertLess(full_visit, single_branch)
        self.assertLess(single_branch, single_visit)
        self.assertLess(single_visit, scratch_lock)
        self.assertLess(scratch_lock, query_sort)
        self.assertNotIn("paint_order_rows.as_ref().clone()", query)
        self.assertNotIn(".get(&(min_x, min_y)).cloned()", query)

    def test_multi_cell_query_reuses_index_owned_scratch(self) -> None:
        source = self.source()
        index_type = source.split("struct HostTemplateNodePaintIndex", 1)[1]
        index_type = index_type.split("impl HostWorkbenchHitIndex", 1)[0]
        query = source.split("fn visit_rows_for_clip", 1)[1]
        query = query.split("fn sort_rows_in_paint_order", 1)[0]

        self.assertIn("query_scratch: Arc<Mutex<PaintRowQueryScratch>>", index_type)
        self.assertIn("PaintRowQueryScratch::with_capacity", source)
        self.assertIn("scratch.seen.clear()", query)
        self.assertIn("scratch.rows.clear()", query)
        self.assertIn('"ui.paint_index.query_scratch_growth_count"', query)
        self.assertNotIn("let mut seen = HashSet::new()", query)
        self.assertNotIn("let mut rows = Vec::new()", query)

    def test_template_paint_pipeline_consumes_rows_through_visitor(self) -> None:
        generation = PRESENTATION_GENERATION.read_text(encoding="utf-8")
        draw = TEMPLATE_DRAW.read_text(encoding="utf-8")
        componentized = COMPONENTIZED_OVERLAY.read_text(encoding="utf-8")

        self.assertIn("pub(crate) fn visit_paint_workbench_rows", generation)
        self.assertIn("index.visit_paint_rows_for_nodes", generation)
        self.assertNotIn("pub(crate) fn paint_workbench_row_indices", generation)
        self.assertIn("visit_paint_workbench_rows", draw)
        self.assertNotIn("paint_workbench_row_indices", draw)
        self.assertIn("fn stream_row_visit_indices", componentized)
        self.assertIn("index.visit_paint_rows_for_subtree", componentized)
        self.assertNotIn("index.paint_rows_for_subtree", componentized)

    def test_rust_regression_proves_reuse_without_query_sort(self) -> None:
        source = self.source()
        regression = source.split(
            "fn paint_index_streams_build_time_order_and_reuses_multi_cell_scratch",
            1,
        )[1]
        regression = regression.split("fn paint_node", 1)[0]

        self.assertEqual(regression.count("query_sort_count_for_test(), 0"), 2)
        self.assertIn("query_scratch_capacity_for_test()", regression)
        self.assertIn("query_scratch_allocation_count_for_test()", regression)


if __name__ == "__main__":
    unittest.main()
