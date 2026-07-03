import re
import unittest
from pathlib import Path


class ContactShadowRenderLayerSetTestFixtureTests(unittest.TestCase):
    def test_wgpu_product_fixture_uses_render_layer_set_fields(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        source_path = (
            repo_root
            / "zircon_plugins/rendering/features/contact_shadow/runtime/src/wgpu_product_tests.rs"
        )
        source = source_path.read_text(encoding="utf-8")

        self.assertIn("RenderLayerSet", source)
        self.assertIn(
            "fn default_render_layer_set() -> RenderLayerSet",
            source,
        )
        self.assertIn(
            "RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK)",
            source,
        )
        self.assertEqual(
            len(
                re.findall(
                    r"^\s*layer_mask: default_render_layer_set\(\),$",
                    source,
                    flags=re.MULTILINE,
                )
            ),
            2,
        )
        self.assertEqual(
            source.count("render_layer_mask: default_render_layer_set()"),
            1,
        )
        self.assertNotIn("layer_mask: DEFAULT_RENDER_LAYER_MASK", source)
        self.assertNotIn("render_layer_mask: DEFAULT_RENDER_LAYER_MASK", source)


if __name__ == "__main__":
    unittest.main()
