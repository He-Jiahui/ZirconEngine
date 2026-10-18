import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/persistent_sequence.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/surface/persistent_sequence/tests/construction_performance_tests.rs"


class RuntimeInterface03PersistentSequenceConstructionPerformanceContract(
    unittest.TestCase
):
    def test_from_iterator_streams_directly_into_segments(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("impl<T> FromIterator<T> for UiPersistentSequence<T>")
        end = source.index("\nimpl<T: Clone> Extend<T>", start)
        body = source[start:end]

        self.assertIn("Self::from_owned_iter(iter.into_iter())", body)
        self.assertNotIn("collect::<Vec<_>>()", body)
        self.assertIn("fn from_owned_iter", source)
        self.assertIn("UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE", source)

    def test_behavior_and_release_benchmark_are_present(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("mod construction_performance_tests", source)
        self.assertIn("buffered_sequence", bench)
        self.assertIn(
            "runtime_interface03_batch62_63_streamed_persistent_sequence_preserves_layout",
            bench,
        )
        self.assertIn(
            "RUNTIME_INTERFACE03_STREAMED_PERSISTENT_SEQUENCE_BENCH_V1", bench
        )
        self.assertIn("saturating_mul(5)", bench)


if __name__ == "__main__":
    unittest.main()
