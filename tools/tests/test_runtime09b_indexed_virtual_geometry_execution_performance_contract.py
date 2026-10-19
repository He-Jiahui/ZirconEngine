from pathlib import Path
import unittest

import sys as _history_sys
from pathlib import Path as _HistoryPath
_HISTORY_REPO_ROOT = _HistoryPath(__file__).resolve().parents[2]
_history_sys.path.insert(0, str(_HISTORY_REPO_ROOT / '.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts'))
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import HistoricalArchiveReader as _ContractHistoryReader
_HISTORY_PATHS = {'docs/plans/optimize/zircon_runtime/09b/2026-08-27-indexed-virtual-geometry-execution-projection.md': 'docs/plans/optimize/zircon_runtime/09b/2026-08-27-indexed-virtual-geometry-execution-projection.md'}
_HISTORY_IDENTITIES = {'docs/plans/optimize/zircon_runtime/09b/2026-08-27-indexed-virtual-geometry-execution-projection.md': (3205, 'affd2f51bcaf51ac692b30db143a9f6e20a2438cb0739836bc2d2f2b5d814c01')}
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

# 虚拟几何执行投影复用首匹配查找索引并稳定排序簇 ID；本组阻止生产路径退回嵌套线性查找，关联 Rust 与发布性能门禁。

ROOT = Path(__file__).resolve().parents[2]
EXECUTION = (
    ROOT
    / "zircon_runtime/src/graphics/runtime/framework/extract/submit/debug/execution.rs"
)
ROOT_SOURCE = (
    ROOT
    / "zircon_runtime/src/graphics/runtime/framework/extract/submit/debug.rs"
)
RECORD = (
    ROOT
    / "docs/plans/optimize/zircon_runtime/09b/2026-08-27-indexed-virtual-geometry-execution-projection.md"
)


def production_source(path: Path) -> str:
    return _read_contract_input(path, encoding="utf-8").split("#[cfg(test)]", 1)[0]



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



class Runtime09BIndexedVirtualGeometryExecutionContract(unittest.TestCase):
    def test_execution_projection_builds_one_reused_lookup(self) -> None:
        root = production_source(ROOT_SOURCE)
        start = root.index("pub(super) fn build_virtual_geometry_debug_snapshot(")
        end = root.index("\nfn vg_execution_required<", start)
        source = root[start:end]
        self.assertEqual(source.count("ExecutionLookup::new(extract)"), 1)
        self.assertEqual(source.count("&execution_lookup"), 2)
        self.assertIn("build_execution_snapshot(", source)
        self.assertIn("build_selected_clusters_from_execution_segments(", source)
        self.assertIn("let execution_lookup = ExecutionLookup::new(extract);", source)
        self.assertIn("build_execution_snapshot(\n        &execution_lookup,", source)
        self.assertIn("build_selected_clusters_from_execution_segments(&execution_lookup, &execution.segments)", source)

        # Production data is a distinct entry, not another lookup inside the debug entry.
        start = root.index("pub(super) fn build_virtual_geometry_execution_data(")
        end = root.index("\npub(super) fn build_virtual_geometry_cull_input(", start)
        execution_data = root[start:end]
        self.assertEqual(execution_data.count("ExecutionLookup::new(extract)"), 1)
        self.assertEqual(execution_data.count("&lookup"), 1)
        self.assertIn("let lookup = ExecutionLookup::new(extract);", execution_data)
        self.assertIn("build_execution_snapshot(\n        &lookup,", execution_data)
        self.assertNotIn("build_selected_clusters_from_execution_segments(", execution_data)
        self.assertIn("build_lossless_candidate_catalog(", execution_data)
        self.assertEqual(
            root.count("ExecutionLookup::new(extract)"),
            source.count("ExecutionLookup::new(extract)") + execution_data.count("ExecutionLookup::new(extract)"),
        )

    def test_lookup_uses_first_match_hash_index_and_sorted_cluster_ids(self) -> None:
        source = production_source(EXECUTION)
        self.assertIn("HashMap<u64, Vec<u32>>", source)
        self.assertIn("HashMap<ExecutionClusterKey, usize>", source)
        self.assertIn(".or_insert(cluster_index)", source)
        self.assertIn("cluster_ids.sort_unstable();", source)
        self.assertIn("cluster_ids.dedup();", source)

    def test_production_projection_does_not_restore_nested_linear_searches(self) -> None:
        source = production_source(EXECUTION)
        self.assertNotIn("fn instance_index_for_cluster_array_index(", source)
        self.assertNotIn("fn cluster_ordinal_for_stable_instance_key(", source)
        self.assertNotIn("cluster_ids.iter().position", source)
        self.assertNotIn(".clusters\n        .iter()\n        .enumerate()\n        .find(", source)

    def test_regression_and_release_performance_gates_are_present(self) -> None:
        source = _mounted_test_source(EXECUTION, "../../../../extract/submit/debug/tests/execution.rs", "tests")
        record = _read_contract_input(RECORD, encoding="utf-8")
        self.assertIn(
            "indexed_execution_lookup_preserves_first_match_and_sorted_unique_ordinal",
            source,
        )
        self.assertIn(
            "indexed_execution_lookup_preserves_legacy_stable_instance_key",
            source,
        )
        self.assertIn(
            "RUNTIME09B_INDEXED_VIRTUAL_GEOMETRY_EXECUTION_BENCH_V1",
            source,
        )
        self.assertIn("indexed_p95.saturating_mul(10_000)", source)
        self.assertIn("97.15%", record)
        self.assertIn("32.82%", record)
        self.assertIn("51.82%", record)


if __name__ == "__main__":
    unittest.main()
