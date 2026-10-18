from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/binding/model/conversion.rs"
TEST = ROOT / "zircon_runtime_interface/src/ui/binding/model/conversion/tests/validation_performance_tests.rs"


class RuntimeInterface03BindingConversionValidationPerformanceContract(unittest.TestCase):
    def test_validation_scans_segments_and_characters_in_one_pass(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn validate_conversion_id(")
        end = source.index("\n}", start) + 2
        body = source[start:end]
        self.assertIn("char_indices()", body)
        self.assertIn("EmptySegment", body)
        self.assertNotIn("split('.')", body)

    def test_behavior_oracle_and_release_benchmark_are_present(self) -> None:
        test = TEST.read_text(encoding="utf-8")
        self.assertIn("single_pass_conversion_id_validation_preserves_two_pass_results", test)
        self.assertIn("#[ignore", test)
        self.assertIn(
            "RUNTIME_INTERFACE03_SINGLE_PASS_CONVERSION_ID_BENCH_V1",
            test,
        )
        self.assertIn("single_pass_samples[p95]", test)


if __name__ == "__main__":
    unittest.main()
