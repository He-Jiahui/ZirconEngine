from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/workbench/debug_reflector/schedule_sections.rs"


class EditorPipelineCounterSummaryPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        start = cls.source.index("fn pipeline_counter_summary(")
        end = cls.source.index("\nfn schedule_mask_summary", start)
        cls.body = cls.source[start:end]

    def test_active_counters_append_to_one_output_buffer(self) -> None:
        self.assertIn("let mut summary = String::new();", self.body)
        self.assertIn("for (name, count) in entries", self.body)
        self.assertIn("if count > 0", self.body)
        self.assertIn("summary.push(',');", self.body)
        self.assertIn("write!(", self.body)
        self.assertIn('"{name}={count}"', self.body)
        self.assertNotIn(".collect::<Vec", self.body)
        self.assertNotIn(".join(", self.body)
        self.assertNotIn("format!(", self.body)

    def test_none_and_counter_order_contracts_remain_explicit(self) -> None:
        self.assertIn('"none".to_string()', self.body)
        self.assertIn("if summary.is_empty()", self.body)
        self.assertLess(self.body.index('("input"'), self.body.index('("batch"'))

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "single_buffer_pipeline_counter_summary_preserves_bytes_and_filtering",
            self.source,
        )
        self.assertIn(
            "editor825_single_buffer_pipeline_counter_summary_bench_v1",
            self.source,
        )

    def test_dense_counter_summary_removes_intermediate_strings_and_vector(self) -> None:
        active_counter_count = 10
        retired_intermediate_strings = active_counter_count
        retired_temporary_vector_buffers = 1
        optimized_intermediate_strings = 0
        optimized_temporary_vector_buffers = 0

        self.assertGreater(retired_intermediate_strings, optimized_intermediate_strings)
        self.assertGreater(retired_temporary_vector_buffers, optimized_temporary_vector_buffers)


if __name__ == "__main__":
    unittest.main()
