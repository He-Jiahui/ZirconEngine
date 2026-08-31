import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
TEXT_SHAPE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/text_shape.rs"
)
PROFILE_BENCH = (
    ROOT
    / "zircon_runtime_interface/src/ui/surface/render/text_shape/projection_profile.rs"
)
PROFILE_PLAN = (
    ROOT
    / "docs/plans/zircon_runtime/text/09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md"
)


class RuntimeTextPaintRunProjectionProfileContractTests(unittest.TestCase):
    def test_private_profile_owner_calls_the_exact_production_helper(self) -> None:
        root = TEXT_SHAPE.read_text(encoding="utf-8")
        profile = PROFILE_BENCH.read_text(encoding="utf-8")

        self.assertIn('path = "text_shape/projection_profile.rs"', root)
        self.assertIn("mod projection_profile;", root)
        self.assertIn("text_paint_runs_from_resolved_layout(", profile)
        self.assertNotIn("resolved_text_run_frame(", profile)

    def test_release_harness_uses_fixed_dense_run_lanes_and_raw_samples(self) -> None:
        profile = PROFILE_BENCH.read_text(encoding="utf-8")

        self.assertIn("const SAMPLE_COUNT: usize = 31;", profile)
        self.assertIn("const DENSE_RUN_COUNTS: [usize; 4] = [1, 100, 1_000, 10_000];", profile)
        self.assertIn("#[ignore =", profile)
        self.assertIn("samples_ns={samples_ns:?}", profile)
        self.assertIn("rss_delta_bytes={rss_delta_bytes:?}", profile)
        self.assertIn("p50_ns={p50_ns}", profile)
        self.assertIn("p95_ns={p95_ns}", profile)
        self.assertIn("p99_ns={p99_ns}", profile)

    def test_harness_reports_static_work_and_does_not_write_artifacts(self) -> None:
        profile = PROFILE_BENCH.read_text(encoding="utf-8")

        self.assertIn("implied_full_line_grapheme_visits", profile)
        self.assertIn("current_rss_bytes", profile)
        self.assertNotIn("target/", profile)
        self.assertNotIn("File::create", profile)
        self.assertNotIn("fs::write", profile)

    def test_plan_keeps_the_baseline_unmeasured_until_the_harness_runs(self) -> None:
        plan = PROFILE_PLAN.read_text(encoding="utf-8")

        self.assertIn("baseline_profile_pending_no_optimization", plan)
        self.assertIn("The test has not run under Cargo", plan)
        self.assertIn("1/100/1k/10k runs", plan)


if __name__ == "__main__":
    unittest.main()
