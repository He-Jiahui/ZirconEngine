import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/binding/model/model_context.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/binding/model/model_context/tests/resolve_performance_tests.rs"


class RuntimeInterface03ModelContextResolvePerformanceContract(unittest.TestCase):
    def test_model_context_resolves_each_layer_before_cloning(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    pub fn resolve(parent:")
        end = source.index("\n    }", start) + len("\n    }")
        body = source[start:end]

        self.assertIn("resolve_provider", body)
        self.assertNotIn("parent.cloned()", body)
        self.assertNotIn("provider_mut", body)
        self.assertIn("fn resolve_provider", source)

    def test_model_context_resolution_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("resolve_cloning_parent", bench)
        self.assertIn("RUNTIME_INTERFACE03_SELECTIVE_MODEL_CONTEXT_CLONE_BENCH_V1", bench)
        self.assertIn("saturating_mul(2)", bench)
        self.assertIn("mod resolve_performance_tests", source)


if __name__ == "__main__":
    unittest.main()
