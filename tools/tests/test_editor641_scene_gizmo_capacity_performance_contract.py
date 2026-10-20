import unittest
from pathlib import Path

import sys as _history_sys
from pathlib import Path as _HistoryPath
_HISTORY_REPO_ROOT = _HistoryPath(__file__).resolve().parents[2]
_history_sys.path.insert(0, str(_HISTORY_REPO_ROOT / '.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts'))
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import HistoricalArchiveReader as _ContractHistoryReader
_HISTORY_PATHS = {'docs/plans/optimize/zircon_editor/641/2026-09-01-preallocated-scene-gizmos.md': 'docs/plans/optimize/zircon_editor/641/2026-09-01-preallocated-scene-gizmos.md'}
_HISTORY_IDENTITIES = {'docs/plans/optimize/zircon_editor/641/2026-09-01-preallocated-scene-gizmos.md': (1894, '70b1f867cb6fd7ae3ac61b7c44a00a91c69e3c7971ad26ffdc44d0a0ed8a477e')}
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
SOURCE = (
    REPO_ROOT / "zircon_editor/src/scene/viewport/render_packet/tests/reused_overlay_storage.rs"
)
PLAN = (
    REPO_ROOT
    / "docs/plans/optimize/zircon_editor/641/2026-09-01-preallocated-scene-gizmos.md"
)



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



class Editor641SceneGizmoCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = _mounted_test_source(REPO_ROOT / "zircon_editor/src/scene/viewport/render_packet.rs", "render_packet/tests/reused_overlay_storage.rs", "reused_overlay_storage_tests")
        cls.plan = _read_contract_input(PLAN, encoding="utf-8")

    def test_release_probe_uses_101_pairs_and_reports_raw_series(self) -> None:
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", self.source)
        self.assertIn("sample_pairs={SAMPLE_PAIRS}", self.source)
        self.assertIn("legacy_samples={}", self.source)
        self.assertIn("optimized_samples={}", self.source)

    def test_scene_gizmo_capacity_bound_remains_direct_and_bounded(self) -> None:
        self.assertIn("let gizmo_capacity = scene", self.source)
        self.assertIn("Vec::with_capacity(gizmo_capacity)", self.source)
        self.assertIn("101 alternating sample pairs", self.plan)


if __name__ == "__main__":
    unittest.main()
