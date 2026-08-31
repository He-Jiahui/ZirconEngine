from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/dynamic_api/session/runtime_ui.rs"


class RuntimeUiAggregateRenderSegmentPerformanceContractTests(unittest.TestCase):
    def test_runtime_surfaces_publish_the_surface_owned_persistent_render_domain(self):
        source = SOURCE.read_text(encoding="utf-8")
        self.assertNotIn("struct RuntimeUiSurfaceRenderCache", source)
        self.assertNotIn("render_cache: RuntimeUiSurfaceRenderCache", source)
        self.assertNotIn("segment_for_extract", source)
        self.assertIn("surface.render_frame_extract()", source)
        self.assertIn("UiRenderSubmission::from_submission_segments(segments)", source)
        self.assertNotIn("frame_scratch", source)
        self.assertNotIn("segment_command_projection_clone_count", source)
        self.assertNotIn("global_node_id(surface_index, command.node_id)", source)

    def test_aggregate_publishes_segment_handles_without_flattening(self):
        source = SOURCE.read_text(encoding="utf-8")
        self.assertNotIn(
            "frame.render_extract.list.commands.iter().cloned().map(",
            source,
        )
        self.assertIn("UiRenderSubmission::from_submission_segments(segments)", source)
        self.assertIn("local_surface_change_reuses_unchanged_segment_allocation", source)
        self.assertNotIn("commands.extend", source)
        self.assertNotIn("segment.iter().cloned()", source)


if __name__ == "__main__":
    unittest.main()
