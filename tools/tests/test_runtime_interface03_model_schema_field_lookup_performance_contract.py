import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/binding/model/model_schema.rs"
BENCH_SOURCE = ROOT / "zircon_runtime_interface/src/ui/binding/model/model_schema/tests/field_lookup_performance_tests.rs"


class RuntimeInterface03ModelSchemaFieldLookupPerformanceContract(unittest.TestCase):
    def test_field_lookup_uses_binary_search_with_compatibility_fallback(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    pub fn field(&self")
        end = source.index("\n    }", start) + len("\n    }")
        body = source[start:end]

        self.assertIn("binary_search_by", body)
        self.assertIn("self.fields.get(index)", body)
        self.assertIn("or_else", body)
        self.assertIn("self.fields.iter().find", body)

    def test_model_schema_field_lookup_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench_source = BENCH_SOURCE.read_text(encoding="utf-8")

        self.assertIn("fn field_linear", source)
        self.assertIn(
            "RUNTIME_INTERFACE03_BINARY_MODEL_FIELD_LOOKUP_BENCH_V1",
            bench_source,
        )
        self.assertIn("saturating_mul(5)", bench_source)


if __name__ == "__main__":
    unittest.main()
