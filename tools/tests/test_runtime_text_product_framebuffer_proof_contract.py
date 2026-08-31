import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PRODUCT_TEST = ROOT / "zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs"
PROOF_PATH = (
    ROOT
    / "zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_path.rs"
)
PRODUCT_RENDERER = (
    ROOT
    / "zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/product_renderer.rs"
)


class RuntimeTextProductFramebufferProofContractTests(unittest.TestCase):
    def test_current_proof_identity_targets_docs_and_not_cargo_output(self) -> None:
        source = PROOF_PATH.read_text(encoding="utf-8")

        self.assertIn(
            '"runtime_text_mvp_foundation_product_framebuffer_20260831.png"',
            source,
        )
        for segment in ['.join("docs")', '.join("tests")', '.join("runtime")', '.join("text")']:
            self.assertIn(segment, source)
        self.assertIn("assert_product_proof_is_outside_target", source)
        self.assertIn("CARGO_TARGET_DIR", source)
        self.assertIn("b'D' | b'd' | b'E' | b'e' | b'F' | b'f'", source)
        self.assertNotIn("temp_dir()", source)

    def test_product_test_uses_real_wgpu_readback_and_pixel_assertions(self) -> None:
        product = PRODUCT_TEST.read_text(encoding="utf-8")
        renderer = PRODUCT_RENDERER.read_text(encoding="utf-8")

        self.assertIn("WgpuRenderFramework::new", renderer)
        self.assertIn("capture_frame(self.viewport)", renderer)
        self.assertIn("write_product_framebuffer_png(&output, &capture.rgba", product)
        self.assertIn("count_changed_pixels_in_frame", product)
        self.assertIn("dominant_checker_channel_counts", product)
        self.assertIn("text_raster_capture_is_stable", renderer)

    def test_typed_icon_and_inline_image_use_the_imported_checker_texture(self) -> None:
        product = PRODUCT_TEST.read_text(encoding="utf-8")

        self.assertIn("res://ui/rich-inline-checker.png", product)
        self.assertIn("[icon=res://ui/rich-inline-checker.png", product)
        self.assertIn("typed icon asset through the WGPU image batch", product)
        self.assertNotIn("strategy", product.lower())


if __name__ == "__main__":
    unittest.main()
