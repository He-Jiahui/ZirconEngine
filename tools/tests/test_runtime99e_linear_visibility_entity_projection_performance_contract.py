from pathlib import Path
import unittest

import sys as _history_sys
from pathlib import Path as _HistoryPath
_HISTORY_REPO_ROOT = _HistoryPath(__file__).resolve().parents[2]
_history_sys.path.insert(0, str(_HISTORY_REPO_ROOT / '.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts'))
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import HistoricalArchiveReader as _ContractHistoryReader
_HISTORY_PATHS = {'docs/plans/optimize/zircon_runtime/99e/2026-08-27-linear-visibility-entity-projection.md': 'docs/plans/optimize/zircon_runtime/99e/2026-08-27-linear-visibility-entity-projection.md'}
_HISTORY_IDENTITIES = {'docs/plans/optimize/zircon_runtime/99e/2026-08-27-linear-visibility-entity-projection.md': (2933, '58e52d4a09a93422ab0b43616321d5583de4df68901c804abcfe8a73de471e12')}
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

# 可见实体投影单次访问渲染项，再对最终集合排序去重；本组阻止恢复有序树中间物化，并关联行为/性能证据。

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/scene/world/render_visibility.rs"
RECORD = (
    ROOT
    / "docs/plans/optimize/zircon_runtime/99e/2026-08-27-linear-visibility-entity-projection.md"
)


def production_source() -> str:
    return _read_contract_input(SOURCE, encoding="utf-8").split("#[cfg(test)]", 1)[0]


def projection_body() -> str:
    source = production_source()
    start = source.index("fn project_visibility_entity_sets(")
    end = source.index("fn sort_and_dedup_entities(", start)
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



class Runtime99ELinearVisibilityEntityProjectionContract(unittest.TestCase):
    def test_projection_removes_ordered_tree_materialization(self) -> None:
        source = production_source()
        self.assertNotIn("BTreeSet", source)
        self.assertNotIn("collect::<BTreeSet", source)

    def test_projection_visits_renderables_once(self) -> None:
        body = projection_body()
        self.assertEqual(body.count("for entry in renderables"), 1)
        self.assertEqual(body.count("Vec::with_capacity(renderables.len())"), 3)

    def test_projection_sorts_and_deduplicates_all_entity_sets(self) -> None:
        body = projection_body()
        self.assertEqual(body.count("sort_and_dedup_entities("), 3)
        source = production_source()
        self.assertIn("entities.sort_unstable();", source)
        self.assertIn("entities.dedup();", source)

    def test_behavior_and_performance_evidence_are_recorded(self) -> None:
        source = _mounted_test_source(SOURCE, "tests/render_visibility.rs", "tests")
        record = _read_contract_input(RECORD, encoding="utf-8")
        self.assertIn(
            "linear_entity_projection_preserves_sorted_unique_mobility_sets", source
        )
        self.assertIn("98.60%", record)
        self.assertIn("56.99%", record)
        self.assertIn("64.63%", record)


if __name__ == "__main__":
    unittest.main()
