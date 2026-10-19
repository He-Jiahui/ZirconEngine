from pathlib import Path
import unittest

import sys as _history_sys
from pathlib import Path as _HistoryPath
_HISTORY_REPO_ROOT = _HistoryPath(__file__).resolve().parents[2]
_history_sys.path.insert(0, str(_HISTORY_REPO_ROOT / '.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts'))
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import HistoricalArchiveReader as _ContractHistoryReader
_HISTORY_PATHS = {'docs/plans/optimize/zircon_runtime/09b/2026-08-27-dirty-proportional-static-index-update.md': 'docs/plans/optimize/zircon_runtime/09b/2026-08-27-dirty-proportional-static-index-update.md'}
_HISTORY_IDENTITIES = {'docs/plans/optimize/zircon_runtime/09b/2026-08-27-dirty-proportional-static-index-update.md': (2658, 'f753e04ff4f6d6fda20b8668d928fdf633b41cd75d0d9e6ac3bf0e041412c156')}
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

# 静态可见性索引更新只按脏键规模建辅助索引，并保留重复实例最后一项的投影语义；计划与源码断言分担性能和行为证据。

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/graphics/visibility/static_index/mod.rs"
RECORD = (
    ROOT
    / "docs/plans/optimize/zircon_runtime/09b/2026-08-27-dirty-proportional-static-index-update.md"
)


def production_source() -> str:
    return _read_contract_input(SOURCE, encoding="utf-8").split("#[cfg(test)]", 1)[0]


def incremental_update_body() -> str:
    source = production_source()
    start = source.index("    pub(crate) fn apply_update_plan(")
    end = source.index("    pub(crate) fn query_bounds(", start)
    return source[start:end]



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



class Runtime09BDirtyProportionalStaticIndexContract(unittest.TestCase):
    def test_changed_key_index_capacity_scales_with_delta(self) -> None:
        body = incremental_update_body()
        self.assertIn("let changed_key_count = plan", body)
        self.assertIn("with_capacity(changed_key_count)", body)
        self.assertIn("HashMap::<u64, Option<&VisibilityBvhInstance>>", body)

    def test_incremental_update_does_not_collect_the_full_scene_index(self) -> None:
        body = incremental_update_body()
        self.assertNotIn("collect::<BTreeMap", body)
        self.assertNotIn("instances_by_stable_instance_key", body)
        self.assertEqual(body.count("for instance in instances"), 1)

    def test_projection_preserves_last_duplicate_instance_semantics(self) -> None:
        source = _read_contract_input(SOURCE, encoding="utf-8")
        tests = _mounted_test_source(SOURCE, "tests/cases.rs", "tests")
        self.assertIn(
            "visibility_static_index_incremental_projection_keeps_last_duplicate_instance",
            tests,
        )
        self.assertIn("*changed_instance = Some(instance);", source)

    def test_performance_model_enforces_memory_and_p95_targets(self) -> None:
        source = _mounted_test_source(SOURCE, "tests/cases.rs", "tests")
        record = _read_contract_input(RECORD, encoding="utf-8")
        self.assertIn("RUNTIME09B_DIRTY_PROPORTIONAL_STATIC_INDEX_BENCH_V1", source)
        self.assertIn("optimized_p95 * 100 <= legacy_p95 * 60", source)
        self.assertIn("99.87%", record)
        self.assertIn("76.2%", record)


if __name__ == "__main__":
    unittest.main()
