import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/pipeline/frame_report.rs"
BENCHMARK_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/pipeline/frame_report/tests/performance_tests.rs"
)


class RuntimeInterface03PipelineStageReportLookupPerformanceContract(unittest.TestCase):
    def test_canonical_pipeline_reports_use_direct_stage_indices(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")

        self.assertIn("stage.runtime_index()", source)
        self.assertIn(".stages", source)
        self.assertIn(".get(index)", source)
        self.assertIn("stage_report_linear", source)

    def test_release_benchmark_and_behavior_oracle_are_present(self) -> None:
        benchmark = BENCHMARK_SOURCE.read_text(encoding="utf-8")

        self.assertIn(
            "runtime_interface03_batch19_stage_report_index_preserves_sparse_fallback",
            benchmark,
        )
        self.assertIn("RUNTIME_INTERFACE03_STAGE_REPORT_INDEX_BENCH_V1", benchmark)


if __name__ == "__main__":
    unittest.main()
