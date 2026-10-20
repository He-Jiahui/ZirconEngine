import unittest
from pathlib import Path

import sys as _history_sys
from pathlib import Path as _HistoryPath
_HISTORY_REPO_ROOT = _HistoryPath(__file__).resolve().parents[2]
_history_sys.path.insert(0, str(_HISTORY_REPO_ROOT / '.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts'))
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import HistoricalArchiveReader as _ContractHistoryReader
_HISTORY_PATHS = {'docs/plans/zircon_runtime/runtime/06/2026-07-09-plugin-surface-and-lifecycle-output-records.md': 'docs/plans/zircon_runtime/runtime/06/2026-07-09-plugin-surface-and-lifecycle-output-records.md'}
_HISTORY_IDENTITIES = {'docs/plans/zircon_runtime/runtime/06/2026-07-09-plugin-surface-and-lifecycle-output-records.md': (42919, '73d6e8c4cab58e0f3e0ef27dfbfd2dd11e2f39099963ea09991a1c5a5046228b')}
_CURRENT_POLICY_INPUTS = ('docs/architecture/runtime-architecture-review-m0.md', 'docs/testing-and-validation/development-conventions.md', 'docs/crates/zircon_runtime/structure/module-convention.md')

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


# 验证原生注册重放错误的职责切分：注册重放错误应为子节点持有，并类型化错误复核守卫读取错误子节点。
class RuntimeNativeRegistrationReplayErrorOwnerStructureTests(unittest.TestCase):
    def test_registration_replay_error_is_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = (
            repo_root
            / "zircon_runtime/src/plugin/loader/live/registration_replay.rs"
        )
        error_path = (
            repo_root
            / "zircon_runtime/src/plugin/loader/live/registration_replay/error.rs"
        )

        owner = _read_contract_input(owner_path, encoding="utf-8")
        self.assertLessEqual(len(owner.splitlines()), 800)

        error = _read_contract_input(error_path, encoding="utf-8")
        self.assertLessEqual(len(error.splitlines()), 800)
        self.assertIn('#[path = "registration_replay/system_preparation.rs"]', owner)
        self.assertIn("mod system_preparation;", owner)
        self.assertIn("mod error;", owner)
        self.assertIn(
            "pub(super) use error::NativePluginRegistrationReplayError;", owner
        )
        self.assertNotIn("enum NativePluginRegistrationReplayError", owner)
        for anchor in (
            "pub(in super::super) enum NativePluginRegistrationReplayError",
            "impl std::fmt::Display for NativePluginRegistrationReplayError",
            "impl std::error::Error for NativePluginRegistrationReplayError",
            "InvalidRegistrationManifest",
            "InvalidSystemAccessAuthority",
            "RegisterNativeSystem",
        ):
            self.assertIn(anchor, error)

        self.assertIn("type NativePluginRegistrationReplayResult<T>", owner)
        self.assertEqual(owner.count(".sort_unstable();"), 3)
        self.assertIn(
            '#[path = "registration_replay/tests/perf.rs"]', owner
        )

    def test_typed_error_review_guard_reads_error_child(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        guard = _read_contract_input(repo_root
            / "zircon_runtime/src/tests/errors/loader/replay_and_runtime/registration_replay.rs", encoding="utf-8")

        self.assertIn("registration_replay/error.rs", guard)
        self.assertIn("registration_replay_error", guard)

    def test_plan_and_boundary_docs_record_error_owner(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        child_path = "registration_replay/error.rs"
        status = (
            "runtime_06_15_native_registration_replay_error_owner_split_"
            "static_passed_cargo_deferred"
        )
        docs = (
            'docs/testing-and-validation/development-conventions.md',
            'docs/architecture/runtime-architecture-review-m0.md',
            'docs/crates/zircon_runtime/structure/module-convention.md',
            "docs/plans/zircon_runtime/runtime/06/2026-07-09-plugin-surface-and-lifecycle-output-records.md",
            "docs/architecture/native-plugin-boundary.md",
        )

        for relative_path in docs:
            source = _read_contract_input(repo_root / relative_path, encoding="utf-8")
            self.assertIn(child_path, source, relative_path)
            self.assertIn(status, source, relative_path)


if __name__ == "__main__":
    unittest.main()
