import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/brush.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/surface/render/brush/tests/material_key_performance_tests.rs"


class RuntimeInterface03MaterialResourceKeyPerformanceContract(unittest.TestCase):
    def test_material_resource_id_uses_one_exact_capacity_buffer(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        impl_start = source.index("impl UiMaterialBrushPayload")
        start = source.index("    pub fn resource_key(&self)", impl_start)
        end = source.index("\n    }", start) + len("\n    }")
        body = source[start:end]

        self.assertIn("material_resource_id", body)
        self.assertNotIn("format!(", body)
        self.assertIn("String::with_capacity", source)
        self.assertIn("id.push('#')", source)

    def test_material_resource_id_has_behavior_and_release_contract(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("mod material_key_performance_tests", source)
        self.assertIn("material_resource_id_formatting", bench)
        self.assertIn("RUNTIME_INTERFACE03_SINGLE_BUFFER_MATERIAL_KEY_BENCH_V1", bench)
        self.assertIn("saturating_mul(5)", bench)


if __name__ == "__main__":
    unittest.main()
