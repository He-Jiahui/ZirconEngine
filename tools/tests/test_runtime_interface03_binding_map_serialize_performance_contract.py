import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/binding/model/binding_value/types.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/binding/model/binding_value/types/tests/map_serialize_performance_tests.rs"


class RuntimeInterface03BindingMapSerializePerformanceContract(unittest.TestCase):
    def test_binding_map_serializes_entries_without_intermediate_vec(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("impl Serialize for UiBindingMap")
        end = source.index("\n}\n\nimpl<'de> Deserialize", start) + len("\n}")
        body = source[start:end]

        self.assertIn("serialize_seq(Some(self.0.len()))", body)
        self.assertIn("serialize_element", body)
        self.assertIn("sequence.end()", body)
        self.assertNotIn("collect::<Vec<_>>()", body)

    def test_binding_map_streaming_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("map_entries_allocating", bench)
        self.assertIn("RUNTIME_INTERFACE03_STREAMED_BINDING_MAP_BENCH_V1", bench)
        self.assertIn("saturating_mul(2)", bench)
        self.assertIn("mod map_serialize_performance_tests", source)


if __name__ == "__main__":
    unittest.main()
