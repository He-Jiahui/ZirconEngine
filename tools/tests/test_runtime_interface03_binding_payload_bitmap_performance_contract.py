import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / (
    "zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs"
)
BENCH_SOURCE = ROOT / "zircon_runtime_interface/src/ui/template/asset/compiler/binding_program/tests/payload_bitmap_performance_tests.rs"


class RuntimeInterface03BindingPayloadBitmapPerformanceContract(unittest.TestCase):
    def test_payload_field_bitmap_is_reused_across_bindings(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    pub fn is_well_formed(&self)")
        end = source.index("\n    pub fn handle_for_node_binding", start)
        body = source[start:end]

        bitmap_index = body.index("let mut seen_payload_fields")
        binding_loop_index = body.index("self.bindings.iter().all")
        self.assertLess(bitmap_index, binding_loop_index)
        self.assertEqual(body.count("vec![false; self.properties.len()]"), 1)
        self.assertIn("seen_payload_fields[property_index] = false", source)

    def test_payload_field_bitmap_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench_source = BENCH_SOURCE.read_text(encoding="utf-8")

        self.assertIn("payload_fields_are_well_formed_allocating", source)
        self.assertIn(
            "RUNTIME_INTERFACE03_REUSED_PAYLOAD_FIELD_BITMAP_BENCH_V1",
            bench_source,
        )
        self.assertIn("saturating_mul(2)", bench_source)


if __name__ == "__main__":
    unittest.main()
