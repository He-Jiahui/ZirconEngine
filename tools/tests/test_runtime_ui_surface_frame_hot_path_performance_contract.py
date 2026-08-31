from pathlib import Path
import unittest

from tools.ui_surface_frame_hot_path_pressure import run


ROOT = Path(__file__).resolve().parents[2]
RUNTIME_UI = ROOT / "zircon_runtime/src/dynamic_api/session/runtime_ui.rs"


class RuntimeUiSurfaceFrameHotPathPerformanceContractTests(unittest.TestCase):
    def test_runtime_render_reads_the_pre_published_persistent_render_domain(self) -> None:
        source = RUNTIME_UI.read_text(encoding="utf-8")
        start = source.index("    pub(super) fn render_submission(")
        end = source.index("\n    pub(super) fn accessibility_snapshot(", start)
        render_submission = source[start:end]

        self.assertNotIn(".surface_frame()", render_submission)
        self.assertNotIn("frame_scratch", source)
        self.assertIn("invalidation_generations().render", render_submission)
        self.assertIn("surface.render_frame_extract()", render_submission)
        self.assertNotIn("&surface.render_extract", render_submission)

    def test_pressure_model_eliminates_heavy_domain_clones(self) -> None:
        result = run(
            surface_count=2,
            render_command_count=4_096,
            hit_entry_count=2_048,
            hit_cell_entry_count=8_192,
            interactive_update_count=1_024,
        )

        self.assertEqual(result["new_surface_frame_materialization_count"], 0)
        self.assertEqual(result["new_generation_check_count"], 2_048)
        self.assertGreater(result["eliminated_clone_work"], 1_000_000)
        self.assertGreater(result["work_reduction_ratio"], 10_000)


if __name__ == "__main__":
    unittest.main()
