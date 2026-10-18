import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/design_tokens/cascade_registry.rs"
BENCH_SOURCE = ROOT / "zircon_runtime_interface/src/ui/design_tokens/cascade_registry/tests/custom_property_alias_performance_tests.rs"


class RuntimeInterface03CascadeAliasProjectionPerformanceContract(unittest.TestCase):
    def test_custom_property_aliases_borrow_canonical_names(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("pub(super) fn cascade_token_values(")
        end = source.index("\nfn custom_property_aliases(", start)
        body = source[start:end]
        alias_start = source.index("fn custom_property_aliases(", end)
        alias_end = source.index("\n#[cfg(test)]", alias_start)
        alias_body = source[alias_start:alias_end]

        self.assertNotIn(".keys().cloned()", body)
        self.assertIn("values.extend(custom_property_aliases(&values))", body)
        self.assertIn(".keys()", alias_body)
        self.assertIn(".map", alias_body)

    def test_custom_property_alias_projection_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench_source = BENCH_SOURCE.read_text(encoding="utf-8")

        self.assertIn("custom_property_aliases_cloning", source)
        self.assertIn(
            "RUNTIME_INTERFACE03_BORROWED_CASCADE_ALIAS_BENCH_V1",
            bench_source,
        )
        self.assertIn("saturating_mul(5)", bench_source)
        self.assertIn("saturating_mul(4)", bench_source)


if __name__ == "__main__":
    unittest.main()
