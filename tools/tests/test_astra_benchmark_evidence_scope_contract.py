"""Keep Astra benchmark evidence scoped, bounded, and statistically explicit."""

from pathlib import Path
import unittest

import sys as _history_sys
from pathlib import Path as _HistoryPath
_HISTORY_REPO_ROOT = _HistoryPath(__file__).resolve().parents[2]
_history_sys.path.insert(0, str(_HISTORY_REPO_ROOT / '.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts'))
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import HistoricalArchiveReader as _ContractHistoryReader
_HISTORY_PATHS = {'docs/plans/optimize/zircon_editor/612/2026-09-01-reset-delta-direct-materialization.md': 'docs/plans/optimize/zircon_editor/612/2026-09-01-reset-delta-direct-materialization.md', 'docs/plans/optimize/zircon_editor/633/2026-09-01-bitset-export-stage-planning.md': 'docs/plans/optimize/zircon_editor/633/2026-09-01-bitset-export-stage-planning.md', 'docs/plans/optimize/zircon_editor/633/2026-09-01-preallocated-widget-dependency-closure.md': 'docs/plans/optimize/zircon_editor/633/2026-09-01-preallocated-widget-dependency-closure.md', 'docs/plans/optimize/zircon_runtime/629/2026-09-01-preallocated-gltf-mesh-dependency-index.md': 'docs/plans/optimize/zircon_runtime/629/2026-09-01-preallocated-gltf-mesh-dependency-index.md', 'docs/plans/optimize/zircon_runtime/629/2026-09-01-preallocated-manifest-membership.md': 'docs/plans/optimize/zircon_runtime/629/2026-09-01-preallocated-manifest-membership.md', 'docs/plans/_archive/astra/performance/01-bounded-hotpaths.md': 'docs/plans/_archive/astra/performance/01-bounded-hotpaths.md'}
_HISTORY_IDENTITIES = {'docs/plans/optimize/zircon_editor/612/2026-09-01-reset-delta-direct-materialization.md': (2534, '5bbaff595153e1e919b8d777fe1b778acba3770cc04336c387facdcde41e8e56'), 'docs/plans/optimize/zircon_editor/633/2026-09-01-bitset-export-stage-planning.md': (1724, '429d0c12e1d0ff92730b7d9c3946479c1cd3bdd558173dfa7dd31dd39630ccca'), 'docs/plans/optimize/zircon_editor/633/2026-09-01-preallocated-widget-dependency-closure.md': (2048, '7695d1c7f3b83610e076984022db33e556ab79df78536e8d63bf1c23ec4451a5'), 'docs/plans/optimize/zircon_runtime/629/2026-09-01-preallocated-gltf-mesh-dependency-index.md': (2326, 'a396c0f0da2d87e85a2235a506e9db1b39eb6aa20013ea99902671fb9210c963'), 'docs/plans/optimize/zircon_runtime/629/2026-09-01-preallocated-manifest-membership.md': (1846, '8c7fc91487e14f906a480deba15de07df3f5af3c165708054a570e841c32e5ed'), 'docs/plans/_archive/astra/performance/01-bounded-hotpaths.md': (17406, 'db11719946998a3abd38c6aef7aae5107e6d16a60934ffc18ddaf077da13a1dc')}
_CURRENT_POLICY_INPUTS = ('docs/testing-and-validation/milestone-validation-policy.md',)

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



ROOT = Path(__file__).resolve().parents[2]

MANIFEST_BENCH = ROOT / "zircon_runtime/src/asset/project/manifest/validation/tests/perf.rs"
GLTF_BENCH = ROOT / "zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets/tests/perf.rs"
DIRTY_BENCH = ROOT / "zircon_editor/src/core/asset/dirty/registry/tests/optimization_tests.rs"
WIDGET_BENCH = ROOT / "zircon_editor/src/ui/asset_editor/promote_widget/tests/perf.rs"
EXPORT_BENCH = ROOT / "zircon_editor/src/core/export/pipeline/tests/perf.rs"

DOCS = [
    ROOT / "docs/plans/optimize/zircon_runtime/629/2026-09-01-preallocated-manifest-membership.md",
    ROOT / "docs/plans/optimize/zircon_runtime/629/2026-09-01-preallocated-gltf-mesh-dependency-index.md",
    ROOT / "docs/plans/optimize/zircon_editor/612/2026-09-01-reset-delta-direct-materialization.md",
    ROOT / "docs/plans/optimize/zircon_editor/633/2026-09-01-preallocated-widget-dependency-closure.md",
    ROOT / "docs/plans/optimize/zircon_editor/633/2026-09-01-bitset-export-stage-planning.md",
]


BENCHMARK_MOUNTS = {
    MANIFEST_BENCH: (ROOT / 'zircon_runtime/src/asset/project/manifest/validation.rs', 'validation/tests/perf.rs', 'optimization_batch_ir_runtime629_tests'),
    GLTF_BENCH: (ROOT / 'zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs', 'gltf_labeled_subassets/tests/perf.rs', 'optimization_batch_is_runtime629_tests'),
    DIRTY_BENCH: (ROOT / 'zircon_editor/src/core/asset/dirty/registry.rs', 'registry/tests/optimization_tests.rs', 'optimization_tests'),
    WIDGET_BENCH: (ROOT / 'zircon_editor/src/ui/asset_editor/promote_widget.rs', 'promote_widget/tests/perf.rs', 'optimization_batch_iw_editor633_tests'),
    EXPORT_BENCH: (ROOT / 'zircon_editor/src/core/export/pipeline.rs', 'pipeline/tests/perf.rs', 'optimization_batch_iv_editor633_tests'),
}


def read(path: Path) -> str:
    if path in BENCHMARK_MOUNTS:
        return _mounted_test_source(*BENCHMARK_MOUNTS[path])
    return _read_contract_input(path, encoding="utf-8")



def _mounted_test_source(parent: Path, relative: str, module: str, cfg: str = "test") -> str:
    """Read the actual explicit Rust test mount; missing or changed mounting fails closed."""
    source = _read_contract_input(parent, encoding="utf-8")
    declaration = f'#[cfg({cfg})]\n#[path = "{relative}"]\nmod {module};'
    if declaration not in source:
        raise AssertionError(f"Rust test mount changed: {parent}::{module}")
    child = (parent.parent / relative).resolve()
    if not child.is_relative_to(_HISTORY_REPO_ROOT):
        raise AssertionError(f"Rust test mount escapes the checkout: {child}")
    return _read_contract_input(child, encoding="utf-8")



class AstraBenchmarkEvidenceScopeContractTests(unittest.TestCase):
    def assert_explicit_percentiles(self, source: str) -> None:
        self.assertIn("percentile_method=nearest_rank", source)
        self.assertRegex(source, r"p50(?:_ns)?")
        self.assertRegex(source, r"p95(?:_ns)?")
        self.assertRegex(source, r"p99(?:_ns)?")
        self.assertIn("benchmark_scope=microbenchmark", source)
        self.assertIn("real_caller_required=true", source)

    def benchmark_body(self, source: str, function_name: str) -> str:
        start = source.index(f"fn {function_name}")
        next_function = source.find("\nfn ", start + 4)
        return source[start:] if next_function < 0 else source[start:next_function]

    def test_manifest_benchmark_uses_the_product_root_bound(self):
        source = read(MANIFEST_BENCH)
        self.assertIn("use zircon_runtime_interface::project::MAX_PROJECT_ASSET_ROOTS;", source)
        self.assertIn("const ROOT_COUNT: usize = MAX_PROJECT_ASSET_ROOTS;", source)
        self.assertNotIn("32_768", source)
        self.assertIn("const SAMPLE_COUNT: usize = 31;", source)
        self.assert_explicit_percentiles(source)

    def test_other_benchmarks_report_p50_p95_p99_and_scope(self):
        cases = (
            (GLTF_BENCH, "optimization_batch_is_runtime629_preallocated_mesh_dependency_benchmark", "const SAMPLE_PAIRS: usize = 31;"),
            (DIRTY_BENCH, "optimization_batch_ic_editor612_reset_delta_seed_p95", "const SAMPLE_PAIRS: usize = 31;"),
            (WIDGET_BENCH, "optimization_batch_iw_editor633_preallocated_widget_dependency_benchmark", "const SAMPLE_PAIRS: usize = 31;"),
            (EXPORT_BENCH, "optimization_batch_r6_wave_editor633_export_stage_validation_bitset_p95", "const SAMPLE_PAIRS: usize = 31;"),
        )
        for path, function_name, sample_declaration in cases:
            with self.subTest(path=path):
                source = read(path)
                self.assertIn(sample_declaration, source)
                self.assert_explicit_percentiles(self.benchmark_body(source, function_name))

    def test_benchmark_docs_do_not_present_helpers_as_product_acceptance(self):
        for path in DOCS:
            with self.subTest(path=path):
                text = read(path).lower()
                self.assertIn("microbenchmark", text)
                self.assertRegex(text, r"real caller|真实 caller|真实调用")
                self.assertRegex(text, r"pending|待测|待验证")
                self.assertRegex(text, r"p50.*p95.*p99|p50/p95/p99")

    def test_astra_performance_plan_records_the_correction(self):
        text = read(ROOT / 'docs/plans/_archive/astra/performance/01-bounded-hotpaths.md').lower()
        self.assertIn("runtime629", text)
        self.assertIn("max_project_asset_roots", text)
        self.assertIn("microbenchmark", text)
        self.assertIn("p50/p95/p99", text)
        self.assertRegex(text, r"真实 caller|real caller")
        # The correction is fixed dated evidence; current policy is checked independently.
        policy = read(ROOT / 'docs/testing-and-validation/milestone-validation-policy.md').lower()
        self.assertIn("max_project_asset_roots", policy)
        self.assertIn("microbenchmark", policy)
        self.assertIn("p50/p95/p99", policy)
        self.assertRegex(policy, r"真实 caller|real caller")


if __name__ == "__main__":
    unittest.main()
