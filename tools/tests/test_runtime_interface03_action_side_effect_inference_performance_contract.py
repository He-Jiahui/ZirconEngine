import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / (
    "zircon_runtime_interface/src/ui/template/asset/action_policy/"
    "side_effect_class.rs"
)
BENCH = ROOT / "zircon_runtime_interface/src/ui/template/asset/action_policy/side_effect_class/tests/performance_tests.rs"


class RuntimeInterface03ActionSideEffectInferencePerformanceContract(unittest.TestCase):
    def test_side_effect_inference_borrows_route_and_action(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    pub fn infer(")
        end = source.index("\n    }", start) + len("\n    }")
        body = source[start:end]

        self.assertIn("contains_ascii_case_insensitive", body)
        self.assertNotIn("to_ascii_lowercase", body)
        self.assertNotIn("format!", body)
        self.assertIn("window.eq_ignore_ascii_case", source)

    def test_side_effect_inference_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("infer_allocating", bench)
        self.assertIn("RUNTIME_INTERFACE03_BORROWED_SIDE_EFFECT_INFERENCE_BENCH_V1", bench)
        self.assertIn("saturating_mul(5)", bench)
        self.assertIn("mod performance_tests", source)


if __name__ == "__main__":
    unittest.main()
