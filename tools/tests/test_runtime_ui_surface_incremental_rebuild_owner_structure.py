import unittest
from pathlib import Path

import sys as _history_sys
from pathlib import Path as _HistoryPath
_HISTORY_REPO_ROOT = _HistoryPath(__file__).resolve().parents[2]
_history_sys.path.insert(0, str(_HISTORY_REPO_ROOT / '.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts'))
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import HistoricalArchiveReader as _ContractHistoryReader
_HISTORY_PATHS = {'docs/plans/zircon_runtime/runtime/09/2026-08-07-runtime-ui-incremental-refresh.md': 'versions/reconciliation-515/9bdb78659a03053cc5ec25b3212e527045340d0d65ca343ff06e1a91c91e9bff/docs/plans/zircon_runtime/runtime/09/2026-08-07-runtime-ui-incremental-refresh.md'}
_HISTORY_IDENTITIES = {'versions/reconciliation-515/9bdb78659a03053cc5ec25b3212e527045340d0d65ca343ff06e1a91c91e9bff/docs/plans/zircon_runtime/runtime/09/2026-08-07-runtime-ui-incremental-refresh.md': (34603, '9bdb78659a03053cc5ec25b3212e527045340d0d65ca343ff06e1a91c91e9bff')}
_CURRENT_POLICY_INPUTS = ('docs/architecture/runtime-architecture-review-m0.md', 'docs/testing-and-validation/development-conventions.md', 'docs/crates/zircon_runtime/ui/architecture.md', 'docs/testing-and-validation/runtime-ui-validation-worklist.md', 'docs/crates/zircon_runtime/structure/module-convention.md')

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


# 核对表面增量重建的子模块归属，且结构状态镜像与代码目录保持一致。
class RuntimeUiSurfaceIncrementalRebuildOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_09_15_ui_surface_incremental_rebuild_owner_split_"
        "static_passed_cargo_profile_deferred"
    )

    def test_incremental_rebuild_is_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = repo_root / "zircon_runtime/src/ui/surface/surface/rebuild.rs"
        child_path = (
            repo_root / "zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs"
        )

        owner = _read_contract_input(owner_path, encoding="utf-8")
        self.assertLessEqual(len(owner.splitlines()), 800)
        self.assertIn("mod incremental;", owner)
        self.assertNotIn("pub fn rebuild_dirty", owner)
        self.assertNotIn("fn should_use_full_layout_rebuild", owner)
        self.assertNotIn("fn merge_incremental_layout_engine_report", owner)
        self.assertNotIn("fn patch_incremental_layout_engine_report", owner)

        child = _read_contract_input(child_path, encoding="utf-8")
        self.assertLessEqual(len(child.splitlines()), 800)
        for anchor in (
            "pub fn rebuild_dirty",
            "fn should_use_full_layout_rebuild",
            "fn merge_incremental_layout_engine_report",
            "fn patch_incremental_layout_engine_report",
            "UI_LAYOUT_INCREMENTAL_MAX_DIRTY_RATIO_DENOMINATOR",
            "UI_LAYOUT_INCREMENTAL_MAX_DIRTY_NODE_COUNT",
            "invalidate_for_changed_text_font_generation",
            "ui.layout.full_rebuild_threshold_count",
            "patch_arranged_tree_geometry",
            "patch_arranged_tree_input",
            "navigation_index_patch_changed_geometry",
            "mark_surface_frame_rebuild_dirty",
            "publish_surface_frame_after_rebuild",
            "record_surface_rebuild_profile",
        ):
            self.assertIn(anchor, child)

        for concurrent_anchor in (
            "arranged_visibility",
            "rebuild_render_extract_with_text_frame",
            "pub fn compute_layout",
            "invalidate_measure",
            "publish_surface_frame_after_rebuild",
        ):
            self.assertIn(concurrent_anchor, owner)

    def test_incremental_rebuild_owner_status_is_mirrored(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        mirrors = (
            repo_root
            / 'docs/testing-and-validation/runtime-ui-validation-worklist.md',
            repo_root
            / 'docs/crates/zircon_runtime/structure/module-convention.md',
            repo_root
            / 'docs/crates/zircon_runtime/ui/architecture.md',
            repo_root
            / "docs/plans/zircon_runtime/runtime/09/2026-08-07-runtime-ui-incremental-refresh.md",
            repo_root / "docs/crates/zircon_runtime/ui/architecture.md",
            repo_root / 'docs/testing-and-validation/development-conventions.md',
            repo_root / 'docs/architecture/runtime-architecture-review-m0.md',
        )
        for mirror_path in mirrors:
            mirror = _read_contract_input(mirror_path, encoding="utf-8")
            self.assertIn(self.STATUS, mirror, mirror_path.as_posix())

        runtime_plan = _read_contract_input(mirrors[1], encoding="utf-8")
        for current_path in (
            "zircon_runtime/src/ui/surface/surface/rebuild.rs",
            "zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs",
            "tools/tests/test_runtime_ui_surface_incremental_rebuild_owner_structure.py",
        ):
            self.assertIn(current_path, runtime_plan)


if __name__ == "__main__":
    unittest.main()
