import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/hit.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/surface/hit/tests/route_validation_performance_tests.rs"


class RuntimeInterface03HitPathRouteValidationPerformanceContract(unittest.TestCase):
    def test_route_validation_compares_borrowed_iterators(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    pub fn with_route(")
        end = source.index("\n    pub fn has_consistent_route", start)
        body = source[start:end]

        self.assertIn("bubble_route", body)
        self.assertIn(".eq(root_to_leaf.iter().rev().copied())", body)
        self.assertIn("root_to_leaf.iter().rev().copied()", body)
        self.assertNotIn("expected_bubble_route: Vec", body)
        self.assertNotIn("collect()", body)

    def test_route_validation_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("validate_route_allocating", bench)
        self.assertIn("validate_route_borrowed", bench)
        self.assertIn("RUNTIME_INTERFACE03_BORROWED_HIT_ROUTE_VALIDATION_BENCH_V1", bench)
        self.assertIn("saturating_mul(5)", bench)
        self.assertIn("mod route_validation_performance_tests", source)


if __name__ == "__main__":
    unittest.main()
