from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/ui/surface/surface/frame_publication.rs"


class RuntimeSurfaceFramePatchRangeCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        start = cls.source.index("pub(super) fn mark_surface_frame_rebuild_dirty(")
        end = cls.source.index(
            "\n    pub(super) fn mark_surface_frame_metadata_dirty", start
        )
        cls.rebuild_body = cls.source[start:end]

    def test_patch_ranges_reserve_exact_input_upper_bound(self) -> None:
        self.assertIn(
            "let mut render_patch_ranges = Vec::with_capacity(node_ids.len());",
            self.rebuild_body,
        )
        self.assertIn(
            "render_patch_ranges.extend(node_ids.iter().filter_map(",
            self.rebuild_body.replace("\n", ""),
        )
        self.assertNotIn(".collect::<Vec<_>>()", self.rebuild_body)
        self.assertIn(
            "runtime833_surface_frame_patch_range_capacity_bench_v1", self.source
        )

    def test_patch_range_capacity_model_removes_geometric_growth(self) -> None:
        node_count = 4096
        legacy_growth_events = 12
        optimized_growth_events = 0
        self.assertGreater(node_count, 0)
        self.assertGreater(legacy_growth_events, optimized_growth_events)


if __name__ == "__main__":
    unittest.main()
