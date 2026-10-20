from tools.tests.rust_test_files import read_rust_test_file
import unittest
from pathlib import Path

import sys as _history_sys
from pathlib import Path as _HistoryPath
_HISTORY_REPO_ROOT = _HistoryPath(__file__).resolve().parents[2]
_history_sys.path.insert(0, str(_HISTORY_REPO_ROOT / '.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts'))
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import HistoricalArchiveReader as _ContractHistoryReader
_HISTORY_PATHS = {'docs/plans/optimize/zircon_runtime/170/2026-09-01-compiled-sequence-channel-sampling-hard-cut.md': 'docs/plans/optimize/zircon_runtime/170/2026-09-01-compiled-sequence-channel-sampling-hard-cut.md'}
_HISTORY_IDENTITIES = {'docs/plans/optimize/zircon_runtime/170/2026-09-01-compiled-sequence-channel-sampling-hard-cut.md': (8257, '1e5e8a853d549e30167b9cfa07a4306cc8280db88b57f21867cbc43a47001e56')}
_CURRENT_POLICY_INPUTS = ()

def _read_contract_input(path, *args, **kwargs):
    candidate = _HistoryPath(path)
    try:
        relative = candidate.absolute().relative_to(_HISTORY_REPO_ROOT).as_posix()
    except ValueError:
        relative = None
    key = _HISTORY_PATHS.get(relative)
    if key is None:
        if relative and relative.startswith('docs/plans/') and relative not in _CURRENT_POLICY_INPUTS:
            raise AssertionError('Unregistered current/history contract input: ' + relative)
        return candidate.read_text(*args, **kwargs)
    if args or kwargs.get('encoding', 'utf-8').lower() not in ('utf-8', 'utf8') or kwargs.get('errors') not in (None, 'strict') or set(kwargs) - {'encoding', 'errors'}:
        raise AssertionError('Historical evidence requires strict UTF-8')
    reader = _ContractHistoryReader(identities=_HISTORY_IDENTITIES)
    text = reader.read(key)
    if text is None:
        raise AssertionError(reader.records)
    return text



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
        cls.source = _read_contract_input(SOURCE, encoding="utf-8")
        cls.world_compiler = _read_contract_input(WORLD_COMPILER, encoding="utf-8")
        cls.compiler_model = _read_contract_input(COMPILER_MODEL, encoding="utf-8")
        cls.plugin_pipeline = _read_contract_input(PLUGIN_PIPELINE, encoding="utf-8")
        cls.world_compiler_tests = _read_contract_input(WORLD_COMPILER_TESTS, encoding="utf-8")
        cls.plan = _read_contract_input(PLAN, encoding="utf-8")

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
