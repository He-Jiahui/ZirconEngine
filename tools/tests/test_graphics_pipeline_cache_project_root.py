import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MESH_CONSTRUCT = REPO_ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/mesh/"
    "mesh_pipeline_cache/construct.rs"
)
RENDERER_CONSTRUCT = REPO_ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/core/"
    "scene_renderer_core_construct/construct/construct.rs"
)


class GraphicsPipelineCacheProjectRootTests(unittest.TestCase):
    def test_production_cache_construction_accepts_an_explicit_project_root(self) -> None:
        source = MESH_CONSTRUCT.read_text(encoding="utf-8")

        self.assertIn("project_root: &std::path::Path", source)
        self.assertIn("default_runtime_shader_cache(project_root)", source)
        self.assertIn("RuntimePipelineCache::new(device, info, project_root)", source)
        self.assertIn("#[cfg(test)]\n    pub(crate) fn new(", source)
        self.assertNotIn("std::env::current_dir()", source)

    def test_renderer_uses_the_active_project_root_before_constructing_caches(self) -> None:
        source = RENDERER_CONSTRUCT.read_text(encoding="utf-8")

        production_start = source.index("#[cfg(not(test))]")
        production_source = source[production_start:]
        self.assertIn("current_project_manager()", production_source)
        self.assertIn(".paths()", production_source)
        self.assertIn(".root()", production_source)
        self.assertIn("requires an active project before pipeline cache construction", production_source)
        self.assertNotIn("std::env::current_dir()", production_source)


if __name__ == "__main__":
    unittest.main()
