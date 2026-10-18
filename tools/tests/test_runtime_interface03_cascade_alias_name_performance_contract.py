import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/design_tokens/cascade_registry.rs"
BENCH_SOURCE = ROOT / "zircon_runtime_interface/src/ui/design_tokens/cascade_registry/tests/custom_property_alias_performance_tests.rs"


class RuntimeInterface03CascadeAliasNamePerformanceContract(unittest.TestCase):
    def test_alias_name_uses_one_destination_buffer(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn custom_property_alias_name(")
        end = source.index("\n}\n", start) + 2
        body = source[start:end]

        self.assertIn("String::with_capacity(canonical_name.len() + 2)", body)
        self.assertIn('alias.push_str("--")', body)
        self.assertIn("canonical_name.split('.')", body)
        self.assertNotIn(".replace(", body)
        self.assertNotIn("format!(", body)

    def test_alias_projection_calls_single_buffer_encoder(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn custom_property_aliases(")
        end = source.index("\n#[cfg(test)]", start)
        body = source[start:end]

        self.assertIn("custom_property_alias_name(canonical_name)", body)

    def test_single_buffer_alias_name_has_release_benchmark(self) -> None:
        bench_source = BENCH_SOURCE.read_text(encoding="utf-8")

        self.assertIn(
            "RUNTIME_INTERFACE03_SINGLE_BUFFER_CASCADE_ALIAS_NAME_BENCH_V1",
            bench_source,
        )
        self.assertIn("saturating_mul(5)", bench_source)
        self.assertIn("saturating_mul(4)", bench_source)


if __name__ == "__main__":
    unittest.main()
