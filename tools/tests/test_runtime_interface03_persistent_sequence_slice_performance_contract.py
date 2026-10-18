import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/persistent_sequence.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/surface/persistent_sequence/tests/construction_performance_tests.rs"


class RuntimeInterface03PersistentSequenceSlicePerformanceContract(unittest.TestCase):
    def test_from_slice_clones_directly_into_segments(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    pub fn from_slice(items: &[T]) -> Self {")
        end = source.index("\n    pub fn get_mut", start)
        body = source[start:end]

        self.assertIn("Self::from_owned_iter(items.iter().cloned())", body)
        self.assertNotIn("items.to_vec()", body)

    def test_behavior_and_release_benchmark_are_present(self) -> None:
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("buffered_slice", bench)
        self.assertIn(
            "runtime_interface03_batch62_63_streamed_slice_preserves_layout", bench
        )
        self.assertIn(
            "RUNTIME_INTERFACE03_STREAMED_PERSISTENT_SLICE_BENCH_V1", bench
        )
        self.assertIn("saturating_mul(5)", bench)


if __name__ == "__main__":
    unittest.main()
