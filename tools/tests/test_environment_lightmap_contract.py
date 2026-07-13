from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ENVIRONMENT_MODULE = (
    REPO_ROOT / "zircon_runtime/src/core/framework/render/environment/mod.rs"
)
LIGHTMAP_CONTRACT = (
    REPO_ROOT / "zircon_runtime/src/core/framework/render/environment/lightmap.rs"
)
LIGHTMAP_TESTS = (
    REPO_ROOT / "zircon_runtime/src/core/framework/render/environment/lightmap/tests.rs"
)
BAKED_EXTRACT = (
    REPO_ROOT / "zircon_runtime/src/core/framework/render/light/snapshots.rs"
)
FRAME_EXTRACT = REPO_ROOT / "zircon_runtime/src/core/framework/render/frame_extract.rs"
POST_PROCESS_BAKED = REPO_ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/"
    "execute_post_process/execute/build_post_process_params/baked_lighting.rs"
)


class EnvironmentLightmapContractTests(unittest.TestCase):
    def test_environment_module_owns_the_baked_lighting_contract(self) -> None:
        module_source = ENVIRONMENT_MODULE.read_text(encoding="utf-8")

        self.assertIn("mod ambient;", module_source)
        self.assertIn("mod lightmap;", module_source)
        for contract_name in (
            "LightmapConsumeContract",
            "LightmapInstanceSlot",
            "LightProbeGridData",
            "LightmapBakeRequest",
            "LightmapBakeOutput",
            "ShL2Rgb",
        ):
            self.assertIn(contract_name, module_source)

    def test_contract_is_versioned_and_renderer_neutral(self) -> None:
        source = LIGHTMAP_CONTRACT.read_text(encoding="utf-8")

        self.assertIn("LIGHTMAP_CONSUME_CONTRACT_VERSION", source)
        self.assertIn("light_set_generation", source)
        self.assertIn("LightmapAtlasDescriptor", source)
        self.assertIn("LightmapAtlasFormat::Rgba16Float", source)
        self.assertIn("pub fn validate", source)
        self.assertNotIn("wgpu::", source)

    def test_bake_dto_carries_scene_budget_and_importable_atlas_pages(self) -> None:
        source = LIGHTMAP_CONTRACT.read_text(encoding="utf-8")

        for contract_anchor in (
            "pub scene_snapshot: LightmapBakeSceneSnapshot",
            "pub atlas_budget: LightmapAtlasBudget",
            "pub texel_density: f32",
            "pub atlas_pages: Vec<LightmapAtlasPage>",
            "pub fn validate_against",
            "pub fn into_consume_contract",
        ):
            self.assertIn(contract_anchor, source)

    def test_probe_index_math_is_checked_at_usize_width(self) -> None:
        source = LIGHTMAP_CONTRACT.read_text(encoding="utf-8")

        self.assertIn("fn probe_grid_index", source)
        self.assertIn("checked_mul", source)
        self.assertIn("checked_add", source)

    def test_environment_is_the_only_baked_lightmap_and_probe_owner(self) -> None:
        source = BAKED_EXTRACT.read_text(encoding="utf-8")
        frame_source = FRAME_EXTRACT.read_text(encoding="utf-8")
        post_source = POST_PROCESS_BAKED.read_text(encoding="utf-8")

        self.assertNotIn("LightmapConsumeContract", source)
        self.assertNotIn("LightProbeGridData", source)
        self.assertNotIn("try_with_consumption_contract", source)
        self.assertNotIn("pub baked_lighting: Option<RenderBakedLightingExtract>", frame_source)
        self.assertIn("extract.environment.baked_lighting()", post_source)

    def test_plan_11_behavior_gates_are_owned_by_the_contract_module(self) -> None:
        source = LIGHTMAP_TESTS.read_text(encoding="utf-8")

        for test_name in (
            "render_env_lightmap_uv_rect_transform_roundtrip",
            "render_env_lightmap_bake_dto_serde_roundtrip",
            "render_env_probe_grid_trilinear_center_equals_cell_average",
        ):
            self.assertIn(f"fn {test_name}()", source)


if __name__ == "__main__":
    unittest.main()
