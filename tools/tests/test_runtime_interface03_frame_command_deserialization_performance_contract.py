import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/frame_extract.rs"
CONSTRUCTION = ROOT / (
    "zircon_runtime_interface/src/ui/surface/render/frame_extract/construction.rs"
)
BENCH = ROOT / "zircon_runtime_interface/src/ui/surface/render/frame_extract/tests/construction_performance_tests.rs"


class RuntimeInterface03FrameCommandDeserializationPerformanceContract(unittest.TestCase):
    def test_deserialization_streams_commands_into_leaf_segments(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        construction = CONSTRUCTION.read_text(encoding="utf-8")

        self.assertIn("mod construction;", source)
        self.assertIn("SeqAccess", construction)
        self.assertIn("Visitor", construction)
        self.assertIn("deserializer.deserialize_seq", construction)
        self.assertNotIn("Vec::<UiRenderCommand>::deserialize", construction)
        self.assertNotIn("Vec::<UiRenderCommand>::deserialize", source)

    def test_streamed_deserialization_has_behavior_and_release_contract(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("mod construction_performance_tests", source)
        self.assertIn("deserialize_frame_commands_cloning", bench)
        self.assertIn("RUNTIME_INTERFACE03_STREAMED_FRAME_COMMAND_DESERIALIZE_BENCH_V1", bench)
        self.assertIn("saturating_mul(10)", bench)


if __name__ == "__main__":
    unittest.main()
