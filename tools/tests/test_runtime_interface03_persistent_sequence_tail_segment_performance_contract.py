import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/persistent_sequence.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/surface/persistent_sequence/tests/tail_segment_performance_tests.rs"


class RuntimeInterface03PersistentSequenceTailSegmentPerformanceContract(
    unittest.TestCase
):
    def test_exact_iterators_use_exact_tail_segment_capacity(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    fn from_owned_iter")
        end = source.index("\n    fn from_segment_nodes", start)
        body = source[start:end]

        self.assertIn("Self::owned_segment_capacity(&items)", body)
        self.assertIn("fn owned_segment_capacity", source)
        self.assertIn("upper_bound == Some(lower_bound)", source)
        self.assertIn("UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE", source)

    def test_behavior_and_release_benchmark_are_present(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("mod tail_segment_performance_tests", source)
        self.assertIn("fixed_capacity_sequence", bench)
        self.assertIn(
            "runtime_interface03_batch66_67_exact_tail_capacity_preserves_layout",
            bench,
        )
        self.assertIn(
            "RUNTIME_INTERFACE03_EXACT_PERSISTENT_TAIL_BENCH_V1", bench
        )
        self.assertIn("saturating_mul(10)", bench)


if __name__ == "__main__":
    unittest.main()
