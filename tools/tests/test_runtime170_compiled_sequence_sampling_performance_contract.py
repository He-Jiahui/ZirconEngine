from tools.tests.rust_test_files import read_rust_test_file
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/animation/sequence/channel_sample.rs"
WORLD_COMPILER = REPO_ROOT / "zircon_runtime/src/animation/sequence/compiled.rs"
COMPILER_MODEL = REPO_ROOT / "zircon_runtime/src/core/framework/animation/compiler/sequence/model.rs"
PLUGIN_PIPELINE = (
    REPO_ROOT / "zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs"
)
WORLD_COMPILER_TESTS = REPO_ROOT / "zircon_runtime/src/animation/sequence/tests/cases.rs"
PLAN = (
    REPO_ROOT
    / "docs/plans/optimize/zircon_runtime/170/2026-09-01-compiled-sequence-channel-sampling-hard-cut.md"
)


class Runtime170CompiledSequenceSamplingPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.world_compiler = WORLD_COMPILER.read_text(encoding="utf-8")
        cls.compiler_model = COMPILER_MODEL.read_text(encoding="utf-8")
        cls.plugin_pipeline = PLUGIN_PIPELINE.read_text(encoding="utf-8")
        cls.world_compiler_tests = WORLD_COMPILER_TESTS.read_text(encoding="utf-8")
        cls.plan = PLAN.read_text(encoding="utf-8")

    def test_release_probe_uses_101_samples_and_nearest_rank_p95(self) -> None:
        self.assertIn("const SAMPLE_COUNT: usize = 101;", read_rust_test_file("zircon_runtime/src/animation/sequence/tests/channel_sample.rs"))
        self.assertIn("let p95 = percentile_index(SAMPLE_COUNT, 95);", read_rust_test_file("zircon_runtime/src/animation/sequence/tests/channel_sample.rs"))
        self.assertIn("fn percentile_index(sample_count: usize, percentile: usize)", read_rust_test_file("zircon_runtime/src/animation/sequence/tests/channel_sample.rs"))
        self.assertIn("iterations={} samples={} sample_count={}", read_rust_test_file("zircon_runtime/src/animation/sequence/tests/channel_sample.rs"))
        self.assertIn("raw_samples={}", read_rust_test_file("zircon_runtime/src/animation/sequence/tests/channel_sample.rs"))
        self.assertIn("compiled_samples={}", read_rust_test_file("zircon_runtime/src/animation/sequence/tests/channel_sample.rs"))

    def test_compiled_path_and_plan_keep_the_hard_cut(self) -> None:
        self.assertIn("sample_channel_keys(self.interpolation(), self.keys(), time_seconds)", self.source)
        self.assertNotIn("self.keys().iter().any", self.source)
        self.assertIn("101 samples", self.plan)
        self.assertIn("nearest-rank", self.plan)

    def test_nearest_rank_model_selects_the_95th_observation_for_101_samples(self) -> None:
        index = ((101 * 95) + 99) // 100 - 1
        self.assertEqual(index, 95)

    def test_compiler_transfers_owned_ir_without_a_boundary_clone(self) -> None:
        self.assertIn(
            "pub fn into_artifact(self) -> Option<AnimationCompiledSequence>",
            self.compiler_model,
        )
        self.assertIn("compile_sequence_for_world(world, source)", self.plugin_pipeline)
        self.assertIn("source_compilation.into_artifact()", self.plugin_pipeline)
        self.assertIn("source: Arc::new(source)", self.world_compiler)
        self.assertNotIn("source: Arc::new(source.clone())", self.world_compiler)
        self.assertNotIn("artifact().cloned()", self.plugin_pipeline)
        self.assertIn(".into_artifact()", self.world_compiler_tests)
        self.assertNotIn(".artifact()\n        .cloned()", self.world_compiler_tests)

    def test_world_compiler_preallocates_the_known_success_track_bound(self) -> None:
        self.assertIn("pub fn track_count(&self) -> usize", self.compiler_model)
        self.assertIn("track_count: usize", self.compiler_model)
        self.assertIn("let track_count = bindings.iter().fold(0usize", self.compiler_model)
        self.assertIn("count.saturating_add(binding.tracks.len())", self.compiler_model)
        self.assertIn("let source_track_capacity = source.track_count();", self.world_compiler)
        self.assertNotIn("source.bindings().iter().fold(", self.world_compiler)
        self.assertIn(
            "let mut tracks = Vec::with_capacity(source_track_capacity);",
            self.world_compiler,
        )
        self.assertIn("let mut missing_tracks = Vec::new();", self.world_compiler)
        self.assertIn("validated source-track bound", self.plan)


if __name__ == "__main__":
    unittest.main()
