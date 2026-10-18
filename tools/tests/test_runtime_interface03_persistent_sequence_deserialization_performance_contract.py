import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/persistent_sequence.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/surface/persistent_sequence/tests/deserialization_performance_tests.rs"


class RuntimeInterface03PersistentSequenceDeserializationPerformanceContract(
    unittest.TestCase
):
    def test_deserialization_streams_seq_access_into_segments(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("impl<'de, T> Deserialize<'de> for UiPersistentSequence<T>")
        end = source.index("\nimpl<'a, T> IntoIterator", start)
        body = source[start:end]

        self.assertIn("deserializer.deserialize_seq", body)
        self.assertIn("UiPersistentSequenceVisitor", body)
        self.assertNotIn("Vec::<T>::deserialize", body)
        self.assertIn("A: SeqAccess<'de>", source)
        self.assertIn("sequence.next_element::<T>()?", source)

    def test_behavior_and_release_benchmark_are_present(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("mod deserialization_performance_tests", source)
        self.assertIn("buffered_deserialize", bench)
        self.assertIn(
            "runtime_interface03_batch64_65_streamed_deserialize_preserves_layout",
            bench,
        )
        self.assertIn(
            "RUNTIME_INTERFACE03_STREAMED_PERSISTENT_DESERIALIZE_BENCH_V1",
            bench,
        )
        self.assertIn("saturating_mul(10)", bench)


if __name__ == "__main__":
    unittest.main()
