from pathlib import Path
import unittest

import sys as _history_sys
from pathlib import Path as _HistoryPath
_HISTORY_REPO_ROOT = _HistoryPath(__file__).resolve().parents[2]
_history_sys.path.insert(0, str(_HISTORY_REPO_ROOT / '.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts'))
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import HistoricalArchiveReader as _ContractHistoryReader
_HISTORY_PATHS = {'docs/plans/optimize/zircon_runtime/99i/2026-08-27-contiguous-transition-validation.md': 'docs/plans/optimize/zircon_runtime/99i/2026-08-27-contiguous-transition-validation.md'}
_HISTORY_IDENTITIES = {'docs/plans/optimize/zircon_runtime/99i/2026-08-27-contiguous-transition-validation.md': (2978, '87c3866de81fdfbf124ea43bfdb9d9cad87749847dce013e933f3328c4400e3c')}
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

# ECS archetype 转换用一段连续组件 ID 缓冲合并排序差量，预留仅覆盖真实新增项；线性 schema 比较保留错误优先级。

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/scene/ecs/archetype/table/table.rs"
RECORD = (
    ROOT
    / "docs/plans/optimize/zircon_runtime/99i/2026-08-27-contiguous-transition-validation.md"
)


def production_source() -> str:
    return _read_contract_input(SOURCE, encoding="utf-8").split("#[cfg(test)]", 1)[0]


def transition_body() -> str:
    source = production_source()
    start = source.index("    pub(crate) fn validate_transition(")
    end = source.index("    pub(crate) fn bind_prevalidated_row(", start)
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



class Runtime99IContiguousTransitionValidationContract(unittest.TestCase):
    def test_transition_uses_one_contiguous_component_id_buffer(self) -> None:
        body = transition_body()
        self.assertNotIn("BTreeSet", body)
        self.assertIn("collect::<Vec<_>>()", body)
        self.assertIn("final_component_ids.sort_unstable();", body)
        self.assertIn("final_component_ids.dedup();", body)

    def test_transition_reserves_only_real_insertions_and_applies_sorted_delta(self) -> None:
        body = transition_body()
        self.assertIn("inserted_component_count", body)
        self.assertIn("reserve(inserted_component_count)", body)
        self.assertIn("apply_component_membership_updates(", body)
        self.assertIn("binary_search(&component_id)", production_source())

    def test_linear_schema_comparison_preserves_error_precedence(self) -> None:
        body = transition_body()
        unexpected = body.index("first_unexpected_component(")
        missing = body.index("first_missing_component(")
        self.assertLess(unexpected, missing)
        source = _mounted_test_source(SOURCE, "tests/table.rs", "tests")
        self.assertIn(
            "runtime99i_contiguous_transition_validation_preserves_error_precedence",
            source,
        )

    def test_performance_evidence_enforces_allocation_and_p95_targets(self) -> None:
        source = _mounted_test_source(SOURCE, "tests/table.rs", "tests")
        record = _read_contract_input(RECORD, encoding="utf-8")
        self.assertIn("RUNTIME99I_CONTIGUOUS_TRANSITION_VALIDATION_BENCH_V1", source)
        self.assertIn("optimized_p95 * 100 <= legacy_p95 * 60", source)
        self.assertIn("99.74%", record)
        self.assertIn("66.90%", record)


if __name__ == "__main__":
    unittest.main()
