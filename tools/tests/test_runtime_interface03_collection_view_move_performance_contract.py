import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/binding/model/parser.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/binding/model/parser/tests/collection_view_performance_tests.rs"


class RuntimeInterface03CollectionViewMovePerformanceContract(unittest.TestCase):
    def test_collection_view_moves_fixed_arguments_without_vec_shifts(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    fn construct_collection_view(")
        end = source.index("\n    fn parse_array", start)
        body = source[start:end]

        self.assertIn("let mut arguments = arguments.into_iter()", body)
        self.assertIn("arguments.get(2)", body)
        self.assertIn("unsigned_argument(constructor, &arguments, 7)", body)
        self.assertIn("unsigned_argument(constructor, &arguments, 1)", body)
        self.assertNotIn(".remove(", body)

    def test_collection_view_move_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("construct_collection_view_removing", bench)
        self.assertIn("move_arguments_removing", bench)
        self.assertIn("move_arguments_consuming", bench)
        self.assertIn("RUNTIME_INTERFACE03_MOVED_COLLECTION_VIEW_ARGUMENTS_BENCH_V1", bench)
        self.assertIn("saturating_mul(5)", bench)
        self.assertIn("mod collection_view_performance_tests", source)


if __name__ == "__main__":
    unittest.main()
