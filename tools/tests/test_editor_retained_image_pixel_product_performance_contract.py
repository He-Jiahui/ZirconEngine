import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PRIMITIVES = REPO_ROOT / "zircon_editor/src/ui/retained_host/primitives.rs"
RETAINED = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "visual_assets/retained.rs"
)


class EditorRetainedImagePixelProductPerformanceContract(unittest.TestCase):
    def test_image_precomputes_a_shared_pixel_product(self) -> None:
        source = PRIMITIVES.read_text(encoding="utf-8")
        image = source.split("pub(crate) struct Image {", 1)[1].split(
            "pub(crate) struct VecModel", 1
        )[0]

        self.assertIn("content_fingerprint: u64", image)
        self.assertIn("pub(crate) fn pixel_product", image)
        self.assertIn("rgba: &self.rgba", image)
        self.assertGreaterEqual(image.count("image_content_fingerprint("), 2)

    def test_untinted_paint_reuses_pixels_and_the_precomputed_key(self) -> None:
        source = RETAINED.read_text(encoding="utf-8")
        paint = source.split("fn retained_image_pixels(", 1)[1].split(
            "#[cfg(test)]", 1
        )[0]
        untinted = paint.rsplit("let image = HostPaintImagePixels", 1)[1]

        self.assertNotIn("to_rgba8", paint)
        self.assertEqual(paint.count("to_vec()"), 1)
        self.assertIn("Arc::clone(product.rgba)", untinted)
        self.assertIn("retained_image_resource_key_from_fingerprint", paint)
        self.assertIn("resource_key: base_key", untinted)
        self.assertNotIn("to_vec()", untinted)
        self.assertNotIn("retained_image_resource_key(", untinted)

    def test_tinted_paint_reuses_the_shared_visual_variant_cache(self) -> None:
        source = RETAINED.read_text(encoding="utf-8")
        paint = source.split("fn retained_image_pixels(", 1)[1].split(
            "#[cfg(test)]", 1
        )[0]
        tinted = paint.split("if let Some(tint) = tint", 1)[1].split(
            "let image = HostPaintImagePixels", 1
        )[0]

        lookup = tinted.index("cached_visual_asset_pixels")
        clone = tinted.index("product.rgba.as_ref().to_vec()")
        self.assertLess(lookup, clone)
        self.assertIn("image_pixels_cache_key", tinted)
        self.assertIn("store_visual_asset_pixels", paint)
        self.assertIn(
            "fn tinted_retained_image_reuses_the_shared_visual_variant_cache", source
        )
        self.assertIn(
            "fn tinted_retained_image_cache_separates_content_and_dimensions", source
        )


if __name__ == "__main__":
    unittest.main()
