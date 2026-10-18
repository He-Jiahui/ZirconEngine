import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/persistent_sequence.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/surface/persistent_sequence/tests/directory_performance_tests.rs"


class RuntimeInterface03PersistentSequenceDirectoryPerformanceContract(
    unittest.TestCase
):
    def test_directory_promotion_moves_owned_child_nodes(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    fn from_segment_nodes(")
        end = source.index("\n}\n\nimpl<T: Clone>", start)
        body = source[start:end]

        self.assertIn("Self::promote_directory_level", body)
        self.assertIn("nodes.into_iter()", body)
        self.assertNotIn(".chunks(UI_PERSISTENT_SEQUENCE_DIRECTORY_FANOUT)", body)
        self.assertNotIn("children.to_vec()", body)

    def test_behavior_and_release_benchmark_are_present(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("mod directory_performance_tests", source)
        self.assertIn("cloned_directory_sequence", bench)
        self.assertIn(
            "runtime_interface03_batch66_67_moved_directory_nodes_preserve_layout",
            bench,
        )
        self.assertIn(
            "RUNTIME_INTERFACE03_MOVED_PERSISTENT_DIRECTORY_BENCH_V1", bench
        )
        self.assertIn("saturating_mul(10)", bench)


if __name__ == "__main__":
    unittest.main()
