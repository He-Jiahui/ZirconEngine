import unittest
from pathlib import Path

import sys as _history_sys
from pathlib import Path as _HistoryPath
_HISTORY_REPO_ROOT = _HistoryPath(__file__).resolve().parents[2]
_history_sys.path.insert(0, str(_HISTORY_REPO_ROOT / '.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts'))
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import HistoricalArchiveReader as _ContractHistoryReader
_HISTORY_PATHS = {'docs/plans/zircon_runtime/text/09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md': 'versions/reconciliation-515/d9589b2ad2f6a3bd60b8176fe14baeb6ecf71abd960f9c7890148eaeb846a014/docs/plans/zircon_runtime/text/09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md'}
_HISTORY_IDENTITIES = {'versions/reconciliation-515/d9589b2ad2f6a3bd60b8176fe14baeb6ecf71abd960f9c7890148eaeb846a014/docs/plans/zircon_runtime/text/09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md': (20861, 'd9589b2ad2f6a3bd60b8176fe14baeb6ecf71abd960f9c7890148eaeb846a014')}
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



ROOT = Path(__file__).resolve().parents[2]
TEXT_SHAPE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/text_shape.rs"
)
PROFILE_BENCH = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/tests/projection_profile.rs"
)
PROFILE_PLAN = (
    ROOT
    / "docs/plans/zircon_runtime/text/09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md"
)

# 读取文本绘制投影剖析入口，确认私有辅助函数与固定密集样本通道一致。

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



class RuntimeTextPaintRunProjectionProfileContractTests(unittest.TestCase):
    def test_private_profile_owner_calls_the_exact_production_helper(self) -> None:
        root = _read_contract_input(TEXT_SHAPE, encoding="utf-8")
        profile = _mounted_test_source(TEXT_SHAPE, "tests/projection_profile.rs", "projection_profile", "all(test, windows)")

        self.assertIn('path = "tests/projection_profile.rs"', root)
        self.assertIn("mod projection_profile;", root)
        self.assertIn("text_paint_runs_from_resolved_layout(", profile)
        self.assertNotIn("resolved_text_run_frame(", profile)

    def test_release_harness_uses_fixed_dense_run_lanes_and_raw_samples(self) -> None:
        profile = _mounted_test_source(TEXT_SHAPE, "tests/projection_profile.rs", "projection_profile", "all(test, windows)")

        self.assertIn("const SAMPLE_COUNT: usize = 31;", profile)
        self.assertIn("const DENSE_RUN_COUNTS: [usize; 4] = [1, 100, 1_000, 10_000];", profile)
        self.assertIn("#[ignore =", profile)
        self.assertIn("samples_ns={samples_ns:?}", profile)
        self.assertIn("rss_delta_bytes={rss_delta_bytes:?}", profile)
        self.assertIn("p50_ns={p50_ns}", profile)
        self.assertIn("p95_ns={p95_ns}", profile)
        self.assertIn("p99_ns={p99_ns}", profile)

    def test_harness_reports_static_work_and_does_not_write_artifacts(self) -> None:
        profile = _mounted_test_source(TEXT_SHAPE, "tests/projection_profile.rs", "projection_profile", "all(test, windows)")

        self.assertIn("implied_full_line_grapheme_visits", profile)
        self.assertIn("current_rss_bytes", profile)
        self.assertNotIn("target/", profile)
        self.assertNotIn("File::create", profile)
        self.assertNotIn("fs::write", profile)

    def test_plan_keeps_the_baseline_unmeasured_until_the_harness_runs(self) -> None:
        plan = _read_contract_input(PROFILE_PLAN, encoding="utf-8")

        self.assertIn("baseline_profile_pending_no_optimization", plan)
        self.assertIn("The test has not run under Cargo", plan)
        self.assertIn("1/100/1k/10k runs", plan)


if __name__ == "__main__":
    unittest.main()
