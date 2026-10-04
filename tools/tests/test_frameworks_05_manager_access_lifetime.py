from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


class Frameworks05ManagerAccessLifetimeTests(unittest.TestCase):
    def test_ui_text_resolves_project_asset_manager_only_for_bounded_operations(self) -> None:
        text_source = (
            REPO_ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs"
        ).read_text(encoding="utf-8")
        construct_source = (
            REPO_ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/construct.rs"
        ).read_text(encoding="utf-8")
        render_source = (
            REPO_ROOT
            / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/record.rs"
        ).read_text(encoding="utf-8")
        scene_source = (
            REPO_ROOT
            / "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs"
        ).read_text(encoding="utf-8")
        graph_source = (
            REPO_ROOT
            / "zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/surface.rs"
        ).read_text(encoding="utf-8")

        constructor_source = text_source.split(
            "    pub(super) fn new_with_font_collection(", 1
        )[1].split("    pub(super) fn published_font_collection_revision(", 1)[0]
        prepare_source = text_source.split("    pub(super) fn prepare(", 1)[1].split(
            "    pub(super) fn clear_frame_state(", 1
        )[0]

        self.assertIn("asset_manager: ProjectAssetManagerAccess", text_source)
        self.assertNotIn("asset_manager: Arc<ProjectAssetManager>", text_source)
        self.assertEqual(constructor_source.count(".resolve()?"), 1)
        self.assertEqual(
            constructor_source.count(
                "let resolved_asset_manager = asset_manager.resolve()?;"
            ),
            1,
        )
        self.assertEqual(prepare_source.count(".resolve()"), 1)
        self.assertEqual(
            prepare_source.count(
                "let asset_manager = self\n"
                "                .asset_manager\n"
                "                .resolve()\n"
                "                .map_err(|error| GraphicsError::Asset(error.to_string()))?;"
            ),
            1,
        )
        self.assertIn(") -> Result<Self, CoreError> {", text_source)
        self.assertIn(") -> Result<(), GraphicsError> {", text_source)

        self.assertNotIn("asset_manager.resolve()", construct_source)
        self.assertIn(
            "ScreenSpaceUiTextSystem::new_with_font_collection(", construct_source
        )
        self.assertIn(".map_err(|error| GraphicsError::Asset(error.to_string()))?", construct_source)

        self.assertIn(
            ") -> Result<ScreenSpaceUiPreparedUpload, GraphicsError> {", render_source
        )
        self.assertIn("self.text_system.prepare(", render_source)
        self.assertIn("            force_full_upload,\n        )?;", render_source)
        self.assertIn("if let Some(screen_space_ui_renderer) = self.screen_space_ui_renderer", scene_source)
        self.assertIn("screen_space_ui_renderer.record(", scene_source)
        self.assertIn("Some(streamer),\n            ) {", scene_source)
        self.assertIn("return Err(error);", scene_source)
        self.assertIn("screen_space_ui_renderer", graph_source)
        self.assertIn(".record(", graph_source)
        self.assertIn(".map_err(|error| error.to_string())?;", graph_source)


if __name__ == "__main__":
    unittest.main()
