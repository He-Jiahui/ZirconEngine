import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / (
    "zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs"
)
BENCH_SOURCE = ROOT / "zircon_runtime_interface/src/ui/template/asset/compiler/binding_program/tests/reference_bitmap_performance_tests.rs"


class RuntimeInterface03BindingReferenceBitmapPerformanceContract(unittest.TestCase):
    def test_dense_binding_sequence_does_not_allocate_reference_bitmap(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    fn node_bindings_are_well_formed(&self)")
        end = source.index(
            "\n    #[cfg(test)]\n    fn node_bindings_are_well_formed_bitmap", start
        )
        body = source[start:end]

        self.assertNotIn("referenced_bindings", body)
        self.assertIn("expected_binding_index != self.bindings.len()", body)

    def test_dense_binding_sequence_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench_source = BENCH_SOURCE.read_text(encoding="utf-8")

        self.assertIn("node_bindings_are_well_formed_bitmap", source)
        self.assertIn(
            "RUNTIME_INTERFACE03_DENSE_BINDING_SEQUENCE_BENCH_V1",
            bench_source,
        )
        self.assertIn("saturating_mul(5)", bench_source)
        self.assertIn("saturating_mul(4)", bench_source)


if __name__ == "__main__":
    unittest.main()
