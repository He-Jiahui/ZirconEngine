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



class RuntimeUiPointerComponentStateOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_09_15_ui_pointer_component_state_owner_split_"
        "static_passed_cargo_profile_deferred"
    )

    def test_pointer_component_state_is_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = (
            repo_root
            / "zircon_runtime/src/ui/surface/surface/pointer_component_events.rs"
        )
        child_path = (
            repo_root
            / "zircon_runtime/src/ui/surface/pointer_component_events/state_invalidation.rs"
        )

        owner = _read_contract_input(owner_path, encoding="utf-8")
        self.assertLessEqual(len(owner.splitlines()), 800)
        self.assertIn("mod state_invalidation;", owner)
        self.assertIn("mod template_action;", owner)
        for moved in (
            "fn apply_pointer_component_state",
            "fn apply_pointer_transient_state_dirty",
            "fn set_node_pressed_dirty",
            "fn mark_component_state_render_dirty",
            "fn mark_component_states_render_dirty",
            "fn minimal_changed_subtree_roots",
            "fn node_is_covered_by_roots",
        ):
            self.assertNotIn(moved, owner)

        child = _read_contract_input(child_path, encoding="utf-8")
        self.assertLessEqual(len(child.splitlines()), 800)
        for anchor in (
            "pub(in crate::ui::surface::surface) fn apply_pointer_component_state",
            "pub(in crate::ui::surface::surface) fn apply_pointer_transient_state_dirty",
            "fn set_node_pressed_dirty",
            "pub(crate) fn mark_component_state_render_dirty",
            "pub(crate) fn mark_component_states_render_dirty",
            "fn minimal_changed_subtree_roots",
            "fn node_is_covered_by_roots",
            "set_hovered",
            "set_pressed",
            "set_focused",
            "node_state_can_affect_descendants",
            "apply_runtime_state_style_subtree",
            "apply_runtime_state_style_node",
            "mark_node_dirty",
        ):
            self.assertIn(anchor, child)

        for event_owner_anchor in (
            "pub(super) fn pointer_component_events",
            "pub(super) fn push_state_damage_frames",
            "pub(super) fn push_pointer_component_events_with_drag_metrics",
            "compiled_binding_event_sources",
        ):
            self.assertIn(event_owner_anchor, owner)

        action_child = _read_contract_input(owner_path.parent / "pointer_component_events/action.rs", encoding="utf-8")
        for action_owner_anchor in (
            "template_action_for_compiled_binding_with_overrides",
            "dense_compiled_payload_overrides_for_benchmark",
            "compiled_binding_handle_for_source",
        ):
            self.assertIn(action_owner_anchor, action_child)

    def test_pointer_component_state_owner_status_is_mirrored(self) -> None:
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
            "zircon_runtime/src/ui/surface/surface/pointer_component_events.rs",
            "zircon_runtime/src/ui/surface/pointer_component_events/state_invalidation.rs",
            "tools/tests/test_runtime_ui_pointer_component_state_owner_structure.py",
        ):
            self.assertIn(current_path, runtime_plan)

    def test_single_pointer_state_change_stays_allocation_free_until_batching(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        source = _read_contract_input(repo_root
            / "zircon_runtime/src/ui/surface/pointer_component_events/state_invalidation.rs", encoding="utf-8")
        implementation = source.split("#[cfg(test)]", 1)[0]
        single_path = implementation.split(
            "pub(in crate::ui::surface::surface) fn apply_pointer_component_state(",
            1,
        )[1].split(
            "pub(in crate::ui::surface::surface) fn apply_pointer_transient_state_dirty(",
            1,
        )[0]

        self.assertIn("ChangedNodeAccumulator", implementation)
        self.assertIn("let mut changed_nodes = ChangedNodeAccumulator::default();", single_path)
        self.assertIn("changed_nodes.insert", single_path)
        self.assertIn("changed_nodes.finish(self)", single_path)
        self.assertNotIn("let mut changed_node_ids = BTreeSet::new()", single_path)

        batch_path = implementation.split(
            "pub(crate) fn mark_component_states_render_dirty(",
            1,
        )[1]
        self.assertIn("if changed_node_ids.len() == 1", batch_path)
        self.assertIn("mark_component_state_render_dirty(node_id)", batch_path)
        roots_path = implementation.split("fn minimal_changed_subtree_roots(", 1)[1]
        self.assertIn("Vec::with_capacity(changed_node_ids.len())", roots_path)


if __name__ == "__main__":
    unittest.main()
