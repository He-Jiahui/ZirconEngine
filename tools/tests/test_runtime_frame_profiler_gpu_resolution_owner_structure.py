import unittest
from pathlib import Path


class RuntimeFrameProfilerGpuResolutionOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_17_15_frame_profiler_gpu_resolution_owner_split_"
        "static_passed_cargo_profile_deferred"
    )

    def test_gpu_resolution_is_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = (
            repo_root
            / "zircon_runtime/src/graphics/runtime/render_framework/frame_profiler.rs"
        )
        child_path = (
            repo_root
            / "zircon_runtime/src/graphics/runtime/render_framework/frame_profiler/gpu_resolution.rs"
        )

        owner = owner_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(owner.splitlines()), 800)

        child = child_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(child.splitlines()), 800)
        self.assertIn("mod gpu_resolution;", owner)
        self.assertIn("use gpu_resolution::FrameProfileWrite;", owner)
        self.assertNotIn("struct FrameProfileWrite", owner)
        self.assertNotIn("fn merge_gpu_timer_result", owner)
        self.assertNotIn("fn merge_gpu_pipeline_statistics_result", owner)
        self.assertEqual(owner.count("#[test]"), 11)

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
            / "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            repo_root
            / "docs/plans/zircon_runtime/render/17-performance-and-profiling.md",
            repo_root
            / "docs/plans/zircon_runtime/render/17/2026-08-11-render17-profiling-readiness-and-optimization-research.md",
            repo_root / "docs/plans/engine-code-structure-convention.md",
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md",
        )
        for mirror_path in mirrors:
            mirror = mirror_path.read_text(encoding="utf-8")
            self.assertIn(self.STATUS, mirror, mirror_path.as_posix())

        runtime_plan = mirrors[0].read_text(encoding="utf-8")
        for current_path in (
            "zircon_runtime/src/graphics/runtime/render_framework/frame_profiler.rs",
            "zircon_runtime/src/graphics/runtime/render_framework/frame_profiler/gpu_resolution.rs",
            "tools/tests/test_runtime_frame_profiler_gpu_resolution_owner_structure.py",
        ):
            self.assertIn(current_path, runtime_plan)


if __name__ == "__main__":
    unittest.main()
