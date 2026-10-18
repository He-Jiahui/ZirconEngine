import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/batch/clip.rs"
BENCHMARK_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/batch/clip/tests/performance_tests.rs"
)


class RuntimeInterface03ClipStateResolutionPerformanceContract(unittest.TestCase):
    def test_clip_resolution_reuses_the_interner_identity_index(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")

        self.assertIn("self.states.indices.get(&identity)", source)
        self.assertIn("self.states.states.get(index)", source)
        self.assertIn("resolve_linear", source)

    def test_release_benchmark_and_behavior_oracle_are_present(self) -> None:
        benchmark = BENCHMARK_SOURCE.read_text(encoding="utf-8")

        self.assertIn(
            "runtime_interface03_batch17_clip_resolution_preserves_deserialized_fallback",
            benchmark,
        )
        self.assertIn("RUNTIME_INTERFACE03_CLIP_STATE_RESOLUTION_BENCH_V1", benchmark)


if __name__ == "__main__":
    unittest.main()
