import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/frame_extract.rs"
CONSTRUCTION = ROOT / (
    "zircon_runtime_interface/src/ui/surface/render/frame_extract/construction.rs"
)
BENCH = ROOT / "zircon_runtime_interface/src/ui/surface/render/frame_extract/tests/construction_performance_tests.rs"


class RuntimeInterface03FrameCommandDirectoryPerformanceContract(unittest.TestCase):
    def test_directory_levels_move_owned_child_nodes(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        construction = CONSTRUCTION.read_text(encoding="utf-8")

        self.assertIn("mod construction;", source)
        self.assertIn("fn promote_directory_level", construction)
        self.assertIn("nodes.into_iter()", construction)
        self.assertIn("children.by_ref().take", construction)
        self.assertNotIn("nodes.chunks", construction)
        self.assertNotIn("children.to_vec", construction)

    def test_moved_directory_has_behavior_and_release_contract(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("mod construction_performance_tests", source)
        self.assertIn("promote_directory_level_cloning", bench)
        self.assertIn("RUNTIME_INTERFACE03_MOVED_FRAME_COMMAND_DIRECTORY_BENCH_V1", bench)
        self.assertIn("saturating_mul(5)", bench)


if __name__ == "__main__":
    unittest.main()
