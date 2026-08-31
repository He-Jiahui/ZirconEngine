import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RUNTIME_SRC = REPO_ROOT / "zircon_runtime" / "src"
EDITOR_SRC = REPO_ROOT / "zircon_editor" / "src"


def read_runtime_source(relative_path: str) -> str:
    return (RUNTIME_SRC / relative_path).read_text(encoding="utf-8")


def read_editor_source(relative_path: str) -> str:
    return (EDITOR_SRC / relative_path).read_text(encoding="utf-8")


class RuntimeDynamicUiExtractGenerationContractTests(unittest.TestCase):
    def test_fallback_ui_cache_is_keyed_by_component_generations_and_viewport(self) -> None:
        session_root = read_runtime_source("dynamic_api/session.rs")
        cache = read_runtime_source("dynamic_api/session/ui_extract_cache.rs")

        self.assertIn("mod ui_extract_cache;", session_root)
        for anchor in (
            "struct RuntimeUiExtractCacheKey",
            "menu_generation",
            "hud_generations",
            "viewport_size",
            "dynamic_component_generation(GAMEPLAY_MENU_COMPONENT)",
            "HUD_COMPONENT_IDS",
            ".map(|component_id| world.dynamic_component_generation(component_id))",
        ):
            self.assertIn(anchor, cache)
        self.assertIn("Option<Arc<UiRenderExtract>>", cache)
        self.assertIn("runtime_session_menu_extract", cache)
        self.assertIn("runtime_session_hud_extract", cache)

    def test_stable_and_changed_generation_regressions_are_owned_by_the_cache(self) -> None:
        cache = read_runtime_source("dynamic_api/session/ui_extract_cache.rs")

        for test_name in (
            "stable_generation_reuses_the_same_ui_extract_allocation",
            "unrelated_world_mutation_keeps_the_cached_ui_extract",
            "target_component_mutation_rebuilds_the_ui_extract_once",
            "viewport_resize_rebuilds_the_ui_extract_once",
            "stable_absent_ui_does_not_revisit_component_rows",
        ):
            self.assertIn(f"fn {test_name}", cache)
        self.assertIn("Arc::ptr_eq", cache)

    def test_runtime_render_pipeline_uses_one_shared_ui_submission_handle(self) -> None:
        shared_handle_paths = (
            "dynamic_api/session/extract.rs",
            "dynamic_api/runtime_loop.rs",
            "core/framework/render/framework.rs",
            "graphics/runtime/render_framework/pipelined/queue.rs",
            "graphics/runtime/render_framework/render_framework_trait_binding/wgpu_framework.rs",
            "graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs",
            "graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs",
            "graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs",
            "graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs",
            "graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs",
            "graphics/types/viewport_render_frame.rs",
            "graphics/types/viewport_render_frame_with_ui.rs",
        )

        for relative_path in shared_handle_paths:
            source = read_runtime_source(relative_path)
            self.assertIn(
                "Arc<UiRenderSubmission>",
                source,
                f"{relative_path} must carry the segmented UI submission product",
            )
            self.assertNotIn(
                "Option<UiRenderSubmission>",
                source,
                f"{relative_path} must retain the shared submission allocation",
            )

    def test_project_ui_aggregate_reuses_its_published_submission(self) -> None:
        runtime_ui = read_runtime_source("dynamic_api/session/runtime_ui.rs")

        self.assertIn("render_cache: RuntimeUiAggregateRenderCache", runtime_ui)
        self.assertIn("Result<Option<Arc<UiRenderSubmission>>, UiTreeError>", runtime_ui)
        self.assertIn("Arc::clone", runtime_ui)
        self.assertIn(
            "stable_project_ui_submission_reuses_the_same_allocation",
            runtime_ui,
        )

    def test_editor_submission_chain_wraps_flat_producers_at_the_boundary(self) -> None:
        flat_producer_paths = (
            "ui/workbench/state/editor_state_render.rs",
            "ui/retained_host/viewport/submit_extract.rs",
            "ui/retained_host/viewport/world_space_ui.rs",
        )
        submission_consumer_paths = (
            "ui/retained_host/viewport/test_render_framework.rs",
            "ui/retained_host/viewport/tests/fake_render_framework.rs",
        )

        for relative_path in flat_producer_paths:
            source = read_editor_source(relative_path)
            self.assertIn("Arc<UiRenderExtract>", source, relative_path)
            self.assertNotIn("Option<UiRenderExtract>", source, relative_path)
        for relative_path in submission_consumer_paths:
            source = read_editor_source(relative_path)
            self.assertIn("Arc<UiRenderSubmission>", source, relative_path)
            self.assertNotIn("Option<UiRenderSubmission>", source, relative_path)

        submit = read_editor_source("ui/retained_host/viewport/submit_extract.rs")
        self.assertIn("ui.map(UiRenderSubmission::single)", submit)
        merge = read_editor_source("ui/retained_host/viewport/world_space_ui.rs")
        self.assertIn("return ui;", merge)
        self.assertIn("Arc::new", merge)

    def test_editor_viewport_hud_is_generation_owned(self) -> None:
        controller = read_editor_source(
            "scene/viewport/controller/scene_viewport_controller.rs"
        )
        construction = read_editor_source(
            "scene/viewport/controller/scene_viewport_controller_construction.rs"
        )
        overlay = read_editor_source(
            "scene/viewport/controller/scene_viewport_controller_build_runtime_overlay_ui.rs"
        )

        self.assertIn("runtime_overlay_ui_cache", controller)
        self.assertIn("runtime_overlay_ui_cache: Default::default()", construction)
        for anchor in (
            "struct RuntimeOverlayUiExtractCacheKey",
            "scene_mode_revision",
            "projection_mode",
            "display_mode",
            "grid_mode",
            "viewport_size",
            "Option<Arc<UiRenderExtract>>",
            "stable_viewport_hud_generation_reuses_the_same_allocation",
            "viewport_hud_key_change_publishes_one_new_allocation",
            "Arc::ptr_eq",
        ):
            self.assertIn(anchor, overlay)

    def test_editor_world_space_ui_merge_is_generation_owned(self) -> None:
        state = read_editor_source("ui/retained_host/viewport/viewport_state.rs")
        world_space = read_editor_source(
            "ui/retained_host/viewport/world_space_ui.rs"
        )

        self.assertIn("world_space_ui_generation", state)
        self.assertIn("world_space_ui_merge_cache", state)
        for anchor in (
            "struct WorldSpaceUiMergeCache",
            "source_generation",
            "Arc::ptr_eq",
            "stable_world_space_generation_reuses_the_merged_allocation",
            "empty_world_space_generation_preserves_the_base_allocation",
        ):
            self.assertIn(anchor, world_space)


if __name__ == "__main__":
    unittest.main()
