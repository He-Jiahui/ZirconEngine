import hashlib
from pathlib import Path
import unittest

from tools.ui_surface_frame_domain_sharing_pressure import run


ROOT = Path(__file__).resolve().parents[2]
RUNTIME_SURFACE = ROOT / "zircon_runtime/src/ui/surface"
INTERFACE_SURFACE = ROOT / "zircon_runtime_interface/src/ui/surface"
EDITOR_TOOLBAR = ROOT / "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer"
PROFILE_COUNTER_EVIDENCE = ROOT / "tools/ui-profile-counter-evidence.ps1"


def read_surface_rebuild_source() -> str:
    rebuild_root = RUNTIME_SURFACE / "surface/rebuild"
    return (
        rebuild_root.with_suffix(".rs").read_text(encoding="utf-8")
        + (rebuild_root / "incremental.rs").read_text(encoding="utf-8")
    )


def function_source(source: str, start_anchor: str, end_anchor: str) -> str:
    start = source.index(start_anchor)
    end = source.index(end_anchor, start)
    return source[start:end]


class RuntimeUiSurfaceFrameDomainSharingPerformanceContractTests(unittest.TestCase):
    def test_public_frame_exposes_arc_domains_and_independent_generations(self) -> None:
        source = (INTERFACE_SURFACE / "frame.rs").read_text(encoding="utf-8")

        self.assertIn("pub struct UiSurfaceFrameDomainGenerations", source)
        for field in (
            "arranged_tree",
            "render_extract",
            "hit_grid",
            "focus_state",
            "focus_path",
            "pipeline_report",
        ):
            self.assertRegex(source, rf"pub {field}: Arc<[^>]+>")
        self.assertNotIn("pub ecs_projection:", source)
        for generation in (
            "layout",
            "render",
            "hit_test",
            "focus",
            "pipeline",
            "window",
        ):
            self.assertIn(f"pub {generation}: u64", source)
        self.assertNotIn("pub ecs: u64", source)

    def test_render_frame_domain_is_an_immutable_segmented_snapshot(self) -> None:
        frame = (INTERFACE_SURFACE / "frame.rs").read_text(encoding="utf-8")
        render = (
            INTERFACE_SURFACE / "render/frame_extract.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub render_extract: Arc<UiRenderFrameExtract>", frame)
        self.assertNotIn("pub render_extract: Arc<UiRenderExtract>", frame)
        self.assertIn("pub const UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE: usize = 64", render)
        self.assertIn("const UI_RENDER_FRAME_DIRECTORY_FANOUT: usize = 32", render)
        self.assertIn("pub struct UiRenderFrameCommands", render)
        self.assertIn("root: Option<Arc<UiRenderFrameCommandNode>>", render)
        self.assertIn("pub fn patch_ranges_from_extract", render)
        self.assertIn("Arc::ptr_eq", render)
        self.assertIn("deep_directory_patch_clones_one_node_per_level", render)
        iterator = function_source(
            render,
            "pub struct UiRenderFrameCommandsIter<'a>",
            "\nimpl ExactSizeIterator",
        )
        self.assertIn("directory_stack", iterator)
        self.assertNotIn("Vec<", iterator)

    def test_local_render_publication_consumes_exact_ranges_without_global_scan(self) -> None:
        publication = (RUNTIME_SURFACE / "surface/frame_publication.rs").read_text(
            encoding="utf-8"
        )
        rebuild = read_surface_rebuild_source()

        self.assertIn("render_patch_ranges", publication)
        self.assertIn("record_render_patch_ranges", publication)
        self.assertIn("patch_ranges_from_extract", publication)
        self.assertIn("render_metadata_changed", publication)
        self.assertNotIn("self.render_extract.clone()", publication)
        self.assertNotIn("self.render_extract.list.commands.iter()", publication)
        self.assertIn("render_local_patch_node_ids", rebuild)
        self.assertGreaterEqual(rebuild.count("render_local_patch_node_ids"), 3)

        tests = (
            ROOT
            / "zircon_runtime/src/ui/tests/surface_frame_authority/arranged_authority.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "render_only_publication_reuses_untouched_command_segments", tests
        )

    def test_publication_reuses_clean_domains_and_counts_only_real_clones(self) -> None:
        source = (RUNTIME_SURFACE / "surface/frame_publication.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("fn shared_domain<T>", source)
        self.assertIn("return Arc::clone(previous);", source)
        self.assertIn("let layout_changed = dirty_domains.layout", source)
        self.assertIn("if layout_changed {", source)
        self.assertIn("if hit_test_changed {", source)
        self.assertIn("let pipeline_changed = dirty_domains.pipeline", source)
        self.assertIn('"ui.surface_frame.pipeline_stage_build_count"', source)

    def test_ecs_projection_is_explicit_debug_work_not_a_frame_domain(self) -> None:
        frame = (INTERFACE_SURFACE / "frame.rs").read_text(encoding="utf-8")
        publication = (RUNTIME_SURFACE / "surface/frame_publication.rs").read_text(
            encoding="utf-8"
        )
        surface = (RUNTIME_SURFACE / "surface.rs").read_text(encoding="utf-8")
        counter_evidence = PROFILE_COUNTER_EVIDENCE.read_text(encoding="utf-8")

        self.assertNotIn("ecs_projection", frame)
        self.assertNotIn("dirty_domains.ecs", publication)
        self.assertNotIn("ui_ecs_projection()", publication)
        self.assertNotIn("ui.surface_frame.ecs_projection_node_count", publication)
        self.assertNotIn("ui.surface_frame.ecs_projection_node_count", counter_evidence)
        self.assertIn("ui.surface_frame.pipeline_stage_build_count", counter_evidence)
        self.assertIn("debug_surface_frame_with_ecs_projection", surface)
        self.assertIn("debug_surface_frame_for_pick_with_ecs_projection", surface)
        self.assertIn("debug_surface_frame_for_selection_with_ecs_projection", surface)

    def test_rebuild_marks_only_changed_publication_domains(self) -> None:
        source = read_surface_rebuild_source()
        tests = (
            ROOT
            / "zircon_runtime/src/ui/tests/surface_frame_authority/arranged_authority.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("mark_surface_frame_metadata_dirty()", source)
        self.assertIn("mark_surface_frame_rebuild_dirty(", source)
        self.assertIn("report.hit_grid_rebuilt || projected_hit_changed", source)
        self.assertIn("report.hit_grid_rebuilt || projected_geometry_changed", source)
        self.assertIn("render_only_rebuild_preserves_layout_and_hit_domains", tests)
        self.assertIn("window_only_publication_reuses_all_heavy_domains", tests)
        self.assertGreaterEqual(tests.count("&before.pipeline_report"), 3)
        self.assertGreaterEqual(tests.count("&after.pipeline_report"), 3)
        self.assertIn("before.domain_generations.pipeline", tests)
        self.assertIn("after.domain_generations.pipeline", tests)

    def test_dirty_rebuild_publishes_before_render_reads_the_render_domain(self) -> None:
        publication = (RUNTIME_SURFACE / "surface/frame_publication.rs").read_text(
            encoding="utf-8"
        )
        rebuild = read_surface_rebuild_source()
        incremental_rebuild = (
            RUNTIME_SURFACE / "surface/rebuild/incremental.rs"
        ).read_text(encoding="utf-8")
        runtime_ui = (
            ROOT / "zircon_runtime/src/dynamic_api/session/runtime_ui.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "pub(crate) fn publish_surface_frame_after_rebuild(&mut self)",
            publication,
        )
        self.assertIn("let _ = self.surface_frame();", publication)
        self.assertGreaterEqual(rebuild.count("self.publish_surface_frame_after_rebuild();"), 3)
        dirty_rebuild = function_source(
            incremental_rebuild,
            "    pub fn rebuild_dirty(",
            "\n}\n\nfn should_use_full_layout_rebuild",
        )
        self.assertGreaterEqual(
            dirty_rebuild.count("self.publish_surface_frame_after_rebuild();"), 2
        )
        render_submission = function_source(
            runtime_ui,
            "    pub(super) fn render_submission(",
            "\n    pub(super) fn accessibility_snapshot(",
        )
        self.assertNotIn("publish_surface_frame_after_rebuild", render_submission)
        self.assertNotIn(".surface_frame()", render_submission)
        self.assertIn("invalidation_generations().render", render_submission)
        self.assertIn("surface.render_frame_extract()", render_submission)
        self.assertNotIn("&surface.render_extract", render_submission)
        pointer_dispatch = function_source(
            runtime_ui,
            "    fn dispatch_pointer_to_surface(",
            "\n    pub(super) fn next_input_metadata(",
        )
        self.assertNotIn("publish_surface_frame_after_rebuild", pointer_dispatch)

    def test_toolbar_subscribes_to_hit_domain_identity_not_outer_frame(self) -> None:
        bridge = (EDITOR_TOOLBAR / "viewport_toolbar_pointer_bridge.rs").read_text(
            encoding="utf-8"
        )
        sync = (EDITOR_TOOLBAR / "sync_surface_frame.rs").read_text(encoding="utf-8")

        self.assertIn("Weak<UiHitTestGrid>", bridge)
        self.assertIn("Arc::as_ptr(&surface_frame.hit_grid)", sync)
        self.assertIn("Arc::downgrade(&surface_frame.hit_grid)", sync)
        self.assertNotIn("Arc::as_ptr(surface_frame)", sync)

    def test_focus_diagnostic_histories_are_bounded_at_the_owner(self) -> None:
        focus = (RUNTIME_SURFACE / "focus.rs").read_text(encoding="utf-8")
        tests = (
            ROOT / "zircon_runtime/src/ui/tests/focus_navigation/focus_state.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("UI_FOCUS_DIAGNOSTIC_HISTORY_CAPACITY: usize = 64", focus)
        self.assertIn("fn push_focus_diagnostic<T>", focus)
        self.assertNotIn("self.focus.changes.push(", focus)
        self.assertNotIn("self.focus.focused_inputs.push(", focus)
        self.assertIn(
            "focus_diagnostic_histories_are_bounded_and_keep_the_newest_event", tests
        )

    def test_layout_publication_validates_focus_topology_without_recloning_state(self) -> None:
        publication = (RUNTIME_SURFACE / "surface/frame_publication.rs").read_text(
            encoding="utf-8"
        )
        arranged = (RUNTIME_SURFACE / "arranged.rs").read_text(encoding="utf-8")
        counter_evidence = PROFILE_COUNTER_EVIDENCE.read_text(encoding="utf-8")
        tests = (
            ROOT
            / "zircon_runtime/src/ui/tests/surface_frame_authority/arranged_authority.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("arranged_focus_path_matches_indexed", publication)
        self.assertIn("let focus_path_changed", publication)
        self.assertIn("focus: false", publication)
        self.assertIn("pub(crate) fn arranged_focus_path_matches_indexed", arranged)
        self.assertNotIn("arranged_focus_path_indexed(", arranged.split(
            "pub(crate) fn arranged_focus_path_matches_indexed", 1
        )[1].split("pub fn is_arranged_render_visible", 1)[0])
        self.assertIn("layout_only_resize_reuses_stable_focus_domains", tests)
        self.assertIn("indexed_focus_path_validation_rejects_a_reparented_route", arranged)
        for counter_name in (
            "ui.surface_frame.focus_state_build_count",
            "ui.surface_frame.focus_path_build_count",
            "ui.surface_frame.focus_path_validation_node_count_upper_bound",
        ):
            self.assertIn(counter_name, publication)
            self.assertIn(counter_name, counter_evidence)

    def test_layout_focus_pressure_model_replaces_clones_with_bounded_validation(self) -> None:
        result = run(
            arranged_node_count=8_192,
            render_command_count=4_096,
            hit_entry_count=4_096,
            hit_cell_entry_count=16_384,
            focus_node_count=128,
            pipeline_stage_count=8,
            window_only_update_count=1_024,
            render_only_update_count=256,
            layout_only_update_count=4_096,
        )

        self.assertEqual(result["historical_layout_focus_clone_work"], 128 * 4_096)
        self.assertEqual(result["new_layout_focus_clone_work"], 0)
        self.assertEqual(
            result["focus_path_validation_node_visit_upper_bound"],
            8 * 4_096,
        )
        self.assertEqual(
            result["eliminated_layout_focus_clone_work"],
            128 * 4_096,
        )
        self.assertEqual(result["layout_focus_clone_to_validation_ratio"], 16.0)

    def test_pressure_model_is_bound_to_current_runtime_and_unreal_sources(self) -> None:
        result = run(
            arranged_node_count=8_192,
            render_command_count=4_096,
            hit_entry_count=4_096,
            hit_cell_entry_count=16_384,
            focus_node_count=128,
            pipeline_stage_count=8,
            window_only_update_count=1_024,
            render_only_update_count=256,
            layout_only_update_count=4_096,
        )

        self.assertEqual(
            result["schema"],
            "zircon.runtime.ui_surface_frame_domain_sharing_pressure.v2",
        )
        binding = result["source_binding"]
        expected_paths = {
            "tools/ui_surface_frame_domain_sharing_pressure.py",
            "tools/ui-profile-counter-evidence.ps1",
            "zircon_runtime/src/ui/surface/arranged.rs",
            "zircon_runtime/src/ui/surface/surface/frame_publication.rs",
            "zircon_runtime_interface/src/ui/surface/focus_state.rs",
            "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp",
        }
        self.assertEqual(set(binding["critical_source_files"]), expected_paths)
        for relative_path, expected_hash in binding["source_sha256"].items():
            actual_hash = hashlib.sha256((ROOT / relative_path).read_bytes()).hexdigest().upper()
            self.assertEqual(actual_hash, expected_hash)
        manifest_payload = "\n".join(
            f"{path}:{binding['source_sha256'][path]}"
            for path in sorted(binding["source_sha256"])
        ).encode("utf-8")
        self.assertEqual(
            hashlib.sha256(manifest_payload).hexdigest().upper(),
            binding["source_manifest_sha256"],
        )

    def test_pressure_model_shares_clean_domains_across_window_and_render_updates(self) -> None:
        result = run(
            arranged_node_count=8_192,
            render_command_count=4_096,
            hit_entry_count=4_096,
            hit_cell_entry_count=16_384,
            focus_node_count=128,
            pipeline_stage_count=8,
            window_only_update_count=1_024,
            render_only_update_count=256,
            changed_render_command_count=1,
            render_segment_size=64,
            directory_fanout=32,
            owned_payload_bytes_per_command=24,
        )

        self.assertEqual(result["new_window_only_element_clone_work"], 0)
        self.assertEqual(
            result["new_render_only_element_clone_work"],
            4_096 * 256,
        )
        self.assertEqual(result["new_changed_domain_clone_work"], 4_096 * 256)
        self.assertGreater(result["eliminated_element_clone_work"], 10_000_000)
        self.assertGreater(result["element_clone_reduction_ratio"], 30)

    def test_pressure_model_rejects_arc_cow_as_a_render_publication_fix(self) -> None:
        result = run(
            arranged_node_count=8_192,
            render_command_count=4_096,
            hit_entry_count=4_096,
            hit_cell_entry_count=16_384,
            focus_node_count=128,
            pipeline_stage_count=8,
            window_only_update_count=1_024,
            render_only_update_count=256,
            changed_render_command_count=1,
            render_segment_size=64,
            directory_fanout=32,
            owned_payload_bytes_per_command=24,
        )

        current_clone_work = 4_096 * 256
        segmented_clone_work = 64 * 256
        self.assertEqual(
            result["current_render_publication_command_clone_work"],
            current_clone_work,
        )
        self.assertEqual(
            result["direct_arc_cow_command_clone_work"],
            current_clone_work,
        )
        self.assertEqual(
            result["persistent_segment_command_clone_work"],
            segmented_clone_work,
        )
        self.assertEqual(result["persistent_directory_depth"], 2)
        self.assertEqual(result["persistent_directory_node_clone_work"], 2 * 256)
        self.assertEqual(
            result["current_render_publication_owned_payload_clone_bytes"],
            current_clone_work * 24,
        )
        self.assertEqual(
            result["persistent_segment_owned_payload_clone_bytes"],
            segmented_clone_work * 24,
        )
        self.assertEqual(result["persistent_segment_clone_reduction_ratio"], 64)


if __name__ == "__main__":
    unittest.main()
