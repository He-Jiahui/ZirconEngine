import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/template/asset/resource_ref/resource_kind.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/template/asset/resource_ref/resource_kind/tests/performance_tests.rs"


class RuntimeInterface03ResourceKindInferencePerformanceContract(unittest.TestCase):
    def test_path_inference_is_borrowed_and_does_not_materialize_segments(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn infer_from_path(")
        end = source.index("\nfn infer_from_path_name", start)
        body = source[start:end]

        self.assertIn("rsplit([", body)
        self.assertIn("infer_from_path_pair", body)
        self.assertNotIn("to_ascii_lowercase", body)
        self.assertNotIn("Vec<", body)
        self.assertNotIn("format!(", body)

    def test_uri_inference_is_borrowed_and_benchmark_is_present(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("eq_ignore_ascii_case", source)
        self.assertIn("ends_with_ascii_case", source)
        self.assertNotIn("let uri = uri.to_ascii_lowercase()", source)
        self.assertIn("legacy_infer_from_path_and_uri", bench)
        self.assertIn("RUNTIME_INTERFACE03_BORROWED_RESOURCE_KIND_BENCH_V1", bench)
        self.assertIn("saturating_mul(5)", bench)
        self.assertIn("mod performance_tests", source)


if __name__ == "__main__":
    unittest.main()
