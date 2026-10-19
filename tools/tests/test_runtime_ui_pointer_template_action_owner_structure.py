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


# 核对指针模板动作迁入子模块，且结构状态镜像记录相同的归属结论。
class RuntimeUiPointerTemplateActionOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_09_15_ui_pointer_template_action_owner_split_"
        "static_passed_cargo_profile_deferred"
    )

    def test_pointer_template_actions_are_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = (
            repo_root
            / "zircon_runtime/src/ui/surface/surface/pointer_component_events.rs"
        )
        child_path = (
            repo_root
            / "zircon_runtime/src/ui/surface/surface/pointer_component_events/action.rs"
        )

        owner = _read_contract_input(owner_path, encoding="utf-8")
        self.assertLessEqual(len(owner.splitlines()), 550)
        self.assertIn("mod template_action;", owner)
        for moved in (
            "fn template_action_for_binding",
            "fn template_action_for_binding_with_overrides",
            "fn template_action_for_compiled_binding_with_overrides",
            "fn dense_compiled_payload_overrides_for_benchmark",
            "fn template_action_for_compiled_binding_with_legacy_overrides_for_benchmark",
            "fn compiled_binding_handle_for_source",
            "fn template_action_payload_value",
            "fn template_action_control_property_value",
            "fn template_action_property_value",
        ):
            self.assertNotIn(moved, owner)

        child = _read_contract_input(child_path, encoding="utf-8")
        self.assertLessEqual(len(child.splitlines()), 400)
        for anchor in (
            "pub(crate) fn template_action_for_binding",
            "pub(crate) fn template_action_for_binding_with_overrides",
            "pub(crate) fn template_action_for_compiled_binding_with_overrides",
            "pub(crate) fn dense_compiled_payload_overrides_for_benchmark",
            "pub(crate) fn template_action_for_compiled_binding_with_legacy_overrides_for_benchmark",
            "pub(crate) fn compiled_binding_handle_for_source",
            "pub(crate) fn template_action_payload_value",
            "pub(crate) fn template_action_control_property_value",
            "pub(crate) fn template_action_property_value",
            "UiBindingMissingValueResolution",
            "resolve_compiled_action_payload_value",
            "template_action_property_value",
        ):
            self.assertIn(anchor, child)

        for event_owner_anchor in (
            "pub(super) fn pointer_component_events",
            "pub(super) fn push_focus_component_events",
            "pub(super) fn push_state_damage_frames",
            "pub(super) fn push_pointer_component_events_with_drag_metrics",
            "fn push_pointer_component_event_for_binding",
        ):
            self.assertIn(event_owner_anchor, owner)

    def test_pointer_template_action_owner_status_is_mirrored(self) -> None:
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
            "zircon_runtime/src/ui/surface/surface/pointer_component_events/action.rs",
            "tools/tests/test_runtime_ui_pointer_template_action_owner_structure.py",
        ):
            self.assertIn(current_path, runtime_plan)


if __name__ == "__main__":
    unittest.main()
