import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/persistent_sequence.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/surface/persistent_sequence/tests/extend_performance_tests.rs"


class RuntimeInterface03PersistentSequenceExtendPerformanceContract(unittest.TestCase):
    def test_extend_streams_existing_and_appended_items_into_segments(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("impl<T: Clone> Extend<T> for UiPersistentSequence<T>")
        end = source.index("\nimpl<T: PartialEq> PartialEq", start)
        body = source[start:end]

        self.assertIn("self.iter().cloned().chain(iter)", body)
        self.assertIn("Self::from_owned_iter", body)
        self.assertNotIn("self.to_vec()", body)

    def test_behavior_and_release_benchmark_are_present(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("mod extend_performance_tests", source)
        self.assertIn("buffered_extend", bench)
        self.assertIn(
            "runtime_interface03_batch64_65_streamed_extend_preserves_layout",
            bench,
        )
        self.assertIn(
            "RUNTIME_INTERFACE03_STREAMED_PERSISTENT_EXTEND_BENCH_V1", bench
        )
        self.assertIn("saturating_mul(5)", bench)


if __name__ == "__main__":
    unittest.main()
