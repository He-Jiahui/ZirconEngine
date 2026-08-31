import unittest
from pathlib import Path


class RuntimePluginRendererOutputsOwnerStructureTests(unittest.TestCase):
    def test_renderer_feedback_domains_are_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = (
            repo_root
            / "zircon_runtime/src/core/framework/render/plugin_renderer_outputs.rs"
        )
        owner = owner_path.read_text(encoding="utf-8")
        owner_dir = owner_path.with_suffix("")
        virtual_geometry = (owner_dir / "virtual_geometry.rs").read_text(
            encoding="utf-8"
        )
        hybrid_gi = (owner_dir / "hybrid_gi.rs").read_text(encoding="utf-8")
        particles = (owner_dir / "particles.rs").read_text(encoding="utf-8")
        tests = (owner_dir / "tests.rs").read_text(encoding="utf-8")

        self.assertLessEqual(len(owner.splitlines()), 50)
        for child in ("hybrid_gi", "particles", "virtual_geometry"):
            self.assertIn(f"mod {child};", owner)
            self.assertIn(f"pub use self::{child}::", owner)
        self.assertIn("#[cfg(test)]\nmod tests;", owner)
        self.assertIn("pub struct RenderPluginRendererOutputs", owner)

        for moved_anchor in (
            "pub struct RenderParticleGpuReadbackOutputs",
            "pub struct RenderVirtualGeometryReadbackOutputs",
            "pub struct RenderHybridGiReadbackOutputs",
            "fn default_plugin_renderer_outputs_are_empty",
        ):
            self.assertNotIn(moved_anchor, owner)

        self.assertIn("pub struct RenderVirtualGeometryReadbackOutputs", virtual_geometry)
        self.assertIn(
            "pub struct RenderVirtualGeometryNodeClusterCullReadbackOutputs",
            virtual_geometry,
        )
        self.assertIn("pub struct RenderHybridGiReadbackOutputs", hybrid_gi)
        self.assertIn(
            "pub struct RenderHybridGiScenePrepareReadbackOutputs", hybrid_gi
        )
        self.assertIn("pub struct RenderParticleGpuReadbackOutputs", particles)
        self.assertIn("fn default_plugin_renderer_outputs_are_empty", tests)

        self.assertLessEqual(len(virtual_geometry.splitlines()), 100)
        self.assertLessEqual(len(hybrid_gi.splitlines()), 320)
        self.assertLessEqual(len(particles.splitlines()), 40)
        self.assertLessEqual(len(tests.splitlines()), 240)


if __name__ == "__main__":
    unittest.main()
