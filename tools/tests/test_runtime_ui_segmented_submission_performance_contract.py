from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SUBMISSION = ROOT / "zircon_runtime/src/core/framework/render/ui_submission.rs"
FRAMEWORK = ROOT / "zircon_runtime/src/core/framework/render/framework.rs"
RENDER_MOD = ROOT / "zircon_runtime/src/core/framework/render/mod.rs"
RUNTIME_UI = ROOT / "zircon_runtime/src/dynamic_api/session/runtime_ui.rs"
VIEWPORT_FRAME = ROOT / "zircon_runtime/src/graphics/types/viewport_render_frame.rs"
PUBLIC_RUNTIME_FRAME = ROOT / "zircon_runtime/src/ui/public_runtime_frame.rs"
RENDERER = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs"
RENDERER_TESTS = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests.rs"
RENDERER_CONTEXT_GUARDS = ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/"
    "render_pass_executor_registry/tests/renderer_context_guards.rs"
)
UI_TEXTURE = ROOT / "zircon_runtime/src/graphics/scene/resources/ui_texture.rs"
PROFILE_MANIFEST = ROOT / "tools/profile-capture-manifest.ps1"
PRODUCT_TEXT_RENDERERS = (
    ROOT / "zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/product_renderer.rs",
    ROOT / "zircon_runtime/tests/runtime_ui_text_render_contract.rs",
)
RUNTIME_UI_SOURCE_GUARDS = (
    ROOT
    / "zircon_runtime/src/tests/runtime_absorption/structure_convention/"
    "runtime_dead_code/runtime_ui.rs",
    ROOT / "zircon_runtime/src/tests/ui_boundary/runtime_host.rs",
)


class RuntimeUiSegmentedSubmissionPerformanceContract(unittest.TestCase):
    def test_render_framework_owns_a_segmented_ui_submission_contract(self) -> None:
        self.assertTrue(
            SUBMISSION.exists(),
            "render framework must own ui_submission.rs before consumers can hard-cut",
        )
        source = SUBMISSION.read_text(encoding="utf-8")
        render_mod = RENDER_MOD.read_text(encoding="utf-8")

        self.assertIn("pub struct UiRenderSubmission", source)
        self.assertIn("segments: Arc<[UiRenderSubmissionSegment]>", source)
        self.assertIn("extract: Arc<UiRenderFrameExtract>", source)
        self.assertIn("pub fn single", source)
        self.assertIn("pub fn from_segments", source)
        self.assertIn("pub fn from_frame_segments", source)
        self.assertIn("pub fn commands", source)
        self.assertIn("ordered_segments_preserve_extract_allocations", source)
        self.assertIn(
            "projected_segment_shares_commands_and_projects_only_route_identity",
            source,
        )
        self.assertIn("UiRenderNodeIdProjection", render_mod)
        self.assertIn("UiRenderSubmissionSegment", render_mod)

    def test_product_submission_path_has_no_flat_extract_authority(self) -> None:
        framework = FRAMEWORK.read_text(encoding="utf-8")
        viewport_frame = VIEWPORT_FRAME.read_text(encoding="utf-8")
        public_runtime_frame = PUBLIC_RUNTIME_FRAME.read_text(encoding="utf-8")
        renderer_context_guards = RENDERER_CONTEXT_GUARDS.read_text(encoding="utf-8")

        self.assertIn("Option<Arc<UiRenderSubmission>>", framework)
        self.assertNotIn("Option<Arc<UiRenderExtract>>", framework)
        self.assertIn("pub ui: Option<Arc<UiRenderSubmission>>", viewport_frame)
        self.assertNotIn("pub ui: Option<Arc<UiRenderExtract>>", viewport_frame)
        self.assertIn("pub ui: Option<Arc<UiRenderSubmission>>", public_runtime_frame)
        self.assertNotIn("pub ui: Option<UiRenderExtract>", public_runtime_frame)
        for source_guard in RUNTIME_UI_SOURCE_GUARDS:
            source = source_guard.read_text(encoding="utf-8")
            self.assertIn("ui: Option<Arc<UiRenderSubmission>>", source)
            self.assertNotIn("ui: Option<UiRenderExtract>", source)
        self.assertIn("UiRenderSubmission::single", renderer_context_guards)
        self.assertNotIn(
            ".with_ui(Some(std::sync::Arc::new(test_ui_extract())))",
            renderer_context_guards,
        )

    def test_runtime_project_publication_keeps_segments_and_never_flattens(self) -> None:
        source = RUNTIME_UI.read_text(encoding="utf-8")
        self.assertIn(
            "pub(super) fn render_submission",
            source,
            "runtime project UI must publish the render submission authority",
        )
        publication = source.split("pub(super) fn render_submission", 1)[1].split(
            "pub(super) fn accessibility_snapshot", 1
        )[0]

        self.assertIn("UiRenderSubmission::from_submission_segments(segments)", publication)
        self.assertIn(
            "local_surface_change_reuses_unchanged_segment_allocation",
            source,
        )
        self.assertIn("surface.render_frame_extract()", publication)
        self.assertIn("UiRenderSubmissionSegment::projected(", publication)
        self.assertNotIn("global_node_id(surface_index, command.node_id)", publication)
        self.assertNotIn("UiRenderList { commands }", publication)
        self.assertNotIn("commands.extend", publication)
        self.assertNotIn("UiRenderList { commands }", publication)
        self.assertNotIn("segment.iter().cloned()", publication)

    def test_renderer_and_texture_discovery_iterate_submission_segments(self) -> None:
        renderer = RENDERER.read_text(encoding="utf-8")
        texture = UI_TEXTURE.read_text(encoding="utf-8")
        self.assertIn(
            "fn plan_screen_space_ui_extract_batches",
            renderer,
            "product planner must delegate segment contents to the extract planner",
        )
        product_planner = renderer.split(
            "fn plan_screen_space_ui_batches_with_framebuffer_background", 1
        )[1].split("fn plan_screen_space_ui_extract_batches", 1)[0]

        self.assertIn("submission: &UiRenderSubmission", product_planner)
        self.assertIn("for segment in submission.segments()", product_planner)
        self.assertIn("extract: &UiRenderFrameExtract", renderer)
        self.assertIn("segment.project_node_id(node_id)", renderer)
        self.assertIn("submission.commands()", texture)
        self.assertNotIn("extract: &UiRenderExtract", texture)
        self.assertIn("ui_texture_discovery_walks_all_submission_segments", texture)
        self.assertIn(
            "screen_space_ui_plan_blocks_framebuffer_background_across_segments",
            RENDERER_TESTS.read_text(encoding="utf-8"),
        )

    def test_product_ownership_chain_is_bound_to_profile_evidence(self) -> None:
        manifest = PROFILE_MANIFEST.read_text(encoding="utf-8")

        for relative_path in (
            "zircon_editor/src/ui/retained_host/viewport/submit_extract.rs",
            "zircon_runtime/src/core/framework/render/framework.rs",
            "zircon_runtime/src/core/framework/render/ui_submission.rs",
            "zircon_runtime/src/dynamic_api/session/runtime_ui.rs",
            "zircon_runtime/src/graphics/scene/resources/ui_texture.rs",
            "zircon_runtime/src/graphics/types/viewport_render_frame.rs",
        ):
            self.assertIn(f'"{relative_path}"', manifest)

    def test_product_text_renderers_wrap_flat_fixtures_once_at_the_boundary(self) -> None:
        for source_path in PRODUCT_TEXT_RENDERERS:
            source = source_path.read_text(encoding="utf-8")
            self.assertIn("UiRenderSubmission::single", source)
            self.assertIn("Some(Arc::clone(&submission))", source)
            self.assertNotIn("Some(ui.clone())", source)


if __name__ == "__main__":
    unittest.main()
