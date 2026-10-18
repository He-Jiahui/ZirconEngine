import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
REPORT_SOURCE = ROOT / "zircon_runtime_interface/src/ui/pipeline/frame_report.rs"
STAGE_SOURCE = ROOT / "zircon_runtime_interface/src/ui/pipeline/stage.rs"
BENCHMARK_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/pipeline/frame_report/tests/performance_tests.rs"
)


class RuntimeInterface03MissingPipelineStagePerformanceContract(unittest.TestCase):
    def test_missing_stage_projection_uses_direct_runtime_indices(self) -> None:
        report = REPORT_SOURCE.read_text(encoding="utf-8")
        stage = STAGE_SOURCE.read_text(encoding="utf-8")

        self.assertIn("runtime_index", stage)
        self.assertIn("let mut present = [false; UiPipelineStage::ORDER.len()]", report)
        self.assertIn("report.stage.runtime_index()", report)
        self.assertIn("missing_required_stages_linear", report)

    def test_release_benchmark_and_behavior_oracle_are_present(self) -> None:
        benchmark = BENCHMARK_SOURCE.read_text(encoding="utf-8")

        self.assertIn(
            "runtime_interface03_batch18_missing_stage_index_preserves_archived_semantics",
            benchmark,
        )
        self.assertIn("RUNTIME_INTERFACE03_MISSING_STAGE_INDEX_BENCH_V1", benchmark)


if __name__ == "__main__":
    unittest.main()
