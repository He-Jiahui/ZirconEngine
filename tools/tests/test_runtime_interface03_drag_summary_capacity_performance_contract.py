import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/component/drag.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/component/drag/tests/summary_performance_tests.rs"


class RuntimeInterface03DragSummaryCapacityPerformanceContract(unittest.TestCase):
    def test_drag_summary_presizes_the_required_output(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    pub fn summary(&self)")
        end = source.index("\n    }", start) + len("\n    }")
        body = source[start:end]

        self.assertIn("String::with_capacity", body)
        self.assertIn("kind.len()", body)
        self.assertIn("name.len()", body)
        self.assertIn("summary.push_str", body)
        self.assertNotIn("format!", body)

    def test_drag_summary_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("summary_growing", bench)
        self.assertIn("RUNTIME_INTERFACE03_PRESIZED_DRAG_SUMMARY_BENCH_V1", bench)
        self.assertIn("saturating_mul(5)", bench)
        self.assertIn("mod summary_performance_tests", source)


if __name__ == "__main__":
    unittest.main()
