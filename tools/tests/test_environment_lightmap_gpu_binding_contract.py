from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ENVIRONMENT_MODULE = (
    REPO_ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs"
)
LIGHTMAP_BINDING = (
    REPO_ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/environment/lightmap_binding.rs"
)
FORWARD_BINDINGS = REPO_ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/"
    "forward_shadow_receiver.rs"
)
DEFERRED_LAYOUT = REPO_ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/deferred/"
    "lighting_bind_group_layout/create.rs"
)
DEFERRED_EXECUTE = REPO_ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/deferred/"
    "deferred_scene_resources/execute_lighting.rs"
)


class EnvironmentLightmapGpuBindingContractTests(unittest.TestCase):
    def test_environment_renderer_owns_the_lightmap_binding_module(self) -> None:
        module_source = ENVIRONMENT_MODULE.read_text(encoding="utf-8")

        self.assertIn("mod lightmap_binding;", module_source)
        self.assertIn("lightmap_bind_group_layout_entries", module_source)

    def test_plan_11_bindings_preserve_probe_grid_slot_23(self) -> None:
        source = LIGHTMAP_BINDING.read_text(encoding="utf-8")

        self.assertIn("LIGHT_PROBE_GRID_BINDING: u32 = 23", source)
        self.assertIn("LIGHTMAP_ATLAS_BINDING: u32 = 24", source)
        self.assertIn("LIGHTMAP_SAMPLER_BINDING: u32 = 28", source)
        self.assertIn("TextureViewDimension::D2Array", source)
        self.assertIn("BufferBindingType::Storage { read_only: true }", source)

    def test_forward_and_deferred_install_the_same_lightmap_resource_abi(self) -> None:
        forward = FORWARD_BINDINGS.read_text(encoding="utf-8")
        deferred_layout = DEFERRED_LAYOUT.read_text(encoding="utf-8")
        deferred_execute = DEFERRED_EXECUTE.read_text(encoding="utf-8")

        self.assertIn("lightmap_bind_group_layout_entries()", forward)
        self.assertIn("let lightmap_bindings = self.lightmaps.bindings()", forward)
        self.assertIn("lightmap_bindings.bind_group_entries()", forward)
        self.assertIn("lightmap_bind_group_layout_entries()", deferred_layout)
        self.assertIn("self.lightmap_bindings.bind_group_entries()", deferred_execute)


if __name__ == "__main__":
    unittest.main()
