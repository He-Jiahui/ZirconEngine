import unittest
from pathlib import Path

import sys as _history_sys
from pathlib import Path as _HistoryPath
_HISTORY_REPO_ROOT = _HistoryPath(__file__).resolve().parents[2]
_history_sys.path.insert(0, str(_HISTORY_REPO_ROOT / '.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts'))
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import HistoricalArchiveReader as _ContractHistoryReader
_HISTORY_PATHS = {'docs/plans/zircon_runtime/render/17/2026-08-11-render17-profiling-readiness-and-optimization-research.md': 'docs/plans/zircon_runtime/render/17/2026-08-11-render17-profiling-readiness-and-optimization-research.md'}
_HISTORY_IDENTITIES = {'docs/plans/zircon_runtime/render/17/2026-08-11-render17-profiling-readiness-and-optimization-research.md': (12232, '8d85f7e68219498638e34829bc6f22224e8c46a5c78d112bf92821459648d578')}
_CURRENT_POLICY_INPUTS = ('docs/architecture/runtime-architecture-review-m0.md', 'docs/testing-and-validation/development-conventions.md', 'docs/crates/zircon_runtime/core/diagnostics/profiling.md', 'docs/crates/zircon_runtime/structure/module-convention.md')

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


# 验证帧分析器 GPU 解析的职责切分：GPU 解析应为子节点持有，并GPU 解析归属状态应为镜像。
class RuntimeFrameProfilerGpuResolutionOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_17_15_frame_profiler_gpu_resolution_owner_split_"
        "static_passed_cargo_profile_deferred"
    )

    def test_gpu_resolution_is_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = (
            repo_root
            / "zircon_runtime/src/graphics/runtime/framework/frame_profiler.rs"
        )
        child_path = (
            repo_root
            / "zircon_runtime/src/graphics/runtime/framework/frame_profiler/gpu_resolution.rs"
        )

        owner = _read_contract_input(owner_path, encoding="utf-8")
        self.assertLessEqual(len(owner.splitlines()), 800)

        child = _read_contract_input(child_path, encoding="utf-8")
        self.assertLessEqual(len(child.splitlines()), 800)
        self.assertIn("mod gpu_resolution;", owner)
        self.assertIn("use gpu_resolution::FrameProfileWrite;", owner)
        self.assertNotIn("struct FrameProfileWrite", owner)
        self.assertNotIn("fn merge_gpu_timer_result", owner)
        self.assertNotIn("fn merge_gpu_pipeline_statistics_result", owner)
        self.assertIn('#[path = "tests/frame_profiler.rs"]', owner)
        tests = _read_contract_input(
            owner_path.parent / "tests/frame_profiler.rs", encoding="utf-8"
        )
        self.assertLessEqual(len(tests.splitlines()), 800)
        self.assertEqual(tests.count("#[test]"), 11)

        for anchor in (
            "struct FrameProfileWrite",
            "pub(super) fn merge_gpu_timer_result",
            "pub(super) fn merge_gpu_pipeline_statistics_result",
            "fn take_next_pass_profile_index",
            "Arc::make_mut",
            "RenderGpuTimingStatus::CapacityExhausted",
            "fn update_subsystem_gpu_times",
            "fn gpu_budget_warning_count",
            "fn saturating_u32_from_u64",
        ):
            self.assertIn(anchor, child)

        for concurrent_anchor in (
            "mod mesh_submission;",
            "memory_budget_warning_count",
            "GpuMemoryBudget",
            "mesh_submission_profile(stats)",
            "const MAX_PENDING_FRAME_PROFILES: usize = 4;",
        ):
            self.assertIn(concurrent_anchor, owner)

    def test_gpu_resolution_owner_status_is_mirrored(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        mirrors = (
            repo_root
            / 'docs/crates/zircon_runtime/structure/module-convention.md',
            repo_root
            / 'docs/crates/zircon_runtime/core/diagnostics/profiling.md',
            repo_root
            / "docs/plans/zircon_runtime/render/17/2026-08-11-render17-profiling-readiness-and-optimization-research.md",
            repo_root / 'docs/testing-and-validation/development-conventions.md',
            repo_root / 'docs/architecture/runtime-architecture-review-m0.md',
        )
        for mirror_path in mirrors:
            mirror = _read_contract_input(mirror_path, encoding="utf-8")
            self.assertIn(self.STATUS, mirror, mirror_path.as_posix())

        runtime_plan = _read_contract_input(mirrors[0], encoding="utf-8")
        for current_path in (
            "zircon_runtime/src/graphics/runtime/framework/frame_profiler.rs",
            "zircon_runtime/src/graphics/runtime/framework/frame_profiler/gpu_resolution.rs",
            "tools/tests/test_runtime_frame_profiler_gpu_resolution_owner_structure.py",
        ):
            self.assertIn(current_path, runtime_plan)


if __name__ == "__main__":
    unittest.main()
