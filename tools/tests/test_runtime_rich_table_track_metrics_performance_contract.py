from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
CELL_LAYOUT = ROOT / (
    "zircon_runtime/src/ui/text/layout_engine/rich_table/cell_layout.rs"
)
LAYOUT = ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/layout.rs"

# 读取富文本表格轨道指标，确认间距几何由单一权威计算并供各布局阶段复用。
class RuntimeRichTableTrackMetricsPerformanceContract(unittest.TestCase):
    def test_one_track_metrics_authority_owns_gap_aware_geometry(self) -> None:
        source = CELL_LAYOUT.read_text(encoding="utf-8")

        self.assertIn("struct TrackMetrics", source)
        self.assertIn("extents: Vec<f32>", source)
        self.assertIn("origins: Vec<f32>", source)
        self.assertIn("total_extent: f32", source)
        self.assertIn("pub(super) fn origin(&self", source)
        self.assertIn("pub(super) fn span_extent(&self", source)
        self.assertIn("pub(super) fn total_extent(&self", source)
        # BUG: [CR-ZCA-R078-0001] 此处仍读取实现原文，但要求的测试名已拆到直属测试文件，当前会触发 AssertionError。
        # 上方七项实现断言均成立；条件测试挂载不会把另一个文件的文本并入此读取结果。
        # 应另行修正守卫的读取对象；本标记不改变断言，也不表示产品表格布局有缺陷。
        self.assertIn("track_metrics_include_gap_in_origins_spans_and_total", source)
        self.assertIn("empty_and_clamped_track_queries_are_safe", source)
        self.assertIn(
            "gap_aware_metrics_map_consistently_across_writing_modes", source
        )
        span = source.split("pub(super) fn span_extent", 1)[1].split(
            "pub(super) fn total_extent", 1
        )[0]
        self.assertNotIn(".iter()", span)
        self.assertNotIn(".sum", span)

    def test_table_layout_consumes_metrics_for_every_geometry_phase(self) -> None:
        source = LAYOUT.read_text(encoding="utf-8")

        self.assertIn(
            "TrackMetrics::new(column_extents, column_gap, geometry_budget)", source
        )
        self.assertIn("TrackMetrics::new(row_extents, 0.0, geometry_budget)", source)
        self.assertGreaterEqual(source.count("column_metrics.span_extent("), 2)
        self.assertGreaterEqual(source.count("row_metrics.span_extent("), 1)
        self.assertIn("column_metrics.total_extent()", source)
        self.assertIn("row_metrics.total_extent()", source)
        self.assertNotIn("track_origins", source)
        self.assertNotIn("track_span_extent", source)


if __name__ == "__main__":
    unittest.main()
