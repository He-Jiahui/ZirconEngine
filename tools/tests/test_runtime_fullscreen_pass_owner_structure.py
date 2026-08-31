import unittest
from pathlib import Path


class RuntimeFullscreenPassOwnerStructureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        self.owner = (
            self.repo_root
            / "zircon_runtime/src/core/framework/render/shader/fullscreen_pass.rs"
        )
        self.owner_dir = self.owner.with_suffix("")

    def test_fullscreen_pass_uses_focused_folder_backed_owners(self) -> None:
        owner_source = self.owner.read_text(encoding="utf-8")
        production_lines = [
            line
            for line in owner_source.splitlines()
            if line.strip() and not line.lstrip().startswith("//")
        ]

        self.assertLessEqual(len(production_lines), 28)
        for declaration in (
            '#[path = "fullscreen_pass/abi.rs"]\nmod abi;',
            '#[path = "fullscreen_pass/builder.rs"]\nmod builder;',
            '#[path = "fullscreen_pass/parameter_encoding.rs"]\nmod parameter_encoding;',
            '#[path = "fullscreen_pass/pipeline_cache_key.rs"]\nmod pipeline_cache_key;',
            '#[path = "fullscreen_pass/plan.rs"]\nmod plan;',
            '#[path = "fullscreen_pass/shader_ref.rs"]\nmod shader_ref;',
            '#[cfg(test)]\n#[path = "fullscreen_pass/tests.rs"]\nmod tests;',
        ):
            self.assertIn(declaration, owner_source)

        for public_reexport in (
            "pub use builder::FullscreenPassBuilder;",
            "pub use pipeline_cache_key::FullscreenPipelineCacheKey;",
            "pub use plan::FullscreenPassPlan;",
            "pub use shader_ref::FullscreenShaderRef;",
        ):
            self.assertIn(public_reexport, owner_source)

        shader_facade = self.owner.parent / "mod.rs"
        shader_facade_source = shader_facade.read_text(encoding="utf-8")
        self.assertIn("pub use fullscreen_pass::{", shader_facade_source)
        for public_symbol in (
            "FullscreenPassBuilder",
            "FullscreenPassPlan",
            "FullscreenPipelineCacheKey",
            "FullscreenShaderRef",
            "FULLSCREEN_TRIANGLE_VERTEX_ENTRY",
        ):
            self.assertIn(public_symbol, shader_facade_source)

        expected_children = {
            "abi.rs": ("pub const FULLSCREEN_FRAME_GROUP",),
            "builder.rs": ("pub struct FullscreenPassBuilder",),
            "parameter_encoding.rs": ("fn fullscreen_parameter_words",),
            "pipeline_cache_key.rs": ("pub struct FullscreenPipelineCacheKey",),
            "plan.rs": ("pub struct FullscreenPassPlan",),
            "shader_ref.rs": ("pub struct FullscreenShaderRef",),
            "tests.rs": (
                "render_fullscreen_pass_builder_emits_pass_input_and_params_abi",
                "render_fullscreen_pass_builder_reports_stage_and_resource_errors",
            ),
        }
        for child_name, anchors in expected_children.items():
            child = self.owner_dir / child_name
            self.assertTrue(child.is_file(), child)
            child_source = child.read_text(encoding="utf-8")
            for anchor in anchors:
                self.assertIn(anchor, child_source)

        for forbidden in (
            "pub struct FullscreenShaderRef",
            "pub struct FullscreenPipelineCacheKey",
            "pub struct FullscreenPassPlan",
            "pub struct FullscreenPassBuilder",
            "fn fullscreen_parameter_words",
        ):
            self.assertNotIn(forbidden, owner_source)


if __name__ == "__main__":
    unittest.main()
