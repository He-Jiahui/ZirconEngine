import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/binding/model/parser.rs"
BENCH_SOURCE = ROOT / "zircon_runtime_interface/src/ui/binding/model/parser/tests/constructed_entry_performance_tests.rs"


class RuntimeInterface03BindingConstructedEntryMovePerformanceContract(unittest.TestCase):
    def test_record_and_map_consume_owned_arguments_without_clone(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        record_start = source.index("    fn construct_record(")
        map_start = source.index("    fn construct_map(", record_start)
        map_end = source.index(
            "\n    #[cfg(test)]\n    fn construct_record_cloning", map_start
        )
        record_body = source[record_start:map_start]
        map_body = source[map_start:map_end]

        self.assertIn("arguments.into_iter()", record_body)
        self.assertIn("arguments.into_iter()", map_body)
        self.assertNotIn(".clone()", record_body)
        self.assertNotIn(".clone()", map_body)

    def test_constructed_entry_move_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench_source = BENCH_SOURCE.read_text(encoding="utf-8")

        self.assertIn("construct_record_cloning", source)
        self.assertIn("construct_map_cloning", source)
        self.assertIn(
            "RUNTIME_INTERFACE03_MOVED_CONSTRUCTED_ENTRY_BENCH_V1",
            bench_source,
        )
        self.assertIn("saturating_mul(5)", bench_source)
        self.assertIn("saturating_mul(4)", bench_source)


if __name__ == "__main__":
    unittest.main()
