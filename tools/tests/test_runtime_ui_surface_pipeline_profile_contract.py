from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
REBUILD = REPO_ROOT / "zircon_runtime/src/ui/surface/surface/rebuild.rs"
REBUILD_INCREMENTAL = (
    REPO_ROOT / "zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs"
)
REBUILD_REPORT = REPO_ROOT / "zircon_runtime/src/ui/surface/surface/rebuild/report.rs"
PROJECTED_HIT = REPO_ROOT / "zircon_runtime/src/ui/surface/frame_hit_test.rs"
NAVIGATION = REPO_ROOT / "zircon_runtime/src/ui/surface/navigation_index.rs"
NAVIGATION_PROFILE = REPO_ROOT / "zircon_runtime/src/ui/surface/navigation_index/profile.rs"
FRAME_PUBLICATION = (
    REPO_ROOT / "zircon_runtime/src/ui/surface/surface/frame_publication.rs"
)
CAPTURE = REPO_ROOT / "tools/ui-profile-capture.ps1"
CAPTURE_MANIFEST = REPO_ROOT / "tools/profile-capture-manifest.ps1"
METRICS = REPO_ROOT / "tools/ui-profile-surface-pipeline-metrics.ps1"


class RuntimeUiSurfacePipelineProfileContract(unittest.TestCase):
    def test_surface_rebuild_exports_stage_time_and_work_from_one_authority(self) -> None:
        source = REBUILD.read_text(encoding="utf-8")
        incremental_source = REBUILD_INCREMENTAL.read_text(encoding="utf-8")
        profile_source = REBUILD_REPORT.read_text(encoding="utf-8")

        for counter in (
            "ui.surface_rebuild.total_elapsed_us",
            "ui.surface_rebuild.dirty_node_count",
            "ui.surface_rebuild.layout_elapsed_us",
            "ui.surface_rebuild.post_layout_elapsed_us",
            "ui.surface_rebuild.base_picking_elapsed_us",
            "ui.surface_rebuild.render_extract_elapsed_us",
            "ui.surface_rebuild.layout_visited_node_count",
            "ui.surface_rebuild.arranged_outer_node_visit_count",
            "ui.surface_rebuild.hit_grid_outer_node_visit_count",
            "ui.surface_rebuild.render_outer_node_visit_count",
            "ui.surface_rebuild.render_command_reused_count",
            "ui.surface_rebuild.render_command_rebuilt_count",
        ):
            self.assertIn(counter, profile_source)
        self.assertIn("#[cfg(feature = \"profiling\")]", profile_source)
        self.assertIn("mod incremental;", source)
        self.assertIn("use report::record_surface_rebuild_profile", source)
        self.assertEqual(source.count("record_surface_rebuild_profile("), 2)
        self.assertEqual(incremental_source.count("record_surface_rebuild_profile("), 2)

    def test_post_hit_and_publication_stages_are_measured_at_their_owners(self) -> None:
        projected_hit = PROJECTED_HIT.read_text(encoding="utf-8")
        navigation = NAVIGATION.read_text(encoding="utf-8")
        navigation_profile = NAVIGATION_PROFILE.read_text(encoding="utf-8")
        frame_publication = FRAME_PUBLICATION.read_text(encoding="utf-8")

        for counter in (
            "ui.surface_projected_hit.rebuild_elapsed_us",
            "ui.surface_projected_hit.patch_elapsed_us",
            "ui.surface_projected_hit.patch_fallback_count",
            "ui.surface_projected_hit.affected_entry_count",
        ):
            self.assertIn(counter, projected_hit)
        self.assertIn("mod profile;", navigation)
        self.assertIn("ui.navigation_index.rebuild_elapsed_us", navigation_profile)
        self.assertIn("ui.surface_frame.publication_elapsed_us", frame_publication)

    def test_profile_capture_exports_and_source_binds_pipeline_metrics(self) -> None:
        self.assertTrue(METRICS.is_file())
        capture = CAPTURE.read_text(encoding="utf-8")
        manifest = CAPTURE_MANIFEST.read_text(encoding="utf-8")
        metrics = METRICS.read_text(encoding="utf-8")

        self.assertIn("ui-profile-surface-pipeline-metrics.ps1", capture)
        self.assertIn("Export-ZirconUiSurfacePipelineMetrics", capture)
        self.assertIn("tools/ui-profile-surface-pipeline-metrics.ps1", manifest)
        self.assertIn("ui_surface_pipeline_metrics.json", metrics)
        self.assertIn("surface_rebuild", metrics)
        self.assertIn("frame_publication", metrics)


if __name__ == "__main__":
    unittest.main()
