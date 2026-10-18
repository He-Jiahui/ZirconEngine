import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/ecs/compute.rs"
BENCH_SOURCE = ROOT / "zircon_runtime_interface/src/ui/ecs/compute/tests/dirty_reason_performance_tests.rs"


class RuntimeInterface03EcsDirtyReasonMaskPerformanceContract(unittest.TestCase):
    def test_projection_buckets_use_fixed_dirty_reason_mask(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")

        self.assertNotIn("BTreeSet<UiPipelineDirtyReason>", source)
        self.assertIn("struct ProjectionDirtyReasonMask(u16);", source)
        self.assertIn("fn insert(&mut self, reason: UiPipelineDirtyReason)", source)
        self.assertIn("fn into_vec(self) -> Vec<UiPipelineDirtyReason>", source)
        self.assertIn("dirty_reasons: ProjectionDirtyReasonMask", source)

    def test_dirty_reason_mask_preserves_declaration_order(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn into_vec(self) -> Vec<UiPipelineDirtyReason>")
        end = source.index("\n    }", start) + 6
        body = source[start:end]

        for reason in [
            "Input",
            "Focus",
            "WidgetBehavior",
            "Text",
            "Style",
            "Layout",
            "LayoutMetrics",
            "Picking",
            "HitGrid",
            "A11y",
            "Render",
            "Template",
            "Window",
            "HostRequest",
            "Diagnostics",
        ]:
            self.assertIn(f"UiPipelineDirtyReason::{reason}", body)

    def test_fixed_dirty_reason_mask_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench_source = BENCH_SOURCE.read_text(encoding="utf-8")

        self.assertIn("mod dirty_reason_performance_tests;", source)
        self.assertIn(
            "RUNTIME_INTERFACE03_FIXED_ECS_DIRTY_REASON_MASK_BENCH_V1",
            bench_source,
        )
        self.assertIn("saturating_mul(5)", bench_source)
        self.assertIn("saturating_mul(4)", bench_source)


if __name__ == "__main__":
    unittest.main()
